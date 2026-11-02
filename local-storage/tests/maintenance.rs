use ainavlog_local_storage::{
    maintenance::{valid_date, Equipment, ServiceEntry},
    ObservationStore,
};
use rusqlite::Connection;
fn entry() -> ServiceEntry {
    ServiceEntry {
        id: "work-1".into(),
        equipment_id: "alternator".into(),
        system: Some("electrical".into()),
        date: "2026-10-30".into(),
        kind: "Inspection".into(),
        description: "Checked wiring 🛥️ and belt".into(),
        provider: "Owner".into(),
        hours: Some(312.5),
        parts: "".into(),
        cost_minor: Some(12345),
        currency: "CAD".into(),
        reference: "Survey A-3".into(),
        notes: "No corrosion".into(),
        next_date: Some("2027-04-30".into()),
        next_hours: Some(500.0),
    }
}
#[test]
fn roundtrip_idempotency_equipment_integrity_and_validation() {
    let store = ObservationStore::in_memory().unwrap();
    let mut e = entry();
    store.save_service(&e).unwrap();
    store.save_service(&e).unwrap();
    let result = store.maintenance_snapshot().unwrap();
    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].entry, e);
    assert!(result.entries[0].created_at > 0);
    assert_eq!(
        result
            .equipment
            .iter()
            .find(|v| v.id == "alternator")
            .unwrap()
            .system,
        "both"
    );
    e.description = "Different work".into();
    assert!(store.save_service(&e).is_err());
    e.id = "missing".into();
    e.equipment_id = "nonexistent".into();
    assert!(store.save_service(&e).is_err());
    e = entry();
    e.id = "invalid".into();
    e.hours = Some(f64::NAN);
    assert!(store.save_service(&e).is_err());
    e = entry();
    e.next_hours = Some(100.0);
    assert!(e.validate().is_err());
    e = entry();
    e.cost_minor = Some(-1);
    assert!(e.validate().is_err());
    assert!(valid_date("2028-02-29"));
    assert!(!valid_date("2026-02-29"));
    assert!(!valid_date("2026-13-01"));
    assert!(!valid_date("é026-10-30"));
    let custom = Equipment {
        id: "custom".into(),
        name: "Port engine".into(),
        system: "engine".into(),
    };
    store.save_equipment(&custom).unwrap();
    store.save_equipment(&custom).unwrap();
    let mut changed = custom.clone();
    changed.name = "Another".into();
    assert!(store.save_equipment(&changed).is_err());
}
#[test]
fn migrates_v1_preserves_telemetry_and_reopens_service_history() {
    let path = std::env::temp_dir().join(format!(
        "maintenance-migrate-{}.sqlite3",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    {
        let store = ObservationStore::open(&path).unwrap();
        drop(store);
        let db = Connection::open(&path).unwrap();
        db.execute_batch("DROP TABLE maintenance_entries; DROP TABLE maintenance_equipment; PRAGMA user_version=1; INSERT INTO samples VALUES(1,'preserve',123,4660,1,2,x'0001'); INSERT INTO observations VALUES(1,'battery_voltage','V',12.76,'Good',NULL);").unwrap();
    }
    {
        let store = ObservationStore::open(&path).unwrap();
        assert_eq!(store.recent(10).unwrap()[0].value, Some(12.76));
        store.save_service(&entry()).unwrap();
    }
    {
        let store = ObservationStore::open(&path).unwrap();
        assert_eq!(
            store.maintenance_snapshot().unwrap().entries[0].entry,
            entry()
        );
    }
    let db = Connection::open(&path).unwrap();
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        2
    );
    db.execute_batch("PRAGMA user_version=999").unwrap();
    drop(db);
    assert!(ObservationStore::open(&path).is_err());
    let _ = std::fs::remove_file(path);
}

#[test]
fn separates_records_on_shared_equipment_without_reclassifying_legacy_work() {
    let store = ObservationStore::in_memory().unwrap();
    let electrical = entry();
    store.save_service(&electrical).unwrap();
    let mut engine = entry();
    engine.id = "engine-work".into();
    engine.system = Some("engine".into());
    store.save_service(&engine).unwrap();
    let mut legacy = entry();
    legacy.id = "legacy-work".into();
    legacy.system = None;
    store.save_service(&legacy).unwrap();
    let result = store.maintenance_snapshot().unwrap();
    assert_eq!(result.entries.len(), 3);
    assert_eq!(
        result
            .entries
            .iter()
            .filter(|e| e.entry.system.as_deref() == Some("electrical"))
            .count(),
        1
    );
    assert_eq!(
        result
            .entries
            .iter()
            .filter(|e| e.entry.system.as_deref() == Some("engine"))
            .count(),
        1
    );
    assert_eq!(
        result
            .entries
            .iter()
            .filter(|e| e.entry.system.is_none())
            .count(),
        1
    );
    let mut invalid = entry();
    invalid.id = "wrong-log".into();
    invalid.equipment_id = "house-battery".into();
    invalid.system = Some("engine".into());
    assert!(store.save_service(&invalid).is_err());
    invalid.system = Some("all".into());
    assert!(invalid.validate().is_err());
}
