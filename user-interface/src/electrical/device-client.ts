import type { ElectricalSample } from './model';

/** Validate the local Rust endpoint without replacing its receipt timestamp. */
export function parseDeviceSample(value: unknown): ElectricalSample | null {
  if (value === null) return null;
  if (typeof value !== 'object' || value === null) throw new Error('Invalid device sample');
  const row = value as Record<string, unknown>;
  if ((row.source !== 'live' && row.source !== 'simulated') || typeof row.receivedAt !== 'number' || !Number.isSafeInteger(row.receivedAt) || row.receivedAt < 0) throw new Error('Invalid device timestamp');
  for (const field of ['houseVoltage', 'houseCurrent', 'houseCharge', 'starterVoltage']) {
    if (row[field] !== null && (typeof row[field] !== 'number' || !Number.isFinite(row[field]))) throw new Error(`Invalid ${field}`);
  }
  return row as ElectricalSample;
}
