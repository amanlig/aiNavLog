export type Battery = 'house' | 'starter';
export type Metric = 'houseCharge' | 'houseVoltage' | 'houseCurrent' | 'starterVoltage';
export type HistoryPoint = {
  receivedAt: number; segment: number;
  houseVoltage: number | null; houseCurrent: number | null;
  houseCharge: number | null; starterVoltage: number | null;
};
export type HistoryResult = { source: 'simulated' | 'live'; demo: boolean; points: HistoryPoint[] };
export const metrics: Record<Metric, { title: string; unit: string; digits: number }> = {
  houseCharge: { title: 'Charge level', unit: '%', digits: 1 },
  houseVoltage: { title: 'Voltage', unit: 'V', digits: 1 },
  houseCurrent: { title: 'Current', unit: 'A', digits: 1 },
  starterVoltage: { title: 'Voltage', unit: 'V', digits: 1 },
};
export function calendarRange(anchor: Date, days: number) {
  const end = new Date(anchor.getFullYear(), anchor.getMonth(), anchor.getDate() + 1);
  const start = new Date(end.getFullYear(), end.getMonth(), end.getDate() - days);
  return { start: start.getTime(), end: end.getTime() };
}
export function dateRange(anchor: Date, days: number, now = Date.now()) {
  // Calendar days in the device timezone, including daylight-saving transitions.
  const range = calendarRange(anchor, days);
  return { ...range, end: Math.min(range.end, now + 1) };
}
export function shiftDay(anchor: Date, days: number) {
  return new Date(anchor.getFullYear(), anchor.getMonth(), anchor.getDate() + days);
}
export function parseHistory(value: unknown, start: number, end: number): HistoryPoint[] {
  if (!Array.isArray(value) || value.length > 8192) throw new Error('Invalid saved history');
  let previous = -1;
  return value.map(row => {
    if (!Array.isArray(row) || row.length !== 7 || !row.every(n => typeof n === 'number' && Number.isFinite(n))) throw new Error('Invalid saved history');
    const [receivedAt, mask, v, a, soc, starter, segment] = row as number[];
    if (!Number.isSafeInteger(receivedAt) || receivedAt < start || receivedAt >= end || receivedAt < previous || !Number.isInteger(mask) || mask < 0 || mask > 15 || !Number.isSafeInteger(segment) || segment < 1) throw new Error('Invalid saved history');
    previous = receivedAt;
    return { receivedAt, segment, houseVoltage: mask & 1 ? v : null, houseCurrent: mask & 2 ? a : null, houseCharge: mask & 4 ? soc : null, starterVoltage: mask & 8 ? starter : null };
  });
}
export function series(points: HistoryPoint[], metric: Metric): HistoryPoint[][] {
  const runs: HistoryPoint[][] = [];
  let run: HistoryPoint[] = [];
  let segment = -1;
  for (const point of points) {
    if (point.segment !== segment || point[metric] === null) {
      if (run.length) runs.push(run);
      run = [];
    }
    if (point[metric] !== null) run.push(point);
    segment = point.segment;
  }
  if (run.length) runs.push(run);
  return runs;
}
export function nearest(points: HistoryPoint[], metric: Metric, time: number) {
  let found: HistoryPoint | null = null;
  for (const point of points) if (point[metric] !== null && (!found || Math.abs(point.receivedAt - time) < Math.abs(found.receivedAt - time))) found = point;
  return found;
}
export function metricRange(points: HistoryPoint[], metric: Metric) {
  let low = Infinity, high = -Infinity;
  for (const point of points) { const value = point[metric]; if (value !== null) { low = Math.min(low, value); high = Math.max(high, value); } }
  return low === Infinity ? null : { low, high };
}
export function demoHistory(start: number, end: number): HistoryPoint[] {
  const points: HistoryPoint[] = [];
  // Absolute local-clock values keep the illustrative record consistent across
  // zoom/range queries. The daily gap is fixed, not regenerated per viewport.
  for (let i = 0; i <= 240; i++) {
    const t = start + Math.floor((end - start - 1) * i / 240);
    const date = new Date(t);
    const phase = (date.getHours() * 3_600_000 + date.getMinutes() * 60_000 + date.getSeconds() * 1000 + date.getMilliseconds()) / 86_400_000;
    if (phase >= .3 && phase < .4) continue;
    const day = date.getFullYear() * 10000 + (date.getMonth() + 1) * 100 + date.getDate();
    points.push({ receivedAt: t, segment: day * 2 + (phase < .3 ? 1 : 2),
      houseCharge: 82 - phase * 8 + (phase > .65 ? Math.min((phase - .65) * 40, 11) : 0),
      houseVoltage: phase > .65 && phase < .9 ? 14.1 : 12.8 - phase * .1,
      houseCurrent: phase > .65 && phase < .9 ? 12 : -5.25,
      starterVoltage: phase > .65 && phase < .9 ? 14.2 : 12.8 - phase * .15 });
  }
  return points;
}
