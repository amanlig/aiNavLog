use ainavlog_device_interface::{
    vendor::victron::{
        OrionRecord, VictronAdvertisement, VictronAdvertisementDecoder, VictronError,
        VictronMessage, VICTRON_COMPANY_ID,
    },
    DeviceError, ProtocolDecoder,
};
use std::time::{Duration, SystemTime};

// Public synthetic key, never a real device credential. Ciphertexts were made
// independently with OpenSSL, not with the implementation under test.
const KEY: &str = "000102030405060708090a0b0c0d0e0f";
const TR: &[u8] = include_bytes!("fixtures/victron/orion_tr.bin");
const XS: &[u8] = include_bytes!("fixtures/victron/orion_xs.bin");

fn advertisement(bytes: &[u8]) -> VictronAdvertisement {
    VictronAdvertisement {
        source_id: "test-orion".into(),
        received_at: SystemTime::UNIX_EPOCH + Duration::from_secs(123),
        company_id: VICTRON_COMPANY_ID,
        payload: bytes.to_vec(),
    }
}
fn decoder() -> VictronAdvertisementDecoder {
    VictronAdvertisementDecoder::new("test-orion", KEY).unwrap()
}
fn close(actual: Option<f64>, expected: f64) {
    assert!((actual.unwrap() - expected).abs() < 1e-9);
}

#[test]
fn published_dcdc_layout_decrypts_through_shared_interface() {
    let mut decoder = decoder();
    let api: &mut dyn ProtocolDecoder<VictronAdvertisement, Message = VictronMessage> =
        &mut decoder;
    let input = advertisement(TR);
    let message = api.decode(&input).unwrap();
    assert_eq!(message.source_id, input.source_id);
    assert_eq!(message.received_at, input.received_at);
    assert_eq!(message.product_id, 0x1234);
    assert_eq!(message.data_counter, 0x1234);
    let OrionRecord::DcDc(data) = message.record else {
        panic!("wrong record")
    };
    assert_eq!(data.device_state, Some(3));
    assert_eq!(data.charger_error, Some(0));
    close(data.input_voltage_v, 13.2);
    close(data.output_voltage_v, 14.4);
    assert_eq!(data.off_reason, 0x80000081); // preserve multiple and unknown bits
}

#[test]
fn iphone_reported_windows_simulator_packet_decrypts() {
    // User transcription from nRF Connect, 2026-10-05:
    // 02E1 1002 3412 0493 0000 A54A A468 4B55 B425 8E1B
    // 02E1 is the displayed company ID, not part of the encrypted payload.
    // Keep B425 exactly as supplied, rather than replacing it with B42B.
    let payload = [
        0x10, 0x02, 0x34, 0x12, 0x04, 0x93, 0x00, 0x00, 0xa5, 0x4a, 0xa4, 0x68, 0x4b, 0x55, 0xb4,
        0x25, 0x8e, 0x1b,
    ];
    let input = advertisement(&payload);
    let message = decoder().decode(&input).unwrap();
    assert_eq!(message.source_id, input.source_id);
    assert_eq!(message.received_at, input.received_at);
    assert_eq!(message.product_id, 0x1234);
    assert_eq!(message.data_counter, 147);
    let OrionRecord::DcDc(data) = message.record else {
        panic!("expected Orion TR record")
    };
    assert_eq!(data.device_state, Some(3)); // Bulk
    assert_eq!(data.charger_error, Some(0));
    close(data.input_voltage_v, 13.26);
    close(data.output_voltage_v, 14.4);
    // Independently decrypted with OpenSSL: 03 00 2e 05 a0 05 00 0e 00 00.
    assert_eq!(data.off_reason, 0x00000e00);
}

#[test]
fn xs_uses_its_own_field_order_signed_current_and_scaling() {
    let message = decoder().decode(&advertisement(XS)).unwrap();
    let OrionRecord::OrionXs(data) = message.record else {
        panic!("wrong record")
    };
    assert_eq!((data.device_state, data.charger_error), (3, 0));
    close(data.input_voltage_v, 13.2);
    close(data.output_voltage_v, 14.4);
    close(data.output_current_a, -12.3);
    close(data.input_current_a, 15.0);
    assert_eq!(data.off_reason, 0x80000081);
}

