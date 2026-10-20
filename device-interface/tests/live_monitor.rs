use ainavlog_device_interface::{
    monitor::{
        read_key_file, run_with_source, AdvertisementSource, MonitorConfig, MonitorState,
        VictronMonitor,
    },
    store::ObservationStore,
    vendor::victron::{VictronAdvertisement, VICTRON_COMPANY_ID},
    DeviceError, DeviceResult, QualityStatus,
};
use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant, SystemTime},
};
const KEY: &str = "000102030405060708090a0b0c0d0e0f";
const TR: &[u8] = include_bytes!("fixtures/victron/orion_tr.bin");
const NA: &[u8] = include_bytes!("fixtures/victron/orion_tr_na.bin");
const NEG: &[u8] = include_bytes!("fixtures/victron/orion_tr_negative.bin");
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Files(PathBuf);
impl Files {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "ainavlog-monitor-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn db(&self) -> PathBuf {
        self.0.join("observations.sqlite3")
    }
    fn key(&self) -> PathBuf {
        self.0.join("orion.key")
    }
    fn write_key(&self, key: &str) {
        std::fs::write(self.key(), key).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(self.key(), std::fs::Permissions::from_mode(0o600)).unwrap();
        }
    }
    fn config(&self) -> MonitorConfig {
        self.write_key(KEY);
        MonitorConfig {
            source_id: "test-orion".into(),
            key_file: self.key(),
            database: self.db(),
            adapter_index: 0,
            stale_after: Duration::from_secs(2),
            retry_delay: Duration::from_secs(1),
            restart_after: Duration::from_secs(4),
        }
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn input(data: &[u8]) -> VictronAdvertisement {
    VictronAdvertisement {
        source_id: "test-orion".into(),
        received_at: SystemTime::UNIX_EPOCH + Duration::from_secs(123),
        company_id: VICTRON_COMPANY_ID,
        payload: data.to_vec(),
    }
}
fn core(store: ObservationStore, now: Instant) -> VictronMonitor {
    VictronMonitor::new(
        "test-orion".into(),
        KEY,
        Duration::from_secs(10),
        store,
        now,
    )
    .unwrap()
}

#[test]
fn decoded_samples_survive_reopen_with_raw_metadata_and_nulls() {
    let files = Files::new();
    let now = Instant::now();
    let mut monitor = core(ObservationStore::open(&files.db()).unwrap(), now);
    let first = monitor.ingest(input(TR), now).unwrap().unwrap();
    assert_eq!(first.observations.len(), 5);
    assert!(first.observations.iter().all(|o| o.observed_at.is_none()));
    let missing = monitor
        .ingest(input(NA), now + Duration::from_secs(1))
        .unwrap()
        .unwrap();
    assert!(missing
        .observations
        .iter()
        .filter(|o| o.quantity != "off_reason")
        .all(|o| o.value.is_none() && o.quality == QualityStatus::Unknown));
    drop(monitor);
    let store = ObservationStore::open(&files.db()).unwrap();
    assert_eq!(store.recent(100).unwrap().len(), 10);
    let c = rusqlite::Connection::open(files.db()).unwrap();
    let (product, counter, raw): (u16, u16, Vec<u8>) = c
        .query_row(
            "SELECT product_id,data_counter,manufacturer_data FROM samples WHERE id=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!((product, counter), (0x1234, 0x1234));
    assert_eq!(raw, TR);
    let fields: String = c
        .query_row(
            "SELECT group_concat(name) FROM pragma_table_info('samples')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(!fields.contains("key"));
}

#[test]
fn duplicates_do_not_refresh_freshness_but_new_data_recovers() {
    let now = Instant::now();
    let wall = SystemTime::now();
    let mut monitor = core(ObservationStore::in_memory().unwrap(), now);
    monitor.ingest(input(TR), now).unwrap();
    assert!(monitor
        .ingest(input(TR), now + Duration::from_secs(1))
        .unwrap()
        .is_none());
    assert_eq!(
        monitor
            .tick(now + Duration::from_secs(11), wall)
            .unwrap()
            .unwrap()
            .state,
        MonitorState::Stale
    );
    assert!(monitor
        .ingest(input(TR), now + Duration::from_secs(12))
        .unwrap()
        .is_none());
    assert_eq!(monitor.state(), MonitorState::Stale);
    assert_eq!(monitor.store().recent(100).unwrap().len(), 5);
    assert_eq!(
        monitor
            .ingest(input(NEG), now + Duration::from_secs(13))
            .unwrap()
            .unwrap()
            .state,
        MonitorState::Live
    );
}

#[test]
fn key_rejection_stops_writes_until_explicit_replacement() {
    let now = Instant::now();
    let mut monitor = core(ObservationStore::in_memory().unwrap(), now);
    let mut wrong = input(TR);
    wrong.payload[7] = 1;
    assert_eq!(
        monitor.ingest(wrong, now).unwrap().unwrap().state,
        MonitorState::KeyRequired
    );
    assert!(monitor.ingest(input(TR), now).unwrap().is_none());
    assert!(monitor.store().recent(10).unwrap().is_empty());
    monitor.replace_key(KEY, now, SystemTime::now()).unwrap();
    assert!(monitor.ingest(input(TR), now).unwrap().is_some());
}

#[test]
fn observation_constraint_failure_rolls_back_whole_sample() {
    let files = Files::new();
    let now = Instant::now();
    let mut monitor = core(ObservationStore::in_memory().unwrap(), now);
    let observations = monitor
        .ingest(input(TR), now)
        .unwrap()
        .unwrap()
        .observations;
    let mut decoder = ainavlog_device_interface::vendor::victron::VictronAdvertisementDecoder::new(
        "test-orion",
        KEY,
    )
    .unwrap();
    let message = decoder.decode_advertisement(&input(TR)).unwrap();
    let mut store = ObservationStore::open(&files.db()).unwrap();
    let mut duplicated = observations.clone();
    duplicated.push(observations[0].clone());
    assert!(store.save(&message, TR, &duplicated).is_err());
    let c = rusqlite::Connection::open(files.db()).unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM samples", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert!(store.recent(10).unwrap().is_empty());
}

#[test]
fn key_file_and_database_validation_are_explicit() {
    let files = Files::new();
    files.write_key(&format!("{KEY}\n"));
    assert_eq!(&**read_key_file(&files.key()).unwrap(), KEY);
    files.write_key("not-a-key");
    assert!(read_key_file(&files.key()).is_err());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        files.write_key(KEY);
        std::fs::set_permissions(files.key(), std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(read_key_file(&files.key()).is_err());
    }
    let c = rusqlite::Connection::open(files.db()).unwrap();
    c.execute("CREATE TABLE unrelated (x)", []).unwrap();
    drop(c);
    assert!(ObservationStore::open(&files.db()).is_err());
}

#[derive(Default)]
struct Counts {
    starts: usize,
    stops: usize,
}
struct FakeSource {
    counts: Arc<Mutex<Counts>>,
    start_failures: usize,
    events: VecDeque<DeviceResult<VictronAdvertisement>>,
}
impl AdvertisementSource for FakeSource {
    async fn start(&mut self) -> DeviceResult<()> {
        self.counts.lock().unwrap().starts += 1;
        if self.start_failures > 0 {
            self.start_failures -= 1;
            Err(DeviceError::Transport("adapter offline".into()))
        } else {
            Ok(())
        }
    }
    async fn next(&mut self) -> DeviceResult<VictronAdvertisement> {
        if let Some(event) = self.events.pop_front() {
            event
        } else {
            std::future::pending().await
        }
    }
    async fn stop(&mut self) -> DeviceResult<()> {
        self.counts.lock().unwrap().stops += 1;
        Ok(())
    }
}

#[tokio::test(start_paused = true)]
async fn supervisor_recovers_startup_and_stream_failure_then_stops_scan() {
    let files = Files::new();
    let config = files.config();
    let counts = Arc::new(Mutex::new(Counts::default()));
    let source = FakeSource {
        counts: counts.clone(),
        start_failures: 1,
        events: VecDeque::from([Err(DeviceError::Disconnected), Ok(input(TR))]),
    };
    let mut reports = Vec::new();
    run_with_source(
        config,
        source,
        async {
            tokio::time::sleep(Duration::from_secs(9)).await;
            Ok(())
        },
        |r| reports.push(r.clone()),
    )
    .await
    .unwrap();
    assert!(counts.lock().unwrap().starts >= 3);
    assert!(counts.lock().unwrap().stops >= 3);
    assert!(reports.iter().any(|r| r.state == MonitorState::Recovering));
    assert!(reports.iter().any(|r| r.state == MonitorState::Live));
    assert!(reports.iter().any(|r| r.state == MonitorState::Stale));
    assert_eq!(reports.last().unwrap().state, MonitorState::Stopped);
    assert_eq!(
        ObservationStore::read_recent(&files.db(), 100)
            .unwrap()
            .len(),
        5
    );
}

#[tokio::test(start_paused = true)]
async fn malformed_packets_are_reported_and_next_valid_sample_is_saved() {
    let files = Files::new();
    let config = files.config();
    let counts = Arc::new(Mutex::new(Counts::default()));
    let source = FakeSource {
        counts,
        start_failures: 0,
        events: VecDeque::from([Ok(input(&TR[..3])), Ok(input(TR)), Ok(input(TR))]),
    };
    let mut reports = Vec::new();
    run_with_source(
        config,
        source,
        async {
            tokio::time::sleep(Duration::from_secs(1)).await;
            Ok(())
        },
        |r| reports.push(r.clone()),
    )
    .await
    .unwrap();
    assert!(reports.iter().any(|r| r.detail.contains("Packet rejected")));
    assert_eq!(reports.iter().filter(|r| r.sample_id.is_some()).count(), 1);
    assert_eq!(
        ObservationStore::read_recent(&files.db(), 100)
            .unwrap()
            .len(),
        5
    );
}

#[tokio::test(start_paused = true)]
async fn fatal_storage_error_still_stops_the_scan() {
    let files = Files::new();
    let config = files.config();
    drop(ObservationStore::open(&files.db()).unwrap());
    let c = rusqlite::Connection::open(files.db()).unwrap();
    c.execute_batch("CREATE TRIGGER fail_save BEFORE INSERT ON samples BEGIN SELECT RAISE(ABORT,'disk failure simulation'); END;").unwrap();
    drop(c);
    let counts = Arc::new(Mutex::new(Counts::default()));
    let source = FakeSource {
        counts: counts.clone(),
        start_failures: 0,
        events: VecDeque::from([Ok(input(TR))]),
    };
    let error = run_with_source(config, source, std::future::pending(), |_| {})
        .await
        .unwrap_err();
    assert!(matches!(error, DeviceError::Storage(_)));
    assert_eq!(counts.lock().unwrap().stops, 1);
    assert!(ObservationStore::read_recent(&files.db(), 10)
        .unwrap()
        .is_empty());
}

#[tokio::test(start_paused = true)]
async fn key_file_change_recovers_a_rejected_key_without_restart() {
    struct TimedSource {
        events: VecDeque<(tokio::time::Instant, VictronAdvertisement)>,
    }
    impl AdvertisementSource for TimedSource {
        async fn start(&mut self) -> DeviceResult<()> {
            Ok(())
        }
        async fn next(&mut self) -> DeviceResult<VictronAdvertisement> {
            if let Some((at, _)) = self.events.front() {
                tokio::time::sleep_until(*at).await;
                Ok(self.events.pop_front().unwrap().1)
            } else {
                std::future::pending().await
            }
        }
        async fn stop(&mut self) -> DeviceResult<()> {
            Ok(())
        }
    }
    let files = Files::new();
    let config = files.config();
    files.write_key("010102030405060708090a0b0c0d0e0f");
    let key_path = files.key();
    let updater = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        std::fs::write(key_path, KEY).unwrap();
    });
    let now = tokio::time::Instant::now();
    let source = TimedSource {
        events: VecDeque::from([(now, input(TR)), (now + Duration::from_secs(3), input(TR))]),
    };
    let mut reports = Vec::new();
    run_with_source(
        config,
        source,
        async {
            tokio::time::sleep(Duration::from_secs(4)).await;
            Ok(())
        },
        |r| reports.push(r.clone()),
    )
    .await
    .unwrap();
    updater.await.unwrap();
    assert!(reports.iter().any(|r| r.state == MonitorState::KeyRequired));
    assert!(reports.iter().any(|r| r.detail.contains("key reloaded")));
    assert_eq!(reports.iter().filter(|r| r.sample_id.is_some()).count(), 1);
}

#[tokio::test(start_paused = true)]
async fn shutdown_cancels_initialization_and_runs_cleanup() {
    struct HangingSource(Arc<Mutex<Counts>>);
    impl AdvertisementSource for HangingSource {
        async fn start(&mut self) -> DeviceResult<()> {
            self.0.lock().unwrap().starts += 1;
            std::future::pending().await
        }
        async fn next(&mut self) -> DeviceResult<VictronAdvertisement> {
            std::future::pending().await
        }
        async fn stop(&mut self) -> DeviceResult<()> {
            self.0.lock().unwrap().stops += 1;
            Ok(())
        }
    }
    let files = Files::new();
    let config = files.config();
    let counts = Arc::new(Mutex::new(Counts::default()));
    run_with_source(
        config,
        HangingSource(counts.clone()),
        async {
            tokio::time::sleep(Duration::from_secs(1)).await;
            Ok(())
        },
        |_| {},
    )
    .await
    .unwrap();
    assert_eq!(counts.lock().unwrap().starts, 1);
    assert_eq!(counts.lock().unwrap().stops, 1);
}
