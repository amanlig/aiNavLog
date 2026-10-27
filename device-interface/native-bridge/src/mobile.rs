//! The phone transport is btleplug; native Expo wrappers only manage app integration.
use super::*;
use ainavlog_device_interface::bluetooth::BluetoothAdapter;
use btleplug::api::{CentralEvent, CentralState};
use std::{
    ffi::CString,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
    thread,
    time::SystemTime,
};
use tokio::sync::watch;

const SOURCE: &str = "windows-bmv-simulator";
const KEY: &str = "000102030405060708090a0b0c0d0e0f";
struct Scanner {
    handle: u64,
    stop: watch::Sender<bool>,
    worker: thread::JoinHandle<()>,
    status: Arc<AtomicI32>,
    stats: Arc<Mutex<ScanStats>>,
}
static SCANNER: Mutex<Option<Scanner>> = Mutex::new(None);

// Never choose a real instrument merely because it advertises Victron data.
fn simulator_packet(company: u16, payload: &[u8]) -> bool {
    company == 0x02e1 && payload.len() >= 24 && payload[..5] == [0x10, 0x02, 0x34, 0x12, 0x02]
}

// OS radio identities may change between advertisements. The public simulator
// is selected by its payload, not by a real-instrument identity or private key.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SimulatorDiagnostics {
    pub packets: u64,
    pub samples: u64,
    pub source_changes: u64,
}
#[derive(Default)]
struct ScanStats {
    counters: SimulatorDiagnostics,
    last_radio: Option<String>,
}
fn ingest_simulator(
    handle: u64,
    radio: &str,
    company: u16,
    payload: &[u8],
    millis: i64,
    stats: &Mutex<ScanStats>,
) -> Option<i32> {
    if !simulator_packet(company, payload) {
        return None;
    }
    let source = CString::new(SOURCE).unwrap();
    let result = unsafe {
        ainavlog_ingest(
            handle,
            source.as_ptr(),
            company,
            payload.as_ptr(),
            payload.len(),
            millis,
        )
    };
    if let Ok(mut stats) = stats.lock() {
        stats.counters.packets += 1;
        if stats
            .last_radio
            .as_deref()
            .is_some_and(|previous| previous != radio)
        {
            stats.counters.source_changes += 1;
        }
        stats.last_radio = Some(radio.to_owned());
        if result == 0 {
            stats.counters.samples += 1;
        }
    }
    Some(result)
}

async fn receive(handle: u64, status: &AtomicI32, stats: &Mutex<ScanStats>) -> Result<(), ()> {
    let mut adapter = tokio::time::timeout(Duration::from_secs(20), BluetoothAdapter::new(0))
        .await
        .map_err(|_| ())?
        .map_err(|_| ())?;
    let mut scan = adapter.start_advertisements().await.map_err(|_| ())?;
    status.store(10, Ordering::SeqCst);
    let mut silence = tokio::time::Instant::now();
    let result = loop {
        // Restart a silent backend; btleplug's scan guard also cleans up on cancellation.
        let remaining = Duration::from_secs(30).saturating_sub(silence.elapsed());
        let event = match tokio::time::timeout(remaining, scan.next_event()).await {
            Ok(Ok(event)) => event,
            _ => break Err(()),
        };
        match event {
            CentralEvent::ManufacturerDataAdvertisement {
                id,
                manufacturer_data,
            } => {
                if let Some(payload) = manufacturer_data.get(&0x02e1) {
                    let millis = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|_| ())?
                        .as_millis();
                    if let Some(result) = ingest_simulator(
                        handle,
                        &id.to_string(),
                        0x02e1,
                        payload,
                        millis as i64,
                        stats,
                    ) {
                        if result < 0 || result == 2 {
                            break Err(());
                        }
                        if result == 0 {
                            status.store(11, Ordering::SeqCst);
                            silence = tokio::time::Instant::now();
                        }
                    }
                }
            }
            CentralEvent::StateUpdate(CentralState::PoweredOff) => break Err(()),
            _ => {}
        }
        if silence.elapsed() >= Duration::from_secs(30) {
            break Err(());
        }
    };
    let _ = scan.stop().await;
    result
}