#[test]
fn unavailable_measurements_and_signed_voltages_are_preserved() {
    let message = decoder()
        .decode(&advertisement(include_bytes!(
            "fixtures/victron/orion_tr_na.bin"
        )))
        .unwrap();
    let OrionRecord::DcDc(data) = message.record else {
        panic!("wrong record")
    };
    assert_eq!(data.device_state, None);
    assert_eq!(data.charger_error, None);
    assert_eq!(data.input_voltage_v, None);
    assert_eq!(data.output_voltage_v, None);
    assert_eq!(data.off_reason, u32::MAX);
    let message = decoder()
        .decode(&advertisement(include_bytes!(
            "fixtures/victron/orion_xs_na.bin"
        )))
        .unwrap();
    let OrionRecord::OrionXs(data) = message.record else {
        panic!("wrong record")
    };
    assert_eq!((data.device_state, data.charger_error), (255, 255));
    assert_eq!(data.input_voltage_v, None);
    assert_eq!(data.output_voltage_v, None);
    assert_eq!(data.input_current_a, None);
    assert_eq!(data.output_current_a, None);
    let message = decoder()
        .decode(&advertisement(include_bytes!(
            "fixtures/victron/orion_tr_negative.bin"
        )))
        .unwrap();
    let OrionRecord::DcDc(data) = message.record else {
        panic!("wrong record")
    };
    assert_eq!(
        (data.device_state, data.charger_error),
        (Some(254), Some(254))
    );
    close(data.input_voltage_v, 0.0);
    close(data.output_voltage_v, -1.25);
}

#[test]
fn every_truncated_prefix_is_rejected_without_panicking() {
    for bytes in [TR, XS] {
        for length in 0..bytes.len() {
            assert!(
                matches!(
                    decoder().decode_advertisement(&advertisement(&bytes[..length])),
                    Err(VictronError::Truncated { .. })
                ),
                "length {length}"
            );
        }
    }
}

#[test]
fn wrong_source_company_and_record_are_rejected_before_key_handling() {
    let mut decoder = decoder();
    let mut input = advertisement(TR);
    input.source_id = "another-device".into();
    input.payload[7] = 1;
    assert_eq!(
        decoder.decode_advertisement(&input),
        Err(VictronError::WrongSource)
    );
    input = advertisement(TR);
    input.company_id = 0;
    assert_eq!(
        decoder.decode_advertisement(&input),
        Err(VictronError::WrongCompany)
    );
    input = advertisement(TR);
    input.payload[0] = 0x01;
    assert_eq!(
        decoder.decode_advertisement(&input),
        Err(VictronError::UnsupportedAdvertisement(1))
    );
    input = advertisement(TR);
    input.payload[4] = 0x01;
    assert_eq!(
        decoder.decode_advertisement(&input),
        Err(VictronError::UnsupportedRecord(1))
    );
    assert_eq!(
        decoder.decode(&input),
        Err(DeviceError::UnsupportedProtocol)
    );
    assert!(decoder.decode(&advertisement(TR)).is_ok());
}

#[test]
fn key_mismatch_disables_decoding_until_explicit_replacement() {
    let mut decoder = decoder();
    let mut bad = advertisement(TR);
    bad.payload[7] = 1;
    assert_eq!(
        decoder.decode_advertisement(&bad),
        Err(VictronError::KeyMismatch)
    );
    assert_eq!(
        decoder.decode_advertisement(&advertisement(TR)),
        Err(VictronError::KeyRequired)
    );
    decoder.replace_key(KEY).unwrap();
    assert!(decoder.decode(&advertisement(TR)).is_ok());
}

#[test]
fn keys_are_strictly_parsed_and_never_formatted() {
    for key in [
        "",
        "00",
        "000102030405060708090a0b0c0d0e0g",
        "000102030405060708090a0b0c0d0e0f ",
        "é000102030405060708090a0b0c0d0e",
    ] {
        let error = VictronAdvertisementDecoder::new("test-orion", key).unwrap_err();
        assert_eq!(error, VictronError::InvalidKey);
    }
    assert!(!format!("{:?}", decoder()).contains(KEY));
    let mut decoder = VictronAdvertisementDecoder::new("test-orion", &KEY.to_uppercase()).unwrap();
    assert!(decoder.replace_key("invalid").is_err());
    assert!(decoder.decode(&advertisement(TR)).is_ok());
}

#[test]
fn extensions_and_duplicate_advertisements_are_allowed() {
    let mut decoder = decoder();
    let ordinary = decoder.decode(&advertisement(TR)).unwrap();
    let extended = decoder
        .decode(&advertisement(include_bytes!(
            "fixtures/victron/orion_tr_extended.bin"
        )))
        .unwrap();
    assert_eq!(ordinary, extended);
    assert_eq!(ordinary, decoder.decode(&advertisement(TR)).unwrap());
}
