import { useEffect, useState } from 'react';
import { AppState } from 'react-native';
import { parseDeviceSample } from './device-client';
import { type ElectricalSample, simulate } from './model';

// The first standalone Play build deliberately uses the labelled demo. A desktop
// loopback URL must not turn a phone build into a permanent connection error.
const deviceUrl = process.env.EXPO_PUBLIC_DEMO_MODE === '1'
  ? undefined : process.env.EXPO_PUBLIC_DEVICE_INTERFACE_URL?.trim();
function foreground() { return AppState.currentState !== 'background' && AppState.currentState !== 'inactive'; }
export function useElectrical() {
  const [sample, setSample] = useState<ElectricalSample | null>(null);
  const [now, setNow] = useState(Date.now);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let disposed = false;
    let step = 0;
    let controller: AbortController | null = null;
    const tick = async () => {
      const time = Date.now();
      setNow(time);
      if (!foreground() || controller) return;
      if (!deviceUrl) { setSample(simulate(step++, time)); return; }
      const request = new AbortController();
      controller = request;
      const timeout = setTimeout(() => request.abort(), 3000);
      try {
        const response = await fetch(deviceUrl, { signal: request.signal });
        if (!response.ok) throw new Error(`Device service returned ${response.status}`);
        const next = parseDeviceSample(await response.json());
        if (!disposed && !request.signal.aborted) { setSample(next); setError(null); }
      } catch {
        if (!disposed && foreground()) {
          setSample(null); setError('Device connection unavailable. Retrying…');
        }
      } finally { clearTimeout(timeout); controller = null; }
    };
    void tick();
    const timer = setInterval(() => { void tick(); }, 1000);
    const subscription = AppState.addEventListener('change', (state) => {
      setNow(Date.now());
      if (state === 'active') void tick();
      else controller?.abort();
    });
    return () => { disposed = true; clearInterval(timer); subscription.remove(); controller?.abort(); };
  }, []);
  return { sample, now, error, live: Boolean(deviceUrl) };
}
