//! Local SQLite history. Packets and their observations commit atomically.
//! No advertisement keys are written to this database.
use ainavlog_data_model::{AppError, AppResult, Observation, SampleMetadata};
use rusqlite::{params, Connection, OpenFlags};
use std::{
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn storage(e: impl std::fmt::Display) -> AppError {
    AppError::Storage(e.to_string())
}
pub fn unix_millis(time: SystemTime) -> AppResult<i64> {
    let duration = time.duration_since(UNIX_EPOCH).map_err(storage)?;
    i64::try_from(duration.as_millis()).map_err(storage)
}

pub struct ObservationStore {
    connection: Connection,
}

#[derive(Debug, Clone)]
pub struct StoredObservation {
    pub sample_id: i64,
    pub source_id: String,
    pub received_at_ms: i64,
    pub quantity: String,
    pub unit: String,
    pub value: Option<f64>,
    pub quality: String,
}

/// Actual packet values selected for a bounded chart, never interpolated.
#[derive(Debug, Clone)]
pub struct BmvHistoryPoint {
    pub received_at_ms: i64,
    pub segment: i64,
    pub house_voltage: Option<f64>,
    pub house_current: Option<f64>,
    pub house_charge: Option<f64>,
    pub starter_voltage: Option<f64>,
}

impl ObservationStore {
    /// Read without opening a receiver. Preserve first/last and each metric's
    /// extrema per chart bucket/continuous segment. Gaps >5s and availability
    /// changes split segments, so downsampling never bridges missing recordings.
    /// Bounds are [start, end); pathological fragmentation returns an error,
    /// never silently truncated history. No file is created for an empty device.
    pub fn read_bmv_history(
        path: &Path,
        source: &str,
        start: i64,
        end: i64,
    ) -> AppResult<Vec<BmvHistoryPoint>> {
        if source.trim().is_empty() || start < 0 || end <= start || end - start > 32 * 86_400_000 {
            return Err(storage("invalid history range (maximum 32 days)"));
        }
        if !path.try_exists().map_err(storage)? {
            return Ok(Vec::new());
        }
        let connection =
            Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(storage)?;
        connection
            .busy_timeout(Duration::from_secs(2))
            .map_err(storage)?;
        let bucket = ((end - start) / 360).max(1);
        let mut query = connection.prepare(
            "WITH packet AS (
                SELECT s.id, s.received_at_ms AS t,
                MAX(CASE WHEN o.quantity='battery_voltage' AND o.unit='V' AND o.quality='Good' THEN o.value END) AS v,
                MAX(CASE WHEN o.quantity='battery_current' AND o.unit='A' AND o.quality='Good' THEN o.value END) AS a,
                MAX(CASE WHEN o.quantity='state_of_charge' AND o.unit='%' AND o.quality='Good' THEN o.value END) AS soc,
                MAX(CASE WHEN o.quantity='auxiliary_voltage' AND o.unit='V' AND o.quality='Good' THEN o.value END) AS starter
                FROM samples s LEFT JOIN observations o ON o.sample_id=s.id
                WHERE s.source_id=?1 AND s.record_type=2 AND s.received_at_ms>=?2 AND s.received_at_ms<?3
                GROUP BY s.id
            ), previous AS (
                SELECT *, LAG(t) OVER w AS prev,
                LAG(v IS NULL) OVER w AS pv, LAG(a IS NULL) OVER w AS pa,
                LAG(soc IS NULL) OVER w AS ps, LAG(starter IS NULL) OVER w AS pst
                FROM packet WINDOW w AS (ORDER BY t,id)
            ), segmented AS (
                SELECT *, SUM(CASE WHEN prev IS NULL OR t-prev>5000
                OR pv!=(v IS NULL) OR pa!=(a IS NULL) OR ps!=(soc IS NULL) OR pst!=(starter IS NULL)
                THEN 1 ELSE 0 END) OVER (ORDER BY t,id) AS segment FROM previous
            ), ranked AS (
                SELECT *, ROW_NUMBER() OVER (PARTITION BY (t-?2)/?4,segment ORDER BY t,id) AS first,
                ROW_NUMBER() OVER (PARTITION BY (t-?2)/?4,segment ORDER BY t DESC,id DESC) AS last,
                ROW_NUMBER() OVER (PARTITION BY (t-?2)/?4,segment ORDER BY v,t,id) AS vmin,
                ROW_NUMBER() OVER (PARTITION BY (t-?2)/?4,segment ORDER BY v DESC,t,id) AS vmax,
                ROW_NUMBER() OVER (PARTITION BY (t-?2)/?4,segment ORDER BY a,t,id) AS amin,
                ROW_NUMBER() OVER (PARTITION BY (t-?2)/?4,segment ORDER BY a DESC,t,id) AS amax,
                ROW_NUMBER() OVER (PARTITION BY (t-?2)/?4,segment ORDER BY soc,t,id) AS smin,
                ROW_NUMBER() OVER (PARTITION BY (t-?2)/?4,segment ORDER BY soc DESC,t,id) AS smax,
                ROW_NUMBER() OVER (PARTITION BY (t-?2)/?4,segment ORDER BY starter,t,id) AS stmin,
                ROW_NUMBER() OVER (PARTITION BY (t-?2)/?4,segment ORDER BY starter DESC,t,id) AS stmax
                FROM segmented
            ) SELECT t,segment,v,a,soc,starter FROM ranked
            WHERE first=1 OR last=1 OR vmin=1 OR vmax=1 OR amin=1 OR amax=1 OR smin=1 OR smax=1 OR stmin=1 OR stmax=1
            ORDER BY t,id LIMIT 8193"
        ).map_err(storage)?;
        let rows = query
            .query_map(params![source, start, end, bucket], |r| {
                Ok(BmvHistoryPoint {
                    received_at_ms: r.get(0)?,
                    segment: r.get(1)?,
                    house_voltage: r.get(2)?,
                    house_current: r.get(3)?,
                    house_charge: r.get(4)?,
                    starter_voltage: r.get(5)?,
                })
            })
            .map_err(storage)?;
        let points = rows.collect::<Result<Vec<_>, _>>().map_err(storage)?;
        if points.len() > 8192 {
            return Err(storage(
                "history has too many recording gaps; select a shorter range",
            ));
        }
        Ok(points)
    }
    /// Return one complete BMV packet for the selected device, never mixing history.
    pub fn read_latest_bmv(path: &Path, source: &str) -> AppResult<Vec<StoredObservation>> {
        let connection =
            Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(storage)?;
        connection
            .busy_timeout(Duration::from_secs(2))
            .map_err(storage)?;
        let mut statement = connection.prepare("SELECT s.id,s.source_id,s.received_at_ms,o.quantity,o.unit,o.value,o.quality FROM samples s JOIN observations o ON o.sample_id=s.id WHERE s.id=(SELECT id FROM samples WHERE source_id=?1 AND record_type=2 ORDER BY id DESC LIMIT 1)").map_err(storage)?;
        let rows = statement
            .query_map([source], |r| {
                Ok(StoredObservation {
                    sample_id: r.get(0)?,
                    source_id: r.get(1)?,
                    received_at_ms: r.get(2)?,
                    quantity: r.get(3)?,
                    unit: r.get(4)?,
                    value: r.get(5)?,
                    quality: r.get(6)?,
                })
            })
            .map_err(storage)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(storage)
    }
    pub fn open(path: &Path) -> AppResult<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(storage)?;
        }
        Self::initialize(Connection::open(path).map_err(storage)?)
    }
    pub fn in_memory() -> AppResult<Self> {
        Self::initialize(Connection::open_in_memory().map_err(storage)?)
    }
    fn initialize(connection: Connection) -> AppResult<Self> {
        connection
            .busy_timeout(Duration::from_secs(2))
            .map_err(storage)?;
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(storage)?;
        let application: i64 = connection
            .pragma_query_value(None, "application_id", |r| r.get(0))
            .map_err(storage)?;
        if version > 1 || (application != 0 && application != 0x41494e4c) {
            return Err(storage(
                "database belongs to another application or newer schema",
            ));
        }
        if application == 0 {
            let tables: i64 = connection.query_row("SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'", [], |r| r.get(0)).map_err(storage)?;
            if tables != 0 {
                return Err(storage(
                    "refusing to initialize a nonempty unrelated database",
                ));
            }
        }
        connection
            .execute_batch(
                "PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;
            BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS samples (
                id INTEGER PRIMARY KEY, source_id TEXT NOT NULL, received_at_ms INTEGER NOT NULL,
                product_id INTEGER NOT NULL, data_counter INTEGER NOT NULL,
                record_type INTEGER NOT NULL, manufacturer_data BLOB NOT NULL);
            CREATE TABLE IF NOT EXISTS observations (
                sample_id INTEGER NOT NULL REFERENCES samples(id), quantity TEXT NOT NULL,
                unit TEXT NOT NULL, value REAL, quality TEXT NOT NULL,
                observed_at_ms INTEGER, PRIMARY KEY(sample_id, quantity));
            CREATE INDEX IF NOT EXISTS samples_source_time ON samples(source_id, received_at_ms);
            CREATE TABLE IF NOT EXISTS monitor_events (
                id INTEGER PRIMARY KEY, source_id TEXT NOT NULL, recorded_at_ms INTEGER NOT NULL,
                state TEXT NOT NULL, detail TEXT NOT NULL);
            PRAGMA application_id=1095323212; PRAGMA user_version=1; COMMIT;",
            )
            .map_err(storage)?;
        Ok(Self { connection })
    }
    pub fn save(
        &mut self,
        message: &SampleMetadata,
        payload: &[u8],
        observations: &[Observation],
    ) -> AppResult<i64> {
        if observations.iter().any(|o| {
            o.source_id != message.source_id
                || o.received_at != message.received_at
                || o.value.is_some_and(|v| !v.is_finite())
        }) {
            return Err(storage(
                "observation does not match sample or has a non-finite value",
            ));
        }
        let received = unix_millis(message.received_at)?;
        let record_type = message.record_type;
        let tx = self.connection.transaction().map_err(storage)?;
        tx.execute("INSERT INTO samples(source_id,received_at_ms,product_id,data_counter,record_type,manufacturer_data) VALUES (?1,?2,?3,?4,?5,?6)",
            params![message.source_id, received, message.product_id, message.data_counter, record_type, payload]).map_err(storage)?;
        let id = tx.last_insert_rowid();
        for o in observations {
            let observed = o.observed_at.map(unix_millis).transpose()?;
            tx.execute("INSERT INTO observations(sample_id,quantity,unit,value,quality,observed_at_ms) VALUES (?1,?2,?3,?4,?5,?6)",
                params![id,o.quantity,o.unit,o.value,format!("{:?}",o.quality),observed]).map_err(storage)?;
        }
        tx.commit().map_err(storage)?;
        Ok(id)
    }
    pub fn record_event(
        &self,
        source: &str,
        time: SystemTime,
        state: &str,
        detail: &str,
    ) -> AppResult<()> {
        self.connection.execute("INSERT INTO monitor_events(source_id,recorded_at_ms,state,detail) VALUES (?1,?2,?3,?4)",params![source,unix_millis(time)?,state,detail]).map_err(storage)?;
        Ok(())
    }
    /// Historical observations, newest first. Quality describes ingestion, not
    /// current freshness; compare received_at_ms with the reader's current clock.
    pub fn recent(&self, limit: usize) -> AppResult<Vec<StoredObservation>> {
        recent(&self.connection, limit)
    }
    /// Read existing history without creating or modifying a database.
    pub fn read_recent(path: &Path, limit: usize) -> AppResult<Vec<StoredObservation>> {
        let connection =
            Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(storage)?;
        recent(&connection, limit)
    }
}
fn recent(connection: &Connection, limit: usize) -> AppResult<Vec<StoredObservation>> {
    if !(1..=10000).contains(&limit) {
        return Err(storage("history limit must be 1..=10000"));
    }
    let mut statement = connection.prepare("SELECT s.id,s.source_id,s.received_at_ms,o.quantity,o.unit,o.value,o.quality FROM samples s JOIN observations o ON o.sample_id=s.id ORDER BY s.id DESC,o.quantity LIMIT ?1").map_err(storage)?;
    let rows = statement
        .query_map([limit as i64], |r| {
            Ok(StoredObservation {
                sample_id: r.get(0)?,
                source_id: r.get(1)?,
                received_at_ms: r.get(2)?,
                quantity: r.get(3)?,
                unit: r.get(4)?,
                value: r.get(5)?,
                quality: r.get(6)?,
            })
        })
        .map_err(storage)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(storage)
}

