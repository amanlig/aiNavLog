import { nativeMode, readNativeHistory } from './native-client';
import { demoHistory, parseHistory, type HistoryResult } from './history';

export async function readHistory(start: number, end: number, signal?: AbortSignal): Promise<HistoryResult> {
  if (nativeMode) return { source: 'simulated', demo: false, points: parseHistory(await readNativeHistory(start, end), start, end) };
  const endpoint = process.env.EXPO_PUBLIC_DEMO_MODE === '1' ? '' : process.env.EXPO_PUBLIC_DEVICE_INTERFACE_URL?.trim();
  if (!endpoint) {
    const availableEnd = Math.min(end, Date.now() + 1);
    return { source: 'simulated', demo: true, points: availableEnd > start ? demoHistory(start, availableEnd) : [] };
  }
  const url = new URL(endpoint);
  url.pathname = `${url.pathname.replace(/\/$/, '')}/history`;
  url.search = `start=${start}&end=${end}`;
  const response = await fetch(url.toString(), { signal });
  if (!response.ok) throw new Error('Could not read saved history. Check the device service or try a shorter range.');
  const result = await response.json();
  if (result?.source !== 'simulated' && result?.source !== 'live') throw new Error('Invalid history source');
  return { source: result.source, demo: false, points: parseHistory(result.points, start, end) };
}
