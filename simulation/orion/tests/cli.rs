use ainavlog_device_interface::store::ObservationStore;
use std::{
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn fault_run_persists_only_valid_samples_and_refuses_existing_database() {
    let dir = std::env::temp_dir().join(format!(
        "orion-cli-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&dir).unwrap();
    let db = dir.join("simulation.sqlite3");
    let output = Command::new(env!("CARGO_BIN_EXE_ainavlog-orion-simulation"))
        .args(["--model", "xs", "--interval-ms", "5", "--database"])
        .arg(&db)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    for state in ["Stale", "KeyRequired", "Stopped", "rejected packet"] {
        assert!(text.contains(state), "missing {state}");
    }
    let rows = ObservationStore::read_recent(&db, 1000).unwrap();
    assert_eq!(rows.len(), 15 * 7);
    assert!(rows.iter().all(|r| r.source_id == "SIMULATED-ORION"));
    assert_eq!(rows.iter().filter(|r| r.value.is_none()).count(), 4);
    let retry = Command::new(env!("CARGO_BIN_EXE_ainavlog-orion-simulation"))
        .arg("--database")
        .arg(&db)
        .output()
        .unwrap();
    assert!(!retry.status.success());
    assert_eq!(
        ObservationStore::read_recent(&db, 1000).unwrap().len(),
        rows.len()
    );
    std::fs::remove_dir_all(dir).unwrap();
}