fn stop(scanner: &mut Option<Scanner>) -> i32 {
    if let Some(scanner) = scanner.take() {
        let _ = scanner.stop.send(true);
        let joined = scanner.worker.join();
        let closed = ainavlog_close(scanner.handle);
        if joined.is_err() {
            return -3;
        }
        return closed;
    }
    0
}

/// Starts only the public BMV simulator, never arbitrary instruments.
/// # Safety
/// Path is NUL-terminated UTF-8; output points to writable u64 storage.
#[no_mangle]
pub unsafe extern "C" fn ainavlog_simulator_start(path: *const c_char, output: *mut u64) -> i32 {
    if output.is_null() {
        return -1;
    }
    *output = 0;
    catch_unwind(AssertUnwindSafe(|| {
        let Ok(mut scanner) = SCANNER.lock() else {
            return -3;
        };
        let result = stop(&mut scanner);
        if result != 0 {
            return result;
        }
        let source = CString::new(SOURCE).unwrap();
        let key = CString::new(KEY).unwrap();
        let result = ainavlog_open(path, source.as_ptr(), key.as_ptr(), output);
        if result != 0 {
            return result;
        }
        let handle = *output;
        let (sender, mut receiver) = watch::channel(false);
        let status = Arc::new(AtomicI32::new(10));
        let worker_status = status.clone();
        let stats = Arc::new(Mutex::new(ScanStats::default()));
        let worker_stats = stats.clone();
        let worker = thread::Builder::new()
            .name("ainavlog-ble".into())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(_) => {
                        worker_status.store(-2, Ordering::SeqCst);
                        return;
                    }
                };
                runtime.block_on(async {
                    loop {
                        tokio::select! {
                            biased;
                            _ = receiver.changed() => break,
                            _ = receive(handle, &worker_status, &worker_stats) => {}
                        }
                        worker_status.store(-2, Ordering::SeqCst);
                        tokio::select! {
                            _ = receiver.changed() => break,
                            _ = tokio::time::sleep(Duration::from_secs(3)) => {}
                        }
                    }
                });
                // Give dropped scan guards time to execute stop_scan before shutdown.
                runtime.block_on(async { tokio::time::sleep(Duration::from_millis(100)).await });
                runtime.shutdown_timeout(Duration::from_secs(2));
            });
        match worker {
            Ok(worker) => {
                *scanner = Some(Scanner {
                    handle,
                    stop: sender,
                    worker,
                    status,
                    stats,
                });
                0
            }
            Err(_) => {
                ainavlog_close(handle);
                *output = 0;
                -3
            }
        }
    }))
    .unwrap_or(-3)
}

#[no_mangle]
pub extern "C" fn ainavlog_simulator_stop() -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        SCANNER
            .lock()
            .map(|mut scanner| stop(&mut scanner))
            .unwrap_or(-3)
    }))
    .unwrap_or(-3)
}

#[no_mangle]
pub extern "C" fn ainavlog_simulator_status() -> i32 {
    SCANNER
        .lock()
        .map(|scanner| {
            scanner
                .as_ref()
                .map(|s| s.status.load(Ordering::SeqCst))
                .unwrap_or(0)
        })
        .unwrap_or(-3)
}

/// Returns counters for the current session; stopped sessions return zeroes.
/// # Safety
/// Output must point to writable aligned SimulatorDiagnostics storage.
#[no_mangle]
pub unsafe extern "C" fn ainavlog_simulator_diagnostics(output: *mut SimulatorDiagnostics) -> i32 {
    if output.is_null() {
        return -1;
    }
    *output = SimulatorDiagnostics::default();
    let Ok(scanner) = SCANNER.lock() else {
        return -3;
    };
    if let Some(scanner) = scanner.as_ref() {
        let Ok(stats) = scanner.stats.lock() else {
            return -3;
        };
        *output = stats.counters;
    }
    0
}

