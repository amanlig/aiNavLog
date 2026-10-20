//! Async BLE central adapter. Call from a Tokio runtime.
use crate::types::*;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use std::{collections::BTreeMap, time::Duration};
use tokio::time::{sleep, timeout};

const OPERATION_TIMEOUT: Duration = Duration::from_secs(20);

fn transport(error: impl std::fmt::Display) -> DeviceError {
    DeviceError::Transport(error.to_string())
}

/// One selected OS adapter and at most one connection owned by this instance.
/// Discovery can include OS-cached devices; it does not guarantee availability.
#[derive(Default)]
pub struct BluetoothAdapter {
    adapter: Option<Adapter>,
    discovered: BTreeMap<String, Peripheral>,
    connected: Option<Peripheral>,
}

impl BluetoothAdapter {
    /// Initialize a particular adapter (zero-based). No hardware is accessed by Default.
    pub async fn new(index: usize) -> DeviceResult<Self> {
        let manager = Manager::new().await.map_err(transport)?;
        let adapters = manager.adapters().await.map_err(transport)?;
        let adapter = adapters.into_iter().nth(index).ok_or_else(|| {
            transport(format!("Bluetooth adapter {index} unavailable; check Bluetooth power, permissions, and OS Bluetooth service"))
        })?;
        Ok(Self {
            adapter: Some(adapter),
            ..Self::default()
        })
    }

    /// Scan for the specified duration and stop scanning before returning results.
    pub async fn discover(&mut self, duration: Duration) -> DeviceResult<Vec<DeviceDescriptor>> {
        if duration.is_zero() {
            return Err(transport("Scan duration must be greater than zero"));
        }
        if self.adapter.is_none() {
            self.adapter = Self::new(0).await?.adapter;
        }
        let adapter = self.adapter.as_ref().unwrap();
        adapter
            .start_scan(ScanFilter::default())
            .await
            .map_err(transport)?;
        sleep(duration).await;
        adapter.stop_scan().await.map_err(transport)?;
        let peripherals = adapter.peripherals().await.map_err(transport)?;
        self.discovered.clear();
        let mut devices = Vec::new();
        for peripheral in peripherals {
            let id = peripheral.id().to_string();
            let name = peripheral
                .properties()
                .await
                .map_err(transport)?
                .and_then(|properties| properties.local_name);
            devices.push(DeviceDescriptor {
                id: id.clone(),
                name,
                transport: TransportKind::Bluetooth,
            });
            self.discovered.insert(id, peripheral);
        }
        devices.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(devices)
    }

    /// Connect to a selected ID. Windows can also attempt a known Bluetooth address
    /// that did not advertise during this scan. This does not prove reachability.
    pub async fn connect_by_id(&mut self, id: &str) -> DeviceResult<()> {
        let known = self
            .discovered
            .keys()
            .find(|key| key.eq_ignore_ascii_case(id))
            .cloned();
        let id = match known {
            Some(id) => id,
            None => self.register_known_address(id).await?,
        };
        self.connect(&DeviceDescriptor {
            id,
            name: None,
            transport: TransportKind::Bluetooth,
        })
        .await
    }

    #[cfg(target_os = "windows")]
    async fn register_known_address(&mut self, id: &str) -> DeviceResult<String> {
        let address = id.parse::<btleplug::api::BDAddr>().map_err(|_| {
            transport("Expected a Windows Bluetooth address such as AA:BB:CC:DD:EE:FF")
        })?;
        if self.adapter.is_none() {
            self.adapter = Self::new(0).await?.adapter;
        }
        let peripheral = self
            .adapter
            .as_ref()
            .unwrap()
            .add_peripheral(&address.into())
            .await
            .map_err(transport)?;
        let id = peripheral.id().to_string();
        self.discovered.insert(id.clone(), peripheral);
        Ok(id)
    }

    #[cfg(not(target_os = "windows"))]
    async fn register_known_address(&mut self, _id: &str) -> DeviceResult<String> {
        Err(transport("Device not found in scan; enable advertising and retry. Connecting to an unscanned address is currently supported only on Windows"))
    }

