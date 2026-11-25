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

#[test]
fn scoped_diagram_commands_keep_multiple_engine_views_and_electrical_separate() {
    let path =
        std::env::temp_dir().join(format!("maintenance-views-{}.sqlite3", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let call = |request: Value| -> Value {
        serde_json::from_str(&execute(&path, &request.to_string())).unwrap()
    };
    let image="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/l9sAAAAASUVORK5CYII=";
    assert_eq!(
        call(json!({"op":"diagram_put","imageId":"engine-image","index":0,"body":image}))["ok"],
        true
    );
    for (id, name) in [
        ("upper", "Upper view"),
        ("port", "Port side view"),
        ("starboard", "Starboard side view"),
        ("bottom", "Bottom view"),
    ] {
        let request = json!({"op":"diagram_save","system":"engine","diagramId":id,"name":name,"baseRevision":null,"equipment":[],"diagram":{"revision":id,"imageId":"engine-image","imageChunks":1,"width":640,"height":480,"areas":[{"equipmentId":"main-engine","x":0.1,"y":0.1,"width":0.4,"height":0.4}]}});
        assert_eq!(call(request.clone())["ok"], true);
        assert_eq!(call(request)["ok"], true);
        assert_eq!(
            call(json!({"op":"diagram_read","system":"engine","diagramId":id}))["diagram"]
                ["revision"],
            id
        );
    }
    assert_eq!(
        call(json!({"op":"diagram_list","system":"engine"}))["diagrams"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(
        call(json!({"op":"diagram_list","system":"electrical"}))["diagrams"],
        json!([])
    );
    assert!(call(json!({"op":"diagram_read"}))["diagram"].is_null());
    assert!(call(json!({"op":"diagram_discard","imageId":"engine-image"}))["error"].is_string());
    assert!(call(json!({"op":"diagram_list","system":"both"}))["error"].is_string());
    let _ = std::fs::remove_file(path);
}