#[cfg(target_os = "android")]
/// # Safety
/// Must be a live JNIEnv* on a thread entered from the app's Java class loader.
#[no_mangle]
pub unsafe extern "C" fn ainavlog_android_init(raw: *mut std::ffi::c_void) -> i32 {
    if raw.is_null() {
        return -1;
    }
    let mut env = jni::EnvUnowned::from_raw(raw.cast());
    match env
        .with_env(|env| -> Result<(), jni::errors::Error> {
            btleplug::platform::init(env)
                .map_err(|_| jni::errors::Error::NullPtr("btleplug initialization"))
        })
        .into_outcome()
    {
        jni::Outcome::Ok(()) => 0,
        _ => -2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rotating_simulator_identities_keep_storing_samples_without_refreshing_duplicates() {
        let path = std::env::temp_dir().join(format!(
            "ainavlog-rotation-{}-{}.sqlite3",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let text = CString::new(path.to_str().unwrap()).unwrap();
        let source = CString::new(SOURCE).unwrap();
        let key = CString::new(KEY).unwrap();
        let mut handle = 0;
        assert_eq!(
            unsafe { ainavlog_open(text.as_ptr(), source.as_ptr(), key.as_ptr(), &mut handle) },
            0
        );
        let stats = Mutex::new(ScanStats::default());
        let first = include_bytes!("../../tests/fixtures/victron/bmv_starter.bin");
        let next = include_bytes!("../../tests/fixtures/victron/bmv_na.bin");
        assert_eq!(
            ingest_simulator(handle, "radio-a", 0x02e1, first, 1000, &stats),
            Some(0)
        );
        assert_eq!(
            ingest_simulator(handle, "radio-b", 0x02e1, next, 2000, &stats),
            Some(0)
        );
        assert_eq!(
            ingest_simulator(handle, "radio-c", 0x02e1, next, 3000, &stats),
            Some(1)
        );
        assert_eq!(
            ingest_simulator(handle, "other", 0, first, 4000, &stats),
            None
        );
        let mut real = *first;
        real[2] = 0;
        assert_eq!(
            ingest_simulator(handle, "real-device", 0x02e1, &real, 4000, &stats),
            None
        );
        let counters = stats.lock().unwrap().counters;
        assert_eq!(
            (counters.packets, counters.samples, counters.source_changes),
            (3, 2, 2)
        );
        let mut sample = ElectricalSample::default();
        assert_eq!(unsafe { ainavlog_latest(handle, &mut sample) }, 0);
        assert_eq!(sample.received_ms, 2000);
        assert_eq!(ainavlog_close(handle), 0);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn stop_cancels_startup_and_closes_the_session() {
        let path = std::env::temp_dir().join(format!(
            "ainavlog-mobile-{}-{}.sqlite3",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let text = CString::new(path.to_str().unwrap()).unwrap();
        let mut handle = 0;
        assert_eq!(
            unsafe { ainavlog_simulator_start(text.as_ptr(), &mut handle) },
            0
        );
        assert_ne!(handle, 0);
        assert_eq!(ainavlog_simulator_stop(), 0);
        assert_eq!(ainavlog_simulator_status(), 0);
        assert_eq!(ainavlog_close(handle), -1);
        assert_eq!(ainavlog_simulator_stop(), 0);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn simulator_identity_filter_excludes_real_and_truncated_records() {
        let packet = include_bytes!("../../tests/fixtures/victron/bmv_starter.bin");
        assert!(simulator_packet(0x02e1, packet));
        for n in 0..24 {
            assert!(!simulator_packet(0x02e1, &packet[..n]));
        }
        assert!(!simulator_packet(0, packet));
        let mut other = *packet;
        other[2] = 0;
        assert!(!simulator_packet(0x02e1, &other));
        other = *packet;
        other[4] = 1;
        assert!(!simulator_packet(0x02e1, &other));
    }
}
