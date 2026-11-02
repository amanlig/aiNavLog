use ainavlog_native_bridge::maintenance::{ainavlog_maintenance, execute};
use serde_json::{json, Value};
use std::ffi::CString;
#[test]
fn commands_and_abi_work_disconnected_and_retry_without_duplicates() {
    let path = std::env::temp_dir().join(format!("maintenance-abi-{}.sqlite3", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let read: Value = serde_json::from_str(&execute(&path, r#"{"op":"list"}"#)).unwrap();
    assert_eq!(read["entries"], json!([]));
    let command=json!({"op":"save","entry":{"id":"once","equipmentId":"house-battery","date":"2026-10-30","kind":"Service","description":"Cleaned terminals 🛥️","provider":"Owner","hours":null,"parts":"","costMinor":0,"currency":"USD","reference":"","notes":"","nextDate":null,"nextHours":null}}).to_string();
    assert_eq!(
        serde_json::from_str::<Value>(&execute(&path, &command)).unwrap()["ok"],
        true
    );
    assert_eq!(
        serde_json::from_str::<Value>(&execute(&path, &command)).unwrap()["ok"],
        true
    );
    let file = CString::new(path.to_str().unwrap()).unwrap();
    let request = CString::new(r#"{"op":"list"}"#).unwrap();
    let mut output = vec![0u8; 32768];
    let mut count = 77;
    assert_eq!(
        unsafe {
            ainavlog_maintenance(
                file.as_ptr(),
                request.as_ptr(),
                output.as_mut_ptr(),
                output.len(),
                &mut count,
            )
        },
        0
    );
    let data: Value = serde_json::from_slice(&output[..count]).unwrap();
    assert_eq!(data["entries"].as_array().unwrap().len(), 1);
    assert_eq!(data["entries"][0]["description"], "Cleaned terminals 🛥️");
    assert_eq!(
        unsafe {
            ainavlog_maintenance(
                file.as_ptr(),
                request.as_ptr(),
                output.as_mut_ptr(),
                1,
                &mut count,
            )
        },
        -1
    );
    assert_eq!(count, 0);
    assert!(
        serde_json::from_str::<Value>(&execute(&path, r#"{"op":"bad"}"#)).unwrap()["error"]
            .is_string()
    );
    let _ = std::fs::remove_file(path);
}
