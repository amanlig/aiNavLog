use ainavlog_device_interface::{
    protocol::{
        nmea0183::{nmea::Hdt, NmeaSentence},
        nmea2000::{ids::field::wind_data as wd, EncodeValue, Frame, Units},
    },
    DeviceError, Nmea0183Codec, Nmea0183Message, Nmea2000Codec, ProtocolDecoder, ProtocolEncoder,
};

#[test]
fn heading_decodes_and_encodes_through_interfaces() {
    let decoder: &mut dyn ProtocolDecoder<str, Message = Nmea0183Message> = &mut Nmea0183Codec;
    let message = decoder.decode("$GPHDT,123.456,T*32\r\n").unwrap();
    assert_eq!(message.frame().talker, "GP");
    assert_eq!(
        message.sentence(),
        &NmeaSentence::Hdt(Hdt {
            heading_true: Some(123.456)
        })
    );
    let encoder: &mut dyn ProtocolEncoder<Nmea0183Message, Output = String> = &mut Nmea0183Codec;
    assert_eq!(encoder.encode(&message).unwrap(), "$GPHDT,123.456,T*32\r\n");
    let outbound = Nmea0183Message::from_sentence(
        "GP",
        &Hdt {
            heading_true: Some(123.456),
        },
    )
    .unwrap();
    let wire = encoder.encode(&outbound).unwrap();
    assert_eq!(
        decoder.decode(&wire).unwrap().sentence(),
        message.sentence()
    );
}

#[test]
fn sentence_errors_and_missing_values_are_explicit() {
    let mut codec = Nmea0183Codec;
    for input in [
        "",
        "$GPHDT,123.456,T*00",
        "$GPHDT,1,T\n$GPHDT,2,T",
        "$GPHDT,é,T",
    ] {
        assert!(matches!(codec.decode(input), Err(DeviceError::Decode(_))));
    }
    let missing = codec.decode("$GPHDT,,T").unwrap();
    assert_eq!(
        missing.sentence(),
        &NmeaSentence::Hdt(Hdt { heading_true: None })
    );
    let malformed = codec.decode("$GPHDT,broken,T").unwrap();
    assert_eq!(malformed.sentence(), missing.sentence());
    assert!(matches!(
        Nmea0183Message::from_sentence(
            "GP,INJECT",
            &Hdt {
                heading_true: Some(1.0)
            }
        ),
        Err(DeviceError::Encode(_))
    ));
    let non_finite = Nmea0183Message::from_sentence(
        "GP",
        &Hdt {
            heading_true: Some(f32::NAN),
        },
    )
    .unwrap();
    assert_eq!(non_finite.sentence(), missing.sentence());
}

#[test]
fn unknown_proprietary_and_ais_envelopes_keep_fields_and_tags() {
    let mut codec = Nmea0183Codec;
    for line in [
        "$GPZZZ,one,,3",
        "$PTEST,1,2",
        "!AIVDM,1,1,,A,payload,0",
        "\\s:bridge\\$GPZZZ,1",
    ] {
        let original = codec.decode(line).unwrap();
        let wire = codec.encode(&original).unwrap();
        assert!(wire.ends_with("\r\n"));
        let again = codec.decode(&wire).unwrap();
        assert_eq!(again.frame(), original.frame());
        assert_eq!(again.sentence(), original.sentence());
    }
}

#[test]
fn wind_encodes_expected_wire_bytes_and_decodes_metadata() {
    let mut codec = Nmea2000Codec::new(Units::Si);
    let mut request = codec
        .message("windData")
        .unwrap()
        .source(42)
        .destination(255)
        .priority(2)
        .timestamp("2026-10-01T00:00:00Z");
    request
        .push(wd::SID, EncodeValue::Int(1))
        .unwrap()
        .push(wd::WIND_SPEED, 5.23)
        .unwrap()
        .push(wd::WIND_ANGLE, 1.5)
        .unwrap()
        .push(wd::REFERENCE, EncodeValue::Lookup("Apparent".into()))
        .unwrap();
    let frame = codec.encode(&request).unwrap();
    assert_eq!(frame.pgn, 130306);
    assert_eq!(&frame.data[..6], &[1, 0x0b, 0x02, 0x98, 0x3a, 0xfa]);
    let decoded = codec.decode(&frame).unwrap();
    assert_eq!((decoded.src, decoded.dst, decoded.prio), (42, 255, 2));
    assert_eq!(decoded.timestamp.as_deref(), Some("2026-10-01T00:00:00Z"));
    assert!(
        (decoded
            .field(wd::WIND_SPEED)
            .unwrap()
            .value
            .as_f64()
            .unwrap()
            - 5.23)
            .abs()
            < 0.001
    );
    assert!(
        (decoded
            .field(wd::WIND_ANGLE)
            .unwrap()
            .as_f64_in("rad")
            .unwrap()
            - 1.5)
            .abs()
            < 0.0001
    );
}

#[test]
fn pgn_errors_are_reported_and_unavailable_is_preserved() {
    let mut codec = Nmea2000Codec::default();
    assert!(matches!(
        codec.message("doesNotExist"),
        Err(DeviceError::Encode(_))
    ));
    let short = Frame::new(None, 2, 130306, 42, 255, [1, 2]);
    assert!(matches!(codec.decode(&short), Err(DeviceError::Decode(_))));
    let request = codec.message("windData").unwrap().priority(8);
    assert!(matches!(
        codec.encode(&request),
        Err(DeviceError::Encode(_))
    ));
    let request = codec.message("windData").unwrap();
    let mut frame = codec.encode(&request).unwrap();
    let decoded = codec.decode(&frame).unwrap();
    assert!(decoded
        .field(wd::WIND_SPEED)
        .unwrap()
        .value
        .is_not_available());
    frame.prio = 8;
    assert!(matches!(codec.decode(&frame), Err(DeviceError::Decode(_))));
    assert!(codec
        .message("windData")
        .unwrap()
        .push_by_name("not a field", 1.0)
        .is_err());
}

#[test]
fn complete_multi_frame_pgn_payload_is_supported() {
    let mut codec = Nmea2000Codec::default();
    let request = codec.message("productInformation").unwrap().source(17);
    let frame = codec.encode(&request).unwrap();
    assert_eq!(frame.pgn, 126996);
    assert!(frame.data.len() > 8);
    let decoded = codec.decode(&frame).unwrap();
    assert_eq!(decoded.id, "productInformation");
    assert_eq!(decoded.src, 17);
    assert_eq!(decoded.data, frame.data.as_slice());
}
