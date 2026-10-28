import { useCallback, useEffect, useRef, useState } from 'react';
import { type GestureResponderEvent, type View } from 'react-native';
import { panWindow, pinchWindow, sameWindow, touchPair, type TimeWindow } from './history-viewport';

type Frame = { window: TimeWindow; x: number; pair: ReturnType<typeof touchPair>; fraction: number };
const fractionAt = (pageX: number, left: number, width: number) => Math.max(0, Math.min(1, ((pageX - left) / width * 420 - 55) / 345));

/** Claim pinch/horizontal movement, leaving ordinary vertical scrolling alone.
 * Preview during movement; commit/query only once on release. */
export function useHistoryGestures(window: TimeWindow, bounds: TimeWindow,
  onWindow: (next: TimeWindow) => void, onActive: (active: boolean) => void, onSelect: (time: number) => void) {
  const ref = useRef<View>(null);
  const geometry = useRef({ left: 0, width: 0 });
  const frame = useRef<Frame | null>(null);
  const origin = useRef<{ x: number; y: number } | null>(null);
  const pending = useRef<TimeWindow | null>(null);
  const blockTap = useRef(false);
  const unblock = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [preview, setPreview] = useState<TimeWindow | null>(null);
  const measure = useCallback(() => ref.current?.measureInWindow((left, _top, width) => { geometry.current = { left, width }; }), []);
  const begin = useCallback((event: GestureResponderEvent) => {
    const touches = event.nativeEvent.touches;
    if (!touches.length || geometry.current.width <= 0) return;
    if (unblock.current) clearTimeout(unblock.current);
    const pair = touchPair(touches);
    const current = pending.current ?? window;
    const x = pair?.midpoint ?? touches[0].pageX;
    frame.current = { window: current, x, pair, fraction: fractionAt(x, geometry.current.left, geometry.current.width) };
    blockTap.current = true;
    onActive(true);
  }, [window, onActive]);
  const move = useCallback((event: GestureResponderEvent) => {
    const current = frame.current;
    const touches = event.nativeEvent.touches;
    const { left, width } = geometry.current;
    if (!current || !touches.length || width <= 0) return;
    event.preventDefault();
    const pair = touchPair(touches);
    if (pair && !current.pair) { begin(event); return; }
    // Lifting one finger must not turn a completed pinch into an accidental pan.
    if (current.pair && !pair) return;
    const next = current.pair && pair && current.pair.distance > 0
      ? pinchWindow(current.window, bounds, pair.distance / current.pair.distance, current.fraction, fractionAt(pair.midpoint, left, width))
      : panWindow(current.window, -(touches[0].pageX - current.x) / (width * 345 / 420) * (current.window.end - current.window.start), bounds);
    pending.current = next;
    setPreview(previous => previous && sameWindow(previous, next) ? previous : next);
  }, [begin, bounds]);
  const finish = useCallback((commit: boolean) => {
    const next = pending.current;
    frame.current = null; pending.current = null; setPreview(null); onActive(false);
    unblock.current = setTimeout(() => { blockTap.current = false; }, 250);
    if (commit && next && !sameWindow(next, window)) onWindow(next);
  }, [window, onActive, onWindow]);
  const start = useCallback((event: GestureResponderEvent) => {
    measure();
    const touches = event.nativeEvent.touches;
    if (touches.length === 1 && !frame.current) origin.current = { x: touches[0].pageX, y: touches[0].pageY };
    return touches.length >= 2;
  }, [measure]);
  const wants = useCallback((event: GestureResponderEvent) => {
    const touches = event.nativeEvent.touches;
    if (touches.length >= 2) return true;
    if (!origin.current || !touches.length || window.end - window.start >= bounds.end - bounds.start) return false;
    const dx = touches[0].pageX - origin.current.x, dy = touches[0].pageY - origin.current.y;
    return Math.abs(dx) > 8 && Math.abs(dx) > Math.abs(dy) * 1.2;
  }, [window, bounds]);
  const anotherTouch = useCallback((event: GestureResponderEvent) => { if (frame.current && event.nativeEvent.touches.length >= 2) begin(event); }, [begin]);
  const release = useCallback(() => finish(true), [finish]);
  const cancel = useCallback(() => finish(false), [finish]);
  const attachView = useCallback((view: View | null) => { ref.current = view; }, []);
  useEffect(() => () => { if (unblock.current) clearTimeout(unblock.current); onActive(false); }, [onActive]);
  const select = useCallback((pageX: number) => {
    if (blockTap.current || !Number.isFinite(pageX)) return;
    ref.current?.measureInWindow((left, _top, width) => {
      if (width > 0) onSelect(window.start + fractionAt(pageX, left, width) * (window.end - window.start));
    });
  }, [onSelect, window]);
  return { attachView, measure, preview, select, handlers: {
    onStartShouldSetResponderCapture: start,
    onMoveShouldSetResponderCapture: wants,
    onMoveShouldSetResponder: wants,
    onResponderGrant: begin,
    onResponderStart: anotherTouch,
    onResponderMove: move,
    onResponderRelease: release,
    onResponderTerminate: cancel,
    onResponderTerminationRequest: () => false,
  } };
}
