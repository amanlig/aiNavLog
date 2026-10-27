//! Host-tested C boundary for platform Bluetooth adapters. See include/ainavlog.h.
#[cfg(feature = "mobile")]
mod mobile;
use ainavlog_device_interface::{
    monitor::VictronMonitor, store::ObservationStore, vendor::victron::VictronAdvertisement,
};
use std::{
    collections::HashMap,
    ffi::{c_char, CStr},
    panic::{catch_unwind, AssertUnwindSafe},
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant, UNIX_EPOCH},
};

struct Session {
    monitor: VictronMonitor,
    path: PathBuf,
    source: String,
}
#[derive(Default)]
struct Registry {
    next: u64,
    sessions: HashMap<u64, Session>,
}
static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();

fn boundary(operation: impl FnOnce(&mut Registry) -> Result<i32, i32>) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        let mut registry = REGISTRY
            .get_or_init(|| Mutex::new(Registry::default()))
            .lock()
            .map_err(|_| -3)?;
        operation(&mut registry)
    }))
    .unwrap_or(Err(-3))
    .unwrap_or_else(|code| code)
}

unsafe fn string(pointer: *const c_char) -> Result<String, i32> {
    if pointer.is_null() {
        return Err(-1);
    }
    CStr::from_ptr(pointer)
        .to_str()
        .map(str::to_owned)
        .map_err(|_| -1)
}

/// Opens a session. Keys are never included in error messages or persisted.
/// # Safety
/// Strings must be valid NUL-terminated UTF-8; output must be writable and aligned.
#[no_mangle]
pub unsafe extern "C" fn ainavlog_open(
    path: *const c_char,
    source: *const c_char,
    key: *const c_char,
    output: *mut u64,
) -> i32 {
    if output.is_null() {
        return -1;
    }
    *output = 0;
    boundary(|registry| {
        let path = PathBuf::from(string(path)?);
        let source = string(source)?;
        if source.trim().is_empty() || path.as_os_str().is_empty() {
            return Err(-1);
        }
        if key.is_null() {
            return Err(-1);
        }
        // Borrow the caller's key; do not create an extra non-zeroizing owned copy.
        let key = CStr::from_ptr(key).to_str().map_err(|_| -1)?;
        let store = ObservationStore::open(&path).map_err(|_| -2)?;
        let monitor = VictronMonitor::new(
            source.clone(),
            key,
            Duration::from_secs(5),
            store,
            Instant::now(),
        )
        .map_err(|_| -2)?;
        let handle = registry.next.checked_add(1).ok_or(-3)?;
        registry.next = handle;
        registry.sessions.insert(
            handle,
            Session {
                monitor,
                path,
                source,
            },
        );
        *output = handle;
        Ok(0)
    })
}

/// Invalid or already closed handles return -1.
#[no_mangle]
pub extern "C" fn ainavlog_close(handle: u64) -> i32 {
    boundary(|registry| registry.sessions.remove(&handle).map(|_| 0).ok_or(-1))
}

/// Submit manufacturer data excluding its two-byte company identifier prefix.
/// Returns 0 for persisted data, 1 for ignored data, 2 when a replacement key is needed.
/// # Safety
/// Source must be NUL-terminated UTF-8; payload must contain length readable bytes.
#[no_mangle]
pub unsafe extern "C" fn ainavlog_ingest(
    handle: u64,
    source: *const c_char,
    company: u16,
    payload: *const u8,
    length: usize,
    received_ms: i64,
) -> i32 {
    if payload.is_null() || length == 0 || length > 255 || received_ms < 0 {
        return -1;
    }
    boundary(|registry| {
        let session = registry.sessions.get_mut(&handle).ok_or(-1)?;
        let received_at = UNIX_EPOCH
            .checked_add(Duration::from_millis(received_ms as u64))
            .ok_or(-1)?;
        let report = session
            .monitor
            .ingest(
                VictronAdvertisement {
                    source_id: string(source)?,
                    company_id: company,
                    payload: std::slice::from_raw_parts(payload, length).to_vec(),
                    received_at,
                },
                Instant::now(),
            )
            .map_err(|_| -2)?;
        if session.monitor.state() == ainavlog_device_interface::monitor::MonitorState::KeyRequired
        {
            return Ok(2);
        }
        Ok(if report.is_some_and(|report| report.sample_id.is_some()) {
            0
        } else {
            1
        })
    })
}

/// A presence bit distinguishes unavailable measurements from real zero values.
#[repr(C)]
#[derive(Default)]
pub struct ElectricalSample {
    pub received_ms: i64,
    pub present: u32,
    pub house_voltage: f64,
    pub house_current: f64,
    pub house_charge: f64,
    pub starter_voltage: f64,
}

/// Reads the latest complete BMV sample, including after session restart.
/// Returns 1 if there is no sample. Receipt time is never refreshed by a read.
/// # Safety
/// Output must point to writable, aligned ElectricalSample storage.
#[no_mangle]
pub unsafe extern "C" fn ainavlog_latest(handle: u64, output: *mut ElectricalSample) -> i32 {
    if output.is_null() {
        return -1;
    }
    *output = ElectricalSample::default();
    boundary(|registry| {
        let session = registry.sessions.get(&handle).ok_or(-1)?;
        let rows =
            ObservationStore::read_latest_bmv(&session.path, &session.source).map_err(|_| -2)?;
        let Some(first) = rows.first() else {
            return Ok(1);
        };
        let mut sample = ElectricalSample {
            received_ms: first.received_at_ms,
            ..Default::default()
        };
        for (index, (quantity, unit, slot)) in [
            ("battery_voltage", "V", &mut sample.house_voltage),
            ("battery_current", "A", &mut sample.house_current),
            ("state_of_charge", "%", &mut sample.house_charge),
            ("auxiliary_voltage", "V", &mut sample.starter_voltage),
        ]
        .into_iter()
        .enumerate()
        {
            if let Some(value) = rows
                .iter()
                .find(|row| row.quantity == quantity && row.unit == unit && row.quality == "Good")
                .and_then(|row| row.value)
                .filter(|value| value.is_finite())
            {
                sample.present |= 1 << index;
                *slot = value;
            }
        }
        *output = sample;
        Ok(0)
    })
}
