import { requireOptionalNativeModule } from 'expo';
import { PermissionsAndroid, Platform } from 'react-native';
import { parseDeviceSample } from './device-client';

export type ReceiverDiagnostics = { packets: number; samples: number; sourceChanges: number };
interface Receiver {
  start(): Promise<void>;
  stop(): Promise<void>;
  snapshot(): Promise<{ status: number; permissionDenied: boolean; sample: unknown; diagnostics?: ReceiverDiagnostics }>;
  history?(start: number, end: number): Promise<unknown>;
}
export async function readNativeHistory(start: number, end: number) {
  if (!receiver?.history) throw new Error('Saved history requires a newer aiNavLog native build.');
  return receiver.history(start, end);
}
const receiver = Platform.OS === 'web' ? null : requireOptionalNativeModule<Receiver>('AiNavLog');
export const nativeMode = Platform.OS !== 'web' && process.env.EXPO_PUBLIC_NATIVE_BMV === '1';

export async function startReceiver() {
  if (!receiver) throw new Error('Bluetooth receiver is missing. Install the latest aiNavLog build.');
  if (Platform.OS === 'android') {
    const required = Number(Platform.Version) >= 31
      ? [PermissionsAndroid.PERMISSIONS.BLUETOOTH_SCAN, PermissionsAndroid.PERMISSIONS.BLUETOOTH_CONNECT]
      : [PermissionsAndroid.PERMISSIONS.ACCESS_FINE_LOCATION];
    const granted = await PermissionsAndroid.requestMultiple(required);
    if (required.some(permission => granted[permission] !== PermissionsAndroid.RESULTS.GRANTED)) {
      throw new Error('Bluetooth permission denied. Allow access in phone Settings, then connect again.');
    }
  }
  await receiver.start();
}
export async function stopReceiver() { await receiver?.stop(); }
export async function readReceiver() {
  if (!receiver) throw new Error('Bluetooth receiver is missing. Install the latest aiNavLog build.');
  const result = await receiver.snapshot();
  return {
    sample: parseDeviceSample(result.sample),
    error: result.permissionDenied ? 'Bluetooth permission denied. Allow access in phone Settings, then connect again.'
      : result.status < 0 ? 'Bluetooth unavailable. Check Bluetooth and simulator broadcasting. Retrying…' : null,
    status: result.status,
    diagnostics: result.diagnostics ?? null,
  };
}
