/** Time-only chart navigation. All bounds and queries use integer Unix milliseconds. */
export type TimeWindow = { start: number; end: number };
export const MINUTE = 60_000;
export const HOUR = 60 * MINUTE;
export const MIN_WINDOW = 5 * MINUTE;
export const PINCH_THRESHOLD = 1.15;

export function sameWindow(a: TimeWindow, b: TimeWindow) {
  return a.start === b.start && a.end === b.end;
}
export function clampWindow(window: TimeWindow, bounds: TimeWindow): TimeWindow {
  const maximum = bounds.end - bounds.start;
  const span = Math.min(maximum, Math.max(Math.min(MIN_WINDOW, maximum), Math.round(window.end - window.start)));
  const start = Math.round(Math.max(bounds.start, Math.min(bounds.end - span, window.start)));
  return { start, end: start + span };
}
export function zoomWindow(window: TimeWindow, span: number, bounds: TimeWindow, time = (window.start + window.end) / 2, fraction = .5): TimeWindow {
  const duration = Math.min(bounds.end - bounds.start, Math.max(MIN_WINDOW, Math.round(span)));
  const anchor = Math.max(window.start, Math.min(window.end, time));
  const start = anchor - Math.max(0, Math.min(1, fraction)) * duration;
  return clampWindow({ start, end: start + duration }, bounds);
}
export function buttonZoom(window: TimeWindow, bounds: TimeWindow, direction: 'in' | 'out', time?: number) {
  const span = window.end - window.start;
  const step = span < HOUR || (direction === 'in' && span === HOUR) ? 5 * MINUTE : HOUR;
  return zoomWindow(window, span + (direction === 'in' ? -step : step), bounds, time);
}
export function panWindow(window: TimeWindow, offset: number, bounds: TimeWindow) {
  return clampWindow({ start: window.start + offset, end: window.end + offset }, bounds);
}
/** One deliberate pinch gesture halves/doubles once, anchored beneath the fingers. */
export function pinchWindow(window: TimeWindow, bounds: TimeWindow, scale: number, initialFraction: number, currentFraction = initialFraction) {
  const factor = scale >= PINCH_THRESHOLD ? .5 : scale <= 1 / PINCH_THRESHOLD ? 2 : 1;
  const span = window.end - window.start;
  return zoomWindow(window, span * factor, bounds, window.start + initialFraction * span, currentFraction);
}
export function windowLabel(window: TimeWindow) {
  const seconds = Math.round((window.end - window.start) / 1000);
  if (seconds % 3600 === 0) return `${seconds / 3600} ${seconds === 3600 ? 'hour' : 'hours'}`;
  if (seconds % 60 === 0 && seconds < 3600) return `${seconds / 60} minutes`;
  const hours = Math.floor(seconds / 3600), minutes = Math.floor(seconds % 3600 / 60), remaining = seconds % 60;
  return [hours ? `${hours} h` : '', minutes ? `${minutes} min` : '', remaining ? `${remaining} s` : ''].filter(Boolean).join(' ');
}
export type TouchPosition = { pageX: number; pageY: number };
export function touchPair(touches: readonly TouchPosition[]) {
  if (touches.length < 2) return null;
  const [a, b] = touches;
  return { distance: Math.hypot(b.pageX - a.pageX, b.pageY - a.pageY), midpoint: (a.pageX + b.pageX) / 2 };
}
