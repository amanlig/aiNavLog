//! Application commands for manually recorded maintenance, independent of BLE.
use ainavlog_local_storage::{
    maintenance::{Equipment, ServiceEntry},
    ObservationStore,
};
use serde_json::{json, Value};
use std::{
    ffi::{c_char, CStr},
    panic::{catch_unwind, AssertUnwindSafe},
    path::Path,
};

pub fn execute(path: &Path, request: &str) -> String {
    let result = (|| -> Result<Value, String> {
        if request.len() > 32768 {
            return Err("Maintenance request too large".into());
        }
        let value: Value =
            serde_json::from_str(request).map_err(|_| "Invalid maintenance request")?;
        let object = value.as_object().ok_or("Invalid maintenance request")?;
        let op = value
            .get("op")
            .and_then(Value::as_str)
            .ok_or("Missing maintenance operation")?;
        if object.keys().any(|k| {
            k != "op"
                && k != if op == "equipment" {
                    "equipment"
                } else {
                    "entry"
                }
        }) {
            return Err("Unexpected maintenance field".into());
        }
        // Validate the entire command before opening/migrating the store.
        let entry = if op == "save" {
            let e: ServiceEntry = serde_json::from_value(value["entry"].clone())
                .map_err(|_| "Invalid service record")?;
            e.validate().map_err(|e| e.to_string())?;
            Some(e)
        } else {
            None
        };
        let equipment = if op == "equipment" {
            let e: Equipment = serde_json::from_value(value["equipment"].clone())
                .map_err(|_| "Invalid equipment")?;
            e.validate().map_err(|e| e.to_string())?;
            Some(e)
        } else {
            None
        };
        if !matches!(op, "list" | "save" | "equipment") {
            return Err("Unknown maintenance operation".into());
        }
        let store =
            ObservationStore::open(path).map_err(|_| "Could not open maintenance storage")?;
        if let Some(e) = entry {
            store
                .save_service(&e)
                .map_err(|_| "Could not save service record. Check equipment and retry.")?;
        }
        if let Some(e) = equipment {
            store
                .save_equipment(&e)
                .map_err(|_| "Could not save equipment")?;
        }
        if op == "list" {
            serde_json::to_value(
                store
                    .maintenance_snapshot()
                    .map_err(|_| "Could not read maintenance history")?,
            )
            .map_err(|e| e.to_string())
        } else {
            Ok(json!({"ok":true}))
        }
    })();
    match result {
        Ok(value) => value.to_string(),
        Err(message) => json!({"error":message}).to_string(),
    }
}

/// Caller-owned UTF-8 buffers; no Rust allocation crosses the ABI.
/// Returns 0 with JSON (including domain errors), -1 invalid args/capacity, -3 panic.
///
/// # Safety
/// Input pointers must reference valid NUL-terminated UTF-8 strings. `output`
/// must be writable for `capacity` bytes and `count` writable for one usize.
/// Buffers must remain valid, non-overlapping, and exclusively owned for the call.
#[no_mangle]
pub unsafe extern "C" fn ainavlog_maintenance(
    path: *const c_char,
    request: *const c_char,
    output: *mut u8,
    capacity: usize,
    count: *mut usize,
) -> i32 {
    if count.is_null() {
        return -1;
    }
    *count = 0;
    if path.is_null() || request.is_null() || output.is_null() {
        return -1;
    }
    catch_unwind(AssertUnwindSafe(|| {
        let Ok(path) = CStr::from_ptr(path).to_str() else {
            return -1;
        };
        let Ok(request) = CStr::from_ptr(request).to_str() else {
            return -1;
        };
        if path.trim().is_empty() {
            return -1;
        }
        let response = execute(Path::new(path), request);
        if response.len() > capacity {
            return -1;
        }
        std::ptr::copy_nonoverlapping(response.as_ptr(), output, response.len());
        *count = response.len();
        0
    }))
    .unwrap_or(-3)
}
