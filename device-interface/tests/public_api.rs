//! Public API contract tests for the current stub phase.
//! Replace these expectations with behavior tests as modules are implemented.

use ainavlog_device_interface::*;
use std::fmt::Debug;
use std::time::{Duration, SystemTime};

fn device(transport: TransportKind) -> DeviceDescriptor {
    DeviceDescriptor {
        id: "test-instrument".to_owned(),
        name: None,
        transport,
    }
}

fn assert_not_implemented<T: Debug>(result: DeviceResult<T>) {
    assert!(
        matches!(&result, Err(DeviceError::NotImplemented)),
        "Expected an explicit unimplemented error, got {result:?}"
    );
}

#[test]
fn connection_manager_does_not_claim_connection_or_status() {
    let mut manager = ConnectionManager::new();
    assert_not_implemented(manager.set_reconnect_policy(ReconnectPolicy {
        max_attempts: 0,
        retry_delay: Duration::ZERO,
    }));
    assert_not_implemented(manager.connect(&device(TransportKind::Bluetooth)));
    assert_not_implemented(manager.status());
    assert_not_implemented(manager.disconnect());
}

fn check_transport(adapter: &mut dyn TransportAdapter, device: &DeviceDescriptor) {
    assert_not_implemented(adapter.discover());
    assert_not_implemented(adapter.connect(device));
    assert_not_implemented(adapter.try_receive());
    assert_not_implemented(adapter.disconnect());
}

#[tokio::test]
async fn bluetooth_rejects_invalid_requests_without_hardware() {
    let mut adapter = BluetoothAdapter::default();
    assert!(adapter.discover(Duration::ZERO).await.is_err());
    assert!(adapter
        .connect(&device(TransportKind::Bluetooth))
        .await
        .is_err());
    assert!(adapter.connect_by_id("not-a-device-address").await.is_err());
    assert_eq!(adapter.disconnect().await, Ok(()));
}

struct StubTransport;
impl TransportAdapter for StubTransport {}

#[test]
fn default_transport_does_not_claim_hardware_access() {
    check_transport(&mut StubTransport, &device(TransportKind::Bluetooth));
}

struct StubProtocol;
impl DeviceProtocolAdapter for StubProtocol {}

#[test]
fn default_protocol_does_not_claim_support_or_invent_readings() {
    let mut protocol = StubProtocol;
    assert_not_implemented(protocol.supports(&device(TransportKind::Bluetooth)));
    assert_not_implemented(protocol.decode(&ReceivedFrame {
        source_id: "test-instrument".to_owned(),
        received_at: SystemTime::UNIX_EPOCH,
        payload: vec![0x00, 0xff],
    }));
}

#[test]
fn normalizer_preserves_missing_measurements() {
    let normalizer = ObservationNormalizer::new();
    let observation = normalizer
        .normalize(&DecodedReading {
            source_id: "test-instrument".to_owned(),
            observed_at: None,
            received_at: SystemTime::UNIX_EPOCH,
            quantity: "voltage".to_owned(),
            unit: "V".to_owned(),
            value: None,
            quality: QualityStatus::Unknown,
        })
        .unwrap();
    assert_eq!(observation.value, None);
    assert_eq!(observation.quality, QualityStatus::Unknown);
}
