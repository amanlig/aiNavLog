export type ElectricalSample = {
  source: 'simulated' | 'live';
  receivedAt: number;
  houseVoltage: number | null;
  houseCurrent: number | null;
  houseCharge: number | null;
  starterVoltage: number | null;
};
export const STALE_MS = 5000;
export function isFresh(sample: ElectricalSample | null, now: number) {
  return sample !== null && now >= sample.receivedAt && now - sample.receivedAt <= STALE_MS;
}
export function starterAlert(sample: ElectricalSample | null, now: number, threshold: number | null) {
  if (threshold === null) return 'Not configured';
  if (!isFresh(sample, now) || sample?.starterVoltage == null || !Number.isFinite(sample.starterVoltage)) return 'Unavailable';
  return sample.starterVoltage < threshold ? 'Low voltage' : 'At or above threshold';
}
export function reading(value: number | null | undefined, digits = 2, signed = false) {
  if (value == null || !Number.isFinite(value)) return '—';
  return `${signed && value > 0 ? '+' : ''}${value.toFixed(digits)}`;
}
export function simulate(step: number, now: number): ElectricalSample {
  const phase = step % 120;
  const charging = phase >= 60;
  const offset = charging ? phase - 60 : phase;
  return {
    source: 'simulated', receivedAt: now,
    houseVoltage: charging ? 14 + Math.floor(offset / 10) / 100 : 12.76 - Math.floor(offset / 10) / 100,
    houseCurrent: charging ? 12 : -5.25,
    houseCharge: charging ? 76.3 + Math.floor(offset / 5) / 10 : 77.5 - Math.floor(offset / 5) / 10,
    starterVoltage: charging ? 14.2 + Math.floor(offset / 15) / 100 : 12.8 - Math.floor(offset / 15) / 100,
  };
}
export function parseThreshold(text: string): number | null {
  if (!/^\d+(\.\d{1,2})?$/.test(text.trim())) return null;
  const value = Number(text);
  return value > 0 && value <= 70 ? value : null;
}
