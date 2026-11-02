export type System = 'engine' | 'electrical' | 'both';
export type Equipment = { id: string; name: string; system: System };
export type Entry = {
  id: string; equipmentId: string; system?: 'engine' | 'electrical'; date: string; kind: 'Service' | 'Inspection' | 'Repair' | 'Replacement';
  description: string; provider: string; hours: number | null; parts: string;
  costMinor: number | null; currency: string; reference: string; notes: string;
  nextDate: string | null; nextHours: number | null;
};
export type SavedEntry = Entry & { createdAt: number };
export type Snapshot = { equipment: Equipment[]; entries: SavedEntry[] };
export const newId = () => `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}-${Math.random().toString(36).slice(2)}`;
export function localDate(date = new Date()) {
  return `${date.getFullYear()}-${String(date.getMonth()+1).padStart(2,'0')}-${String(date.getDate()).padStart(2,'0')}`;
}
export function validDate(value: string) {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value) || value < '1900-01-01') return false;
  const [y,m,d] = value.split('-').map(Number);
  const date = new Date(y,m-1,d);
  return date.getFullYear()===y && date.getMonth()===m-1 && date.getDate()===d;
}
export function optionalNumber(value: string, label: string) {
  if (!value.trim()) return null;
  if (!/^\d+(\.\d+)?$/.test(value.trim())) throw Error(`${label} must be a positive number or zero.`);
  const number = Number(value);
  if (!Number.isFinite(number) || number>1e9) throw Error(`${label} is too large.`);
  return number;
}
export function costMinor(value: string) {
  if (!value.trim()) return null;
  if (!/^\d+(\.\d{1,2})?$/.test(value.trim())) throw Error('Cost must have at most two decimal places.');
  const [whole,fraction=''] = value.trim().split('.');
  const amount=Number(whole)*100+Number(fraction.padEnd(2,'0'));
  if (!Number.isSafeInteger(amount) || amount>999999999999) throw Error('Cost is too large.');
  return amount;
}
export function validate(entry: Entry) {
  if (!entry.equipmentId) throw Error('Select equipment.');
  if (!validDate(entry.date)) throw Error('Enter a valid service date (YYYY-MM-DD).');
  if (entry.date>localDate()) throw Error('Completed work cannot have a future service date.');
  if (!entry.description.trim()) throw Error('Describe the work performed.');
  if (entry.nextDate && (!validDate(entry.nextDate) || entry.nextDate<entry.date)) throw Error('Next service date must be on or after the work date.');
  if (entry.nextHours!==null && entry.hours!==null && entry.nextHours<=entry.hours) throw Error('Next service hours must exceed the recorded equipment hours.');
  if (entry.costMinor!==null && !/^[A-Z]{3}$/.test(entry.currency)) throw Error('Enter a three-letter currency such as CAD or USD.');
}
export function inSystem(equipment: Equipment, system: System | 'all') {
  return system==='all' || equipment.system==='both' || equipment.system===system;
}

/** Older records without a log retain their original meaning. Shared equipment
 * cannot be assigned to either log without an owner's decision. */
export function recordSystem(entry: Pick<Entry, 'system'>, equipment: Equipment) {
  return entry.system ?? (equipment.system === 'both' ? 'legacy' : equipment.system);
}
