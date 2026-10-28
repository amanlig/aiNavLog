#include <jni.h>
#include "ainavlog.h"
#include <vector>

extern "C" JNIEXPORT jint JNICALL
Java_expo_modules_ainavlog_Bridge_nativeInit(JNIEnv *env, jobject) {
  return ainavlog_android_init(env);
}
extern "C" JNIEXPORT jlong JNICALL
Java_expo_modules_ainavlog_Bridge_nativeStart(JNIEnv *env, jobject, jstring path) {
  if (!path) return 0;
  const char *text = env->GetStringUTFChars(path, nullptr);
  if (!text) return 0;
  uint64_t handle = 0;
  const int32_t result = ainavlog_simulator_start(text, &handle);
  env->ReleaseStringUTFChars(path, text);
  return result == 0 ? static_cast<jlong>(handle) : 0;
}
extern "C" JNIEXPORT jint JNICALL
Java_expo_modules_ainavlog_Bridge_nativeStop(JNIEnv *, jobject) {
  return ainavlog_simulator_stop();
}
extern "C" JNIEXPORT jint JNICALL
Java_expo_modules_ainavlog_Bridge_nativeStatus(JNIEnv *, jobject) {
  return ainavlog_simulator_status();
}
extern "C" JNIEXPORT jlongArray JNICALL
Java_expo_modules_ainavlog_Bridge_nativeCounters(JNIEnv *env, jobject) {
  ainavlog_simulator_counters counters = {};
  if (ainavlog_simulator_diagnostics(&counters) != 0) return nullptr;
  const jlong values[] = {static_cast<jlong>(counters.packets), static_cast<jlong>(counters.samples), static_cast<jlong>(counters.source_changes)};
  jlongArray result = env->NewLongArray(3);
  if (result) env->SetLongArrayRegion(result, 0, 3, values);
  return result;
}
extern "C" JNIEXPORT jdoubleArray JNICALL
Java_expo_modules_ainavlog_Bridge_nativeSample(JNIEnv *env, jobject, jlong handle) {
  ainavlog_electrical_sample sample = {};
  if (ainavlog_latest(static_cast<uint64_t>(handle), &sample) != 0) return nullptr;
  const jdouble values[] = {static_cast<double>(sample.received_ms), static_cast<double>(sample.present),
    sample.house_voltage, sample.house_current, sample.house_charge, sample.starter_voltage};
  jdoubleArray result = env->NewDoubleArray(6);
  if (result) env->SetDoubleArrayRegion(result, 0, 6, values);
  return result;
}

extern "C" JNIEXPORT jdoubleArray JNICALL
Java_expo_modules_ainavlog_Bridge_nativeHistory(JNIEnv *env, jobject, jstring path, jlong start, jlong end) {
  if (!path) return nullptr;
  const char *text = env->GetStringUTFChars(path, nullptr);
  if (!text) return nullptr;
  std::vector<ainavlog_history_point> rows(8192);
  size_t count = 0;
  const int32_t status = ainavlog_history(text, "windows-bmv-simulator", start, end, rows.data(), rows.size(), &count);
  env->ReleaseStringUTFChars(path, text);
  if (status != 0) return nullptr;
  std::vector<jdouble> values;
  values.reserve(count * 7);
  for (size_t i = 0; i < count; ++i) {
    const auto &sample = rows[i].sample;
    values.insert(values.end(), {static_cast<double>(sample.received_ms), static_cast<double>(sample.present),
      sample.house_voltage, sample.house_current, sample.house_charge, sample.starter_voltage, static_cast<double>(rows[i].segment)});
  }
  jdoubleArray result = env->NewDoubleArray(static_cast<jsize>(values.size()));
  if (result && !values.empty()) env->SetDoubleArrayRegion(result, 0, static_cast<jsize>(values.size()), values.data());
  return result;
}