#[cfg(test)]
mod history_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Database(std::path::PathBuf);
    impl Database {
        fn new() -> Self {
            Self(std::env::temp_dir().join(format!(
                "ainavlog-history-{}-{}.sqlite3",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            )))
        }
    }
    impl Drop for Database {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    fn insert(
        connection: &Connection,
        source: &str,
        time: i64,
        soc: Option<f64>,
        voltage: Option<f64>,
    ) {
        connection.execute("INSERT INTO samples(source_id,received_at_ms,product_id,data_counter,record_type,manufacturer_data) VALUES(?1,?2,4660,1,2,x'00')",params![source,time]).unwrap();
        let id = connection.last_insert_rowid();
        for (quantity, unit, value) in [
            ("state_of_charge", "%", soc),
            ("battery_voltage", "V", voltage),
            ("battery_current", "A", Some(-5.25)),
            ("auxiliary_voltage", "V", Some(12.8)),
        ] {
            connection.execute("INSERT INTO observations(sample_id,quantity,unit,value,quality) VALUES(?1,?2,?3,?4,?5)",params![id,quantity,unit,value,if value.is_some(){"Good"}else{"Unknown"}]).unwrap();
        }
    }
    #[test]
    fn disconnected_read_preserves_values_range_source_and_gaps() {
        let file = Database::new();
        assert!(ObservationStore::read_bmv_history(&file.0, "sim", 0, 100)
            .unwrap()
            .is_empty());
        assert!(!file.0.exists());
        let store = ObservationStore::open(&file.0).unwrap();
        insert(&store.connection, "sim", 1000, Some(77.5), Some(12.76));
        insert(&store.connection, "other", 2000, Some(99.0), Some(99.0));
        insert(&store.connection, "sim", 3000, None, Some(12.7));
        insert(&store.connection, "sim", 4000, Some(77.4), Some(12.7));
        insert(&store.connection, "sim", 15000, Some(77.3), Some(12.6));
        drop(store); // no receiver or writer is required
        let rows = ObservationStore::read_bmv_history(&file.0, "sim", 1000, 15000).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].house_voltage, Some(12.76));
        assert_eq!(rows[0].house_current, Some(-5.25));
        assert_eq!(rows[1].house_charge, None);
        assert_ne!(rows[0].segment, rows[1].segment);
        assert_ne!(rows[1].segment, rows[2].segment);
        let rows = ObservationStore::read_bmv_history(&file.0, "sim", 4000, 15001).unwrap();
        assert_eq!(rows.len(), 2);
        assert_ne!(rows[0].segment, rows[1].segment);
        assert!(ObservationStore::read_bmv_history(&file.0, "sim", 0, 0).is_err());
        assert!(ObservationStore::read_bmv_history(&file.0, "sim", 0, 33 * 86_400_000).is_err());
    }
    #[test]
    fn dense_history_is_bounded_and_keeps_extrema_and_packet_values() {
        let file = Database::new();
        let mut store = ObservationStore::open(&file.0).unwrap();
        let tx = store.connection.transaction().unwrap();
        for i in 0..20000 {
            insert(
                &tx,
                "sim",
                i * 1000,
                Some(if i == 4321 {
                    12.3
                } else if i == 9876 {
                    98.7
                } else {
                    75.0
                }),
                Some(if i == 3456 { 14.6 } else { 12.8 }),
            );
        }
        tx.commit().unwrap();
        let rows = ObservationStore::read_bmv_history(&file.0, "sim", 0, 86_400_000).unwrap();
        assert!(
            rows.len() < 1000,
            "dense rows must be reduced, not returned unbounded"
        );
        assert_eq!(rows.first().unwrap().received_at_ms, 0);
        assert_eq!(rows.last().unwrap().received_at_ms, 19999000);
        assert!(rows.iter().all(|r| r.segment == 1));
        assert!(rows
            .iter()
            .any(|r| r.house_charge == Some(12.3) && r.received_at_ms == 4321000));
        assert!(rows
            .iter()
            .any(|r| r.house_charge == Some(98.7) && r.received_at_ms == 9876000));
        assert!(rows
            .iter()
            .any(|r| r.house_voltage == Some(14.6) && r.house_current == Some(-5.25)));
    }
    #[test]
    fn excessive_fragmentation_errors_instead_of_silently_truncating() {
        let file = Database::new();
        let mut store = ObservationStore::open(&file.0).unwrap();
        let tx = store.connection.transaction().unwrap();
        for i in 0..8193 {
            insert(&tx, "sim", i * 6000, Some(75.0), Some(12.8));
        }
        tx.commit().unwrap();
        assert!(ObservationStore::read_bmv_history(&file.0, "sim", 0, 86_400_000).is_err());
    }
}
