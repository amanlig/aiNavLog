import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from 'react';
import { AppState } from 'react-native';
import { parseDeviceSample } from './device-client';
import { type ElectricalSample, simulate } from './model';
import { nativeMode, startReceiver, stopReceiver, readReceiver, type ReceiverDiagnostics } from './native-client';

const deviceUrl = process.env.EXPO_PUBLIC_DEMO_MODE === '1'
  ? undefined : process.env.EXPO_PUBLIC_DEVICE_INTERFACE_URL?.trim();
function foreground() { return AppState.currentState !== 'background' && AppState.currentState !== 'inactive'; }
function useElectricalState() {
  const [sample, setSample] = useState<ElectricalSample | null>(null);
  const [now, setNow] = useState(Date.now);
  const [error, setError] = useState<string | null>(null);
  const [connected, setConnected] = useState(false);
  const [busy, setBusy] = useState(false);
  const [diagnostics, setDiagnostics] = useState<ReceiverDiagnostics | null>(null);
  const wanted = useRef(false);
  const queue = useRef(Promise.resolve());
  const serialize = (operation: () => Promise<void>) => {
    queue.current = queue.current.then(operation).catch(() => {});
    return queue.current;
  };
  const connect = () => {
    wanted.current = true;
    setBusy(true);
    return serialize(async () => {
      try {
        if (!wanted.current || !foreground()) return;
        await startReceiver();
        setConnected(true); setError(null);
      } catch (failure) {
        wanted.current = false; setConnected(false); setSample(null);
        setError(failure instanceof Error ? failure.message : 'Bluetooth unavailable.');
      } finally { setBusy(false); }
    });
  };
  const disconnect = () => {
    wanted.current = false;
    setBusy(true);
    return serialize(async () => {
      try { await stopReceiver(); } finally {
        setConnected(false); setSample(null); setError(null); setBusy(false);
      }
    });
  };
  useEffect(() => {
    let disposed = false;
    let step = 0;
    let polling = false;
    let controller: AbortController | null = null;
    const tick = async () => {
      const time = Date.now();
      setNow(time);
      if (!foreground() || polling) return;
      polling = true;
      if (nativeMode) {
        try {
          const next = await readReceiver();
          if (!disposed) { setSample(next.sample); setDiagnostics(next.diagnostics); if (wanted.current || next.error) setError(next.error); }
        } catch (failure) {
          if (!disposed) { setSample(null); setError(failure instanceof Error ? failure.message : 'Bluetooth unavailable.'); }
        } finally { polling = false; }
        return;
      }
      if (!deviceUrl) { setSample(simulate(step++, time)); polling = false; return; }
      const request = new AbortController();
      controller = request;
      const timeout = setTimeout(() => request.abort(), 3000);
      try {
        const response = await fetch(deviceUrl, { signal: request.signal });
        if (!response.ok) throw new Error(`Device service returned ${response.status}`);
        const next = parseDeviceSample(await response.json());
        if (!disposed && !request.signal.aborted) { setSample(next); setError(null); }
      } catch {
        if (!disposed && foreground()) { setSample(null); setError('Device connection unavailable. Retrying…'); }
      } finally { clearTimeout(timeout); controller = null; polling = false; }
    };
    void tick();
    const timer = setInterval(() => { void tick(); }, 1000);
    const subscription = AppState.addEventListener('change', (state) => {
      setNow(Date.now());
      if (state === 'active') {
        if (nativeMode && wanted.current) void serialize(async () => {
          if (!wanted.current || !foreground()) return;
          try { await startReceiver(); setConnected(true); setError(null); }
          catch { setConnected(false); setError('Bluetooth unavailable. Connect again to retry.'); }
        });
        void tick();
      } else {
        controller?.abort();
        if (nativeMode) void serialize(async () => { await stopReceiver(); setConnected(false); });
      }
    });
    return () => {
      disposed = true; wanted.current = false; clearInterval(timer); subscription.remove(); controller?.abort();
      if (nativeMode) void serialize(stopReceiver);
    };
  }, []);
  return { sample, now, error, diagnostics, live: nativeMode || Boolean(deviceUrl), native: nativeMode, connected, busy, connect, disconnect };
}
const ElectricalContext = createContext<ReturnType<typeof useElectricalState> | null>(null);
export function ElectricalProvider({ children }: { children: ReactNode }) {
  const value = useElectricalState();
  return <ElectricalContext.Provider value={value}>{children}</ElectricalContext.Provider>;
}
export function useElectrical() {
  const value = useContext(ElectricalContext);
  if (!value) throw new Error('ElectricalProvider is required');
  return value;
}
