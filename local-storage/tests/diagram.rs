use ainavlog_local_storage::{
    diagram::{Area, Diagram},
    maintenance::Equipment,
    ObservationStore,
};
use rusqlite::Connection;
fn equipment(id: &str) -> Equipment {
    Equipment {
        id: id.into(),
        name: "Battery 1".into(),
        system: "electrical".into(),
        category: "LiFePO₄ battery".into(),
        model: "Example model".into(),
        serial_number: "ABC123".into(),
        make: "Example maker".into(),
        manufactured_date: "2025-03-01".into(),
        installed_date: "2026-01-15".into(),
    }
}
fn diagram() -> Diagram {
    Diagram {
        revision: "revision-1".into(),
        image_id: "image-1".into(),
        image_chunks: 1,
        width: 960,
        height: 720,
        areas: vec![Area {
            equipment_id: "battery-1".into(),
            x: 0.1,
            y: 0.2,
            width: 0.3,
            height: 0.4,
        }],
    }
}
const IMAGE:&str="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/l9sAAAAASUVORK5CYII=";
#[test]
fn migration_roundtrip_reviewed_links_and_history_survive_diagram_replacement() {
    let path =
        std::env::temp_dir().join(format!("diagram-migration-{}.sqlite3", std::process::id()));
    let _ = std::fs::remove_file(&path);
    {
        let store = ObservationStore::open(&path).unwrap();
        drop(store);
        let db = Connection::open(&path).unwrap();
        db.execute_batch("DROP TABLE electrical_diagram; DROP TABLE electrical_diagram_assets; DROP TABLE maintenance_equipment_details; PRAGMA user_version=2; INSERT INTO maintenance_entries VALUES('service','house-battery','2026-11-01',123,'{\"id\":\"service\",\"equipmentId\":\"house-battery\",\"date\":\"2026-11-01\",\"kind\":\"Service\",\"description\":\"Cleaned\",\"provider\":\"\",\"hours\":null,\"parts\":\"\",\"costMinor\":null,\"currency\":\"\",\"reference\":\"\",\"notes\":\"\",\"nextDate\":null,\"nextHours\":null}');").unwrap();
    }
    let mut first = diagram();
    {
        let store = ObservationStore::open(&path).unwrap();
        assert!(store.read_diagram().unwrap().is_none());
        store.put_diagram_chunk("image-1", 0, IMAGE).unwrap();
        store.put_diagram_chunk("image-1", 0, IMAGE).unwrap();
        assert!(store.put_diagram_chunk("image-1", 0, "different").is_err());
        store
            .save_diagram(&first, &[equipment("battery-1")], None)
            .unwrap();
        store
            .save_diagram(&first, &[equipment("battery-1")], None)
            .unwrap();
        assert!(store.discard_diagram_asset("image-1").is_err());
    }
    let store = ObservationStore::open(&path).unwrap();
    assert_eq!(store.read_diagram().unwrap(), Some(first.clone()));
    assert_eq!(store.read_diagram_chunk("image-1", 0).unwrap(), IMAGE);
    let snapshot = store.maintenance_snapshot().unwrap();
    assert_eq!(snapshot.entries.len(), 1);
    assert_eq!(
        snapshot
            .equipment
            .iter()
            .find(|e| e.id == "battery-1")
            .unwrap()
            .serial_number,
        "ABC123"
    );
    let battery = snapshot.equipment.iter().find(|e| e.id == "battery-1").unwrap();
    assert_eq!(battery.make, "Example maker");
    assert_eq!(battery.manufactured_date, "2025-03-01");
    assert_eq!(battery.installed_date, "2026-01-15");
    first.revision = "revision-2".into();
    first.image_id = "image-2".into();
    first.areas.clear();
    store.put_diagram_chunk("image-2", 0, IMAGE).unwrap();
    assert!(store.save_diagram(&first, &[], None).is_err());
    store.save_diagram(&first, &[], Some("revision-1")).unwrap();
    assert!(store.read_diagram_chunk("image-1", 0).is_err());
    let snapshot = store.maintenance_snapshot().unwrap();
    assert_eq!(snapshot.entries.len(), 1);
    assert!(snapshot.equipment.iter().any(|e| e.id == "battery-1"));
    drop(store);
    let _ = std::fs::remove_file(path);
}
#[test]
fn failed_review_rolls_back_new_equipment_and_rejects_missing_chunks_and_foreign_links() {
    let store = ObservationStore::in_memory().unwrap();
    let mut d = diagram();
    assert!(store
        .save_diagram(&d, &[equipment("battery-1")], None)
        .is_err());
    store.put_diagram_chunk("image-1", 0, IMAGE).unwrap();
    d.areas[0].equipment_id = "nonexistent".into();
    assert!(store
        .save_diagram(&d, &[equipment("battery-1")], None)
        .is_err());
    assert!(!store
        .maintenance_snapshot()
        .unwrap()
        .equipment
        .iter()
        .any(|e| e.id == "battery-1"));
    assert!(store.read_diagram().unwrap().is_none());
    d.areas[0].equipment_id = "main-engine".into();
    assert!(store.save_diagram(&d, &[], None).is_err());
    d.areas[0].x = 0.99;
    assert!(d.validate().is_err());
    assert!(store.put_diagram_chunk("../../outside", 0, IMAGE).is_err());
    store.put_diagram_chunk("unused", 0, IMAGE).unwrap();
    store.discard_diagram_asset("unused").unwrap();
    assert!(store.read_diagram_chunk("unused", 0).is_err());
}
