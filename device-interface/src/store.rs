//! Local SQLite history. Packets and their observations commit atomically.
//! No advertisement keys are written to this database.
use crate::{vendor::victron::VictronMessage, DeviceError, DeviceResult, Observation};
use rusqlite::{params, Connection, OpenFlags};
use std::{
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn storage(e: impl std::fmt::Display) -> DeviceError {
    DeviceError::Storage(e.to_string())
}
pub fn unix_millis(time: SystemTime) -> DeviceResult<i64> {
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

impl ObservationStore {
    /// Return one complete BMV packet for the selected device, never mixing history.
    pub fn read_latest_bmv(path: &Path, source: &str) -> DeviceResult<Vec<StoredObservation>> {
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
    pub fn open(path: &Path) -> DeviceResult<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(storage)?;
        }
        Self::initialize(Connection::open(path).map_err(storage)?)
    }
    pub fn in_memory() -> DeviceResult<Self> {
        Self::initialize(Connection::open_in_memory().map_err(storage)?)
    }
    fn initialize(connection: Connection) -> DeviceResult<Self> {
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
        message: &VictronMessage,
        payload: &[u8],
        observations: &[Observation],
    ) -> DeviceResult<i64> {
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
        let record_type = match message.record {
            crate::vendor::victron::OrionRecord::DcDc(_) => 4,
            crate::vendor::victron::OrionRecord::OrionXs(_) => 15,
            crate::vendor::victron::OrionRecord::BatteryMonitor(_) => 2,
        };
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
    ) -> DeviceResult<()> {
        self.connection.execute("INSERT INTO monitor_events(source_id,recorded_at_ms,state,detail) VALUES (?1,?2,?3,?4)",params![source,unix_millis(time)?,state,detail]).map_err(storage)?;
        Ok(())
    }
    /// Historical observations, newest first. Quality describes ingestion, not
    /// current freshness; compare received_at_ms with the reader's current clock.
    pub fn recent(&self, limit: usize) -> DeviceResult<Vec<StoredObservation>> {
        recent(&self.connection, limit)
    }
    /// Read existing history without creating or modifying a database.
    pub fn read_recent(path: &Path, limit: usize) -> DeviceResult<Vec<StoredObservation>> {
        let connection =
            Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(storage)?;
        recent(&connection, limit)
    }
}
fn recent(connection: &Connection, limit: usize) -> DeviceResult<Vec<StoredObservation>> {
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
