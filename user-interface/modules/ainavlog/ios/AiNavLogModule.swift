import ExpoModulesCore
// Rust C declarations are exposed by this pod's public ainavlog.h header.
import CoreBluetooth

// Swift connects Expo to Rust; btleplug owns scanning and CoreBluetooth callbacks.
public class AiNavLogModule: Module {
  private let queue = DispatchQueue(label: "com.ainavlog.bridge")
  private var handle: UInt64 = 0
  private var foreground = true

  private func stop() {
    _ = ainavlog_simulator_stop()
    handle = 0
  }

  public func definition() -> ModuleDefinition {
    Name("AiNavLog")
    AsyncFunction("start") { () -> Void in
      guard self.foreground else { throw NSError(domain: "AiNavLog", code: -1, userInfo: [NSLocalizedDescriptionKey: "Open aiNavLog to connect the simulator."]) }
      let directory = try FileManager.default.url(for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
      let path = directory.appendingPathComponent("bmv-simulator.sqlite3").path
      let result = path.withCString { ainavlog_simulator_start($0, &self.handle) }
      if result != 0 { throw NSError(domain: "AiNavLog", code: Int(result), userInfo: [NSLocalizedDescriptionKey: "Could not open the Bluetooth receiver."]) }
    }.runOnQueue(queue)
    AsyncFunction("stop") { self.stop() }.runOnQueue(queue)
    AsyncFunction("snapshot") { () -> [String: Any] in
      let authorization = CBCentralManager.authorization
      var counters = ainavlog_simulator_counters()
      _ = ainavlog_simulator_diagnostics(&counters)
      var sample = ainavlog_electrical_sample()
      var value: Any = NSNull()
      if self.handle != 0 && ainavlog_latest(self.handle, &sample) == 0 {
        func field(_ bit: UInt32, _ number: Double) -> Any { sample.present & bit != 0 ? number as Any : NSNull() }
        value = ["source": "simulated", "receivedAt": Double(sample.received_ms),
          "houseVoltage": field(1, sample.house_voltage), "houseCurrent": field(2, sample.house_current),
          "houseCharge": field(4, sample.house_charge), "starterVoltage": field(8, sample.starter_voltage)] as [String: Any]
      }
      return ["status": Int(ainavlog_simulator_status()), "permissionDenied": authorization == .denied || authorization == .restricted, "sample": value,
        "diagnostics": ["packets": Double(counters.packets), "samples": Double(counters.samples), "sourceChanges": Double(counters.source_changes)]]
    }.runOnQueue(queue)
    OnAppEntersForeground { self.queue.async { self.foreground = true } }
    OnAppEntersBackground { self.queue.async { self.foreground = false; self.stop() } }
    OnDestroy { self.queue.async { self.stop() } }
  }
}