    /// Connect only to an explicitly selected device returned by discovery.
    /// No characteristics are written or subscribed to.
    pub async fn connect(&mut self, device: &DeviceDescriptor) -> DeviceResult<()> {
        if self.connected.is_some() {
            return Err(transport(
                "Disconnect the current device before connecting again",
            ));
        }
        let peripheral = self
            .discovered
            .get(&device.id)
            .cloned()
            .ok_or_else(|| transport("Device not discovered; scan before connecting"))?;
        if peripheral.is_connected().await.map_err(transport)? {
            return Err(transport(
                "Device is already connected; refusing to take ownership of an existing connection",
            ));
        }
        // Retain the handle even on failure so callers can clean up a partial connection.
        self.connected = Some(peripheral.clone());
        timeout(OPERATION_TIMEOUT, peripheral.connect())
            .await
            .map_err(|_| transport(format!("Connection to {} timed out after {} seconds. Check that the device is awake, in range, and accepting BLE connections", device.id, OPERATION_TIMEOUT.as_secs())))?
            .map_err(|error| transport(format!("Connection to {} failed: {error}. Discovery alone does not establish BLE/GATT compatibility; check the device's connection mode and whether another app is using it", device.id)))?;
        if !peripheral.is_connected().await.map_err(transport)? {
            return Err(DeviceError::Disconnected);
        }
        Ok(())
    }

    pub async fn disconnect(&mut self) -> DeviceResult<()> {
        if let Some(peripheral) = &self.connected {
            timeout(OPERATION_TIMEOUT, peripheral.disconnect())
                .await
                .map_err(|_| transport("BLE disconnection timed out"))?
                .map_err(transport)?;
            self.connected = None;
        }
        Ok(())
    }
}

/// An active scan with an event subscription established before scanning starts.
/// Explicitly call stop; Drop attempts cleanup if a Tokio runtime is available.
pub struct AdvertisementScan {
    adapter: Adapter,
    events: std::pin::Pin<Box<dyn futures_util::Stream<Item = btleplug::api::CentralEvent> + Send>>,
    stopped: bool,
}

impl BluetoothAdapter {
    pub async fn start_advertisements(&mut self) -> DeviceResult<AdvertisementScan> {
        if self.adapter.is_none() {
            self.adapter = Self::new(0).await?.adapter;
        }
        let adapter = self.adapter.as_ref().unwrap().clone();
        let events = timeout(OPERATION_TIMEOUT, adapter.events())
            .await
            .map_err(|_| transport("BLE event subscription timed out"))?
            .map_err(transport)?;
        // Construct the cleanup guard before start_scan so cancellation/failure
        // also attempts to stop a partially started OS scan.
        let mut scan = AdvertisementScan {
            adapter,
            events,
            stopped: false,
        };
        let started = timeout(
            OPERATION_TIMEOUT,
            scan.adapter.start_scan(ScanFilter::default()),
        )
        .await;
        match started {
            Ok(Ok(())) => Ok(scan),
            result => {
                let _ = scan.stop().await;
                match result {
                    Ok(Err(error)) => Err(transport(error)),
                    _ => Err(transport("BLE scan startup timed out")),
                }
            }
        }
    }
}
impl AdvertisementScan {
    pub async fn next_event(&mut self) -> DeviceResult<btleplug::api::CentralEvent> {
        use futures_util::StreamExt;
        self.events.next().await.ok_or(DeviceError::Disconnected)
    }
    pub async fn stop(&mut self) -> DeviceResult<()> {
        if !self.stopped {
            timeout(OPERATION_TIMEOUT, self.adapter.stop_scan())
                .await
                .map_err(|_| transport("BLE scan shutdown timed out"))?
                .map_err(transport)?;
            self.stopped = true;
        }
        Ok(())
    }
}
impl Drop for AdvertisementScan {
    fn drop(&mut self) {
        if !self.stopped {
            if let Ok(runtime) = tokio::runtime::Handle::try_current() {
                let adapter = self.adapter.clone();
                runtime.spawn(async move {
                    let _ = timeout(OPERATION_TIMEOUT, adapter.stop_scan()).await;
                });
            }
        }
    }
}
