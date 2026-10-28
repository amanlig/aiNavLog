package expo.modules.ainavlog

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition

internal object Bridge {
  init { System.loadLibrary("ainavlog_jni") }
  external fun nativeInit(): Int
  external fun nativeStart(path: String): Long
  external fun nativeStop(): Int
  external fun nativeStatus(): Int
  external fun nativeCounters(): LongArray?
  external fun nativeSample(handle: Long): DoubleArray?
  external fun nativeHistory(path: String, start: Long, end: Long): DoubleArray?
}

class AiNavLogModule : Module() {
  private val lock = Any()
  private val cleanup = java.util.concurrent.Executors.newSingleThreadExecutor()
  @Volatile private var foreground = true
  private var handle = 0L
  private fun stop() = synchronized(lock) {
    Bridge.nativeStop()
    handle = 0L
  }
  override fun definition() = ModuleDefinition {
    Name("AiNavLog")
    AsyncFunction("start") {
      synchronized(lock) {
        check(foreground) { "Open aiNavLog to connect the simulator" }
        val context = appContext.reactContext ?: error("App context unavailable")
        check(Bridge.nativeInit() == 0) { "Could not initialize Bluetooth" }
        handle = Bridge.nativeStart(java.io.File(context.filesDir, "bmv-simulator.sqlite3").absolutePath)
        check(handle != 0L) { "Could not open the Bluetooth receiver" }
      }
    }
    AsyncFunction("stop") { stop(); Unit }
    AsyncFunction("history") { start: Double, end: Double ->
      check(start.isFinite() && end.isFinite() && start >= 0 && end > start && end <= 8_640_000_000_000_000.0 && end-start <= 32*86_400_000.0) { "Invalid history range" }
      val context = appContext.reactContext ?: error("App context unavailable")
      val path = java.io.File(context.filesDir, "bmv-simulator.sqlite3").absolutePath
      val rows = Bridge.nativeHistory(path, start.toLong(), end.toLong()) ?: error("Could not read saved history. Try a shorter range.")
      rows.toList().chunked(7)
    }
    AsyncFunction("snapshot") {
      synchronized(lock) {
        val row = if (handle != 0L) Bridge.nativeSample(handle) else null
        val sample = row?.let {
          val mask = it[1].toInt()
          fun field(bit: Int, index: Int): Double? = if (mask and bit != 0) it[index] else null
          mapOf("source" to "simulated", "receivedAt" to it[0],
            "houseVoltage" to field(1, 2), "houseCurrent" to field(2, 3),
            "houseCharge" to field(4, 4), "starterVoltage" to field(8, 5))
        }
        val counters = Bridge.nativeCounters() ?: longArrayOf(0, 0, 0)
        mapOf("status" to Bridge.nativeStatus(), "permissionDenied" to false, "sample" to sample,
          "diagnostics" to mapOf("packets" to counters[0].toDouble(), "samples" to counters[1].toDouble(), "sourceChanges" to counters[2].toDouble()))
      }
    }
    OnActivityEntersForeground { foreground = true }
    OnActivityEntersBackground { foreground = false; cleanup.execute { stop() } }
    OnDestroy { foreground = false; cleanup.execute { stop() }; cleanup.shutdown() }
  }
}
