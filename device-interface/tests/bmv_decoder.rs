use ainavlog_device_interface::{
    monitor::{normalize_victron, VictronMonitor},
    store::ObservationStore,
    vendor::victron::{
        BatteryAuxiliary, VictronAdvertisement, VictronAdvertisementDecoder, VictronRecord,
    },
    ProtocolDecoder,
};
use std::time::{Duration, Instant, SystemTime};
const KEY: &str = "000102030405060708090a0b0c0d0e0f";
fn hex(s: &str) -> Vec<u8> {
    let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn packet(payload: Vec<u8>) -> VictronAdvertisement {
    VictronAdvertisement {
        source_id: "bmv-test".into(),
        received_at: SystemTime::now(),
        company_id: 0x02e1,
        payload,
    }
}
#[test]
fn iphone_packet_and_independent_charge_discharge_fixtures() {
    for (raw, counter, voltage, current, soc) in [
        (
            "1002 3412 0274 0000 7639 9A0B CF2F E420 6FE7 00BB 4D06 36A8",
            116,
            14.05,
            12.0,
            77.4,
        ),
        (
            include_str!("../../simulation/bmv712/fixture.hex"),
            0x1234,
            12.76,
            -5.25,
            77.5,
        ),
        (
            include_str!("../../simulation/bmv712/charging-fixture.hex"),
            0x1234,
            14.0,
            12.0,
            76.3,
        ),
    ] {
        let message = VictronAdvertisementDecoder::new("bmv-test", KEY)
            .unwrap()
            .decode(&packet(hex(raw)))
            .unwrap();
        assert_eq!(message.data_counter, counter);
        assert_eq!(message.product_id, 0x1234);
        let VictronRecord::BatteryMonitor(d) = &message.record else {
            panic!("expected battery record")
        };
        for (actual, expected) in [
            (d.battery_voltage_v, voltage),
            (d.battery_current_a, current),
            (d.state_of_charge_percent, soc),
        ] {
            assert!((actual.unwrap() - expected).abs() < 1e-9);
        }
        assert_eq!(d.auxiliary, BatteryAuxiliary::Disabled);
        assert_eq!(d.alarm_reason, 0);
        assert_eq!(normalize_victron(&message).unwrap().len(), 6);
    }
}
#[test]
fn missing_values_and_starter_voltage_are_distinct() {
    let message = VictronAdvertisementDecoder::new("bmv-test", KEY)
        .unwrap()
        .decode(&packet(
            include_bytes!("fixtures/victron/bmv_na.bin").to_vec(),
        ))
        .unwrap();
    let VictronRecord::BatteryMonitor(d) = &message.record else {
        panic!()
    };
    assert_eq!(
        (
            d.battery_voltage_v,
            d.battery_current_a,
            d.consumed_ah,
            d.state_of_charge_percent
        ),
        (None, None, None, None)
    );
    assert_eq!(d.time_to_go_minutes, None);
    let input = packet(include_bytes!("fixtures/victron/bmv_starter.bin").to_vec());
    let message = VictronAdvertisementDecoder::new("bmv-test", KEY)
        .unwrap()
        .decode(&input)
        .unwrap();
    let VictronRecord::BatteryMonitor(d) = message.record else {
        panic!()
    };
    assert_eq!(d.auxiliary, BatteryAuxiliary::Voltage(Some(12.8)));
    let now = Instant::now();
    let mut monitor = VictronMonitor::new(
        "bmv-test".into(),
        KEY,
        Duration::from_secs(3),
        ObservationStore::in_memory().unwrap(),
        now,
    )
    .unwrap();
    let report = monitor.ingest(input.clone(), now).unwrap().unwrap();
    assert!(report
        .observations
        .iter()
        .any(|o| o.quantity == "auxiliary_voltage" && o.value == Some(12.8)));
    assert_eq!(monitor.store().recent(100).unwrap().len(), 7);
    assert!(monitor.ingest(input, now).unwrap().is_none());
}
#[test]
fn truncation_rejected_and_extensions_accepted() {
    let raw = hex(include_str!("../../simulation/bmv712/fixture.hex"));
    for length in 0..23 {
        assert!(VictronAdvertisementDecoder::new("bmv-test", KEY)
            .unwrap()
            .decode(&packet(raw[..length].to_vec()))
            .is_err());
    }
    // The 15th plaintext byte finishes SOC; a 16th reserved byte is optional.
    for length in [23, 24] {
        assert!(VictronAdvertisementDecoder::new("bmv-test", KEY)
            .unwrap()
            .decode(&packet(raw[..length].to_vec()))
            .is_ok());
    }
    let mut extended = raw;
    extended.extend([0, 1, 2]);
    assert!(VictronAdvertisementDecoder::new("bmv-test", KEY)
        .unwrap()
        .decode(&packet(extended))
        .is_ok());
}

#[test]
fn ui_reads_one_complete_packet_for_selected_bmv() {
    let path = std::env::temp_dir().join(format!(
        "ainavlog-ui-{}-{}.sqlite3",
        std::process::id(),
        SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut store = ObservationStore::open(&path).unwrap();
    let mut decoder = VictronAdvertisementDecoder::new("bmv-test", KEY).unwrap();
    for raw in [
        include_bytes!("fixtures/victron/bmv_starter.bin").as_slice(),
        include_bytes!("fixtures/victron/bmv_na.bin").as_slice(),
    ] {
        let frame = packet(raw.to_vec());
        let message = decoder.decode(&frame).unwrap();
        store
            .save(&message, raw, &normalize_victron(&message).unwrap())
            .unwrap();
        let rows = ObservationStore::read_latest_bmv(&path, "bmv-test").unwrap();
        assert!(!rows.is_empty());
        assert!(rows.iter().all(|o| o.sample_id == rows[0].sample_id));
        assert_eq!(
            rows[0].received_at_ms,
            ainavlog_device_interface::store::unix_millis(frame.received_at).unwrap()
        );
    }
    let rows = ObservationStore::read_latest_bmv(&path, "bmv-test").unwrap();
    assert!(rows
        .iter()
        .find(|o| o.quantity == "battery_voltage")
        .unwrap()
        .value
        .is_none());
    assert!(!rows.iter().any(|o| o.quantity == "auxiliary_voltage"));
    assert!(ObservationStore::read_latest_bmv(&path, "another-device")
        .unwrap()
        .is_empty());
    drop(store);
    std::fs::remove_file(path).unwrap();
}
