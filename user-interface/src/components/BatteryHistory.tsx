import { useCallback, useEffect, useMemo, useState } from 'react';
import { Link } from 'expo-router';
import { ActivityIndicator, AppState, Platform, Pressable, ScrollView, StyleSheet, Text, View, type ViewStyle } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { StatusBar } from 'expo-status-bar';
import Svg, { Circle, Line, Path, Rect, Text as SvgText } from 'react-native-svg';
import { AppHeader } from './AppHeader';
import { usePreferences } from '../preferences/Preferences';
import { readHistory } from '../electrical/history-client';
import { calendarRange, metricRange, metrics, nearest, series, shiftDay, type Battery, type HistoryPoint, type HistoryResult, type Metric } from '../electrical/history';
import { reading } from '../electrical/model';
import { buttonZoom, clampWindow, HOUR, MIN_WINDOW, panWindow, sameWindow, windowLabel, type TimeWindow } from '../electrical/history-viewport';
import { useHistoryGestures } from '../electrical/use-history-gestures';

const timeLabel = (time: number, days: number, detailed = false) => new Date(time).toLocaleString(undefined,
  { month: days === 1 ? undefined : 'short', day: days === 1 ? undefined : 'numeric', hour: '2-digit', minute: '2-digit', second: detailed ? '2-digit' : undefined });

function HistoryChart({ points, metric, window, limits, selected, onSelect, onWindow, onActive, disabled }: {
  points: HistoryPoint[]; metric: Metric; window: TimeWindow; limits: TimeWindow;
  selected: HistoryPoint | null; onSelect: (time: number) => void;
  onWindow: (next: TimeWindow) => void; onActive: (active: boolean) => void; disabled: boolean;
}) {
  const { colors: c } = usePreferences();
  const { attachView, measure, handlers, preview, select } = useHistoryGestures(window, limits, onWindow, onActive, onSelect);
  const shown = preview ?? window;
  const { start, end } = shown;
  const visible = points.filter(p => p.receivedAt >= start && p.receivedAt < end);
  const recordedBounds = metricRange(visible, metric) ?? metricRange(points, metric);
  const bounds = recordedBounds ?? { low: 0, high: 1 };
  const runs = series(visible, metric);
  const padding = Math.max((bounds.high - bounds.low) * .15, metric === 'houseCharge' ? 1 : .1);
  const bottom = bounds.low - padding, top = bounds.high + padding;
  const x = (t: number) => 55 + (t - start) / (end - start) * 345;
  const y = (value: number) => 25 + (top - value) / (top - bottom) * 150;
  const gaps: [number, number][] = [];
  let previous = start;
  for (const run of runs) {
    if (run[0].receivedAt > previous) gaps.push([previous, run[0].receivedAt]);
    previous = run[run.length - 1].receivedAt;
  }
  if (previous < end) gaps.push([previous, end]);
  return <View>
    {preview && <Text style={{ color: c.accent, marginBottom: 8 }}>Preview · {windowLabel(shown)}</Text>}
    <View ref={attachView} onLayout={measure} {...handlers} pointerEvents={disabled ? 'none' : 'auto'}
      style={Platform.OS === 'web' ? { touchAction: 'pan-y' } as ViewStyle : undefined}>
      <Pressable accessibilityRole="button" accessibilityLabel={`${metrics[metric].title} history chart. ${recordedBounds ? `Lowest ${reading(bounds.low, 1)}, highest ${reading(bounds.high, 1)} ${metrics[metric].unit}.` : 'No readings recorded.'} Pinch to zoom, drag horizontally to pan, or tap to inspect a reading. Zoom and pan buttons are also available.`}
        onPress={event => select(event.nativeEvent.pageX)}>
      <View pointerEvents="none" style={{ width: '100%', aspectRatio: 420 / 230 }}><Svg width="100%" height="100%" viewBox="0 0 420 230" accessible={false}>
        {gaps.map(([a, b], i) => <Rect key={`gap${i}`} x={x(a)} y={25} width={Math.max(0, x(b) - x(a))} height={150} fill={c.line} opacity={.38} />)}
        {[0, 1, 2].map(i => {
          const value = top - i / 2 * (top - bottom);
          return <ViewGrid key={i} y={y(value)} text={recordedBounds ? reading(value, metric === 'houseCharge' ? 0 : 1) : '—'} color={c.muted} line={c.line} />;
        })}
        {!visible.some(p => p[metric] !== null) && <SvgText x={225} y={108} textAnchor="middle" fontSize={13} fill={c.muted}>No readings in this interval</SvgText>}
        {runs.map((run, i) => run.length === 1
          ? <Circle key={i} cx={x(run[0].receivedAt)} cy={y(run[0][metric]!)} r={3} fill={c.accent} />
          : <Path key={i} d={run.map((p, index) => `${index ? 'L' : 'M'}${x(p.receivedAt)},${y(p[metric]!)}`).join(' ')} stroke={c.accent} strokeWidth={2.5} fill="none" />)}
        {selected && selected.receivedAt >= start && selected.receivedAt < end && selected[metric] !== null && <>
          <Line x1={x(selected.receivedAt)} x2={x(selected.receivedAt)} y1={25} y2={175} stroke={c.muted} strokeDasharray="3 4" />
          <Circle cx={x(selected.receivedAt)} cy={y(selected[metric]!)} r={5} fill={c.accent} stroke={c.panel} strokeWidth={2} />
        </>}
        {[0, .5, 1].map((fraction, i) => <SvgText key={i} x={55 + fraction * 345} y={202} fontSize={11} fill={c.muted} textAnchor={i === 0 ? 'start' : i === 2 ? 'end' : 'middle'}>
          {new Date(start + fraction * (end - start - 1)).toLocaleString(undefined, end - start <= 24 * HOUR ? { hour: '2-digit', minute: '2-digit', second: end - start <= HOUR ? '2-digit' : undefined } : { month: 'short', day: 'numeric' })}
        </SvgText>)}
      </Svg></View>
      </Pressable>
    </View>
    <Text style={{ color: c.muted, fontSize: 12 }}>{preview ? 'Release to load readings for this interval.' : 'Shaded areas = no readings recorded'}</Text>
    <Text style={{ color: c.muted, fontSize: 12, marginTop: 5 }}>Pinch to zoom · drag sideways to pan · tap to inspect.</Text>
  </View>;
}
function ViewGrid({ y, text, color, line }: { y: number; text: string; color: string; line: string }) {
  return <><Line x1={55} x2={400} y1={y} y2={y} stroke={line} /><SvgText x={43} y={y + 4} fontSize={12} fill={color} textAnchor="end">{text}</SvgText></>;
}

export function BatteryHistory() {
  const { colors: c } = usePreferences();
  const [battery, setBattery] = useState<Battery>('house');
  const [houseMetric, setHouseMetric] = useState<Metric>('houseCharge');
  const metric: Metric = battery === 'starter' ? 'starterVoltage' : houseMetric;
  const [days, setDays] = useState(1);
  const [anchor, setAnchor] = useState(() => new Date());
  const [refresh, setRefresh] = useState(0);
  const limits = useMemo(() => calendarRange(anchor, days), [anchor, days]);
  const [viewport, setViewport] = useState<TimeWindow | null>(null);
  const window = viewport ?? limits;
  const zoomed = !sameWindow(window, limits);
  const [gesturing, setGesturing] = useState(false);
  const [result, setResult] = useState<(HistoryResult & { start: number; end: number }) | null>(null);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(true);
  const [selectedTime, setSelectedTime] = useState<number | null>(null);
  const clearData = useCallback((reset = true) => { setLoading(true); setError(''); if (reset) { setResult(null); setSelectedTime(null); } }, []);
  const reload = useCallback(() => { clearData(false); setRefresh(n => n + 1); }, [clearData]);
  useEffect(() => {
    let active = true;
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 15000);
    const range = { start: window.start, end: window.end };
    readHistory(range.start, range.end, controller.signal).then(next => {
      if (active) { setResult({ ...next, ...range }); setSelectedTime(previous => previous ?? next.points.at(-1)?.receivedAt ?? null); }
    }).catch(failure => {
      if (active) { setResult(null); setError(controller.signal.aborted ? 'History request timed out. Please retry.' : failure instanceof Error ? failure.message : 'Could not load saved history.'); }
    }).finally(() => { clearTimeout(timeout); if (active) setLoading(false); });
    return () => { active = false; clearTimeout(timeout); controller.abort(); };
  }, [window.start, window.end, refresh]);
  useEffect(() => {
    const listener = AppState.addEventListener('change', state => { if (state === 'active') reload(); });
    return () => listener.remove();
  }, [reload]);
  const points = (result?.points ?? []).filter(p => p.receivedAt >= window.start && p.receivedAt < window.end);
  const bounds = metricRange(points, metric);
  const selected = selectedTime === null ? points.findLast(p => p[metric] !== null) ?? null : nearest(points, metric, selectedTime);
  const selectedPoints = points.filter(p => p[metric] !== null);
  const selectedIndex = selected ? selectedPoints.indexOf(selected) : -1;
  const applyWindow = useCallback((next: TimeWindow) => {
    const bounded = clampWindow(next, limits);
    if (sameWindow(bounded, window)) return;
    clearData(false); setViewport(sameWindow(bounded, limits) ? null : bounded);
  }, [limits, window, clearData]);
  const resetZoom = () => { if (zoomed) { clearData(false); setViewport(null); } };
  const info = metrics[metric];
  const isToday = anchor.toDateString() === new Date().toDateString();
  const button = (label: string, action: () => void, chosen = false, disabled = false, accessibleLabel = label) => <Pressable key={label} accessibilityRole="button" accessibilityLabel={accessibleLabel} accessibilityState={{ selected: chosen, disabled }} disabled={disabled}
    onPress={action} style={[s.button, { borderColor: chosen ? c.accent : c.line, backgroundColor: chosen ? c.soft : c.panel, opacity: disabled ? .45 : 1 }]}>
    <Text style={{ color: chosen ? c.accent : c.ink, fontWeight: chosen ? '700' : '400', textAlign: 'center' }}>{label}</Text>
  </Pressable>;
  return <SafeAreaView edges={['top', 'bottom']} style={s.safe}>
    <StatusBar style="light" /><AppHeader />
    <ScrollView testID="battery-history-scroll" scrollEnabled={!gesturing} pinchGestureEnabled={false} style={{ backgroundColor: c.background }} contentContainerStyle={s.page}>
      <Link href="/elec" style={{ color: c.accent, paddingVertical: 12 }}>‹ Electrical</Link>
      <Text accessibilityRole="header" style={[s.title, { color: c.ink }]}>Historical Usage</Text>
      <Text style={{ color: c.muted }}>Review the readings recorded on this device.</Text>
      <View style={s.row}>{button('House', () => setBattery('house'), battery === 'house')}{button('Starter', () => setBattery('starter'), battery === 'starter')}</View>
      <View style={[s.row, { alignItems: 'center' }]}>
        {button('‹ Earlier', () => { clearData(); setViewport(null); setAnchor(shiftDay(anchor, -days)); })}
        <Text style={{ flex: 2, color: c.ink, textAlign: 'center', fontWeight: '600' }}>{days > 1 ? `${shiftDay(anchor, 1 - days).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })} – ` : ''}{anchor.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })}</Text>
        {button('Later ›', () => { clearData(); setViewport(null); setAnchor(shiftDay(anchor, days) > new Date() ? new Date() : shiftDay(anchor, days)); }, false, isToday)}
      </View>
      <View style={s.row}>{[1, 7, 30].map(value => button(value === 1 ? '24 hours' : `${value} days`, () => { if (value !== days) { clearData(); setViewport(null); setDays(value); } }, days === value))}</View>
      <View style={[s.row, { alignItems: 'center' }]}>
        {button('−', () => applyWindow(buttonZoom(window, limits, 'out', selected?.receivedAt)), false, loading || !zoomed, 'Zoom out')}
        <View style={{ flex: 2, alignItems: 'center', gap: 4 }}>
          <Text style={{ color: c.muted, fontSize: 12 }}>Visible window</Text>
          <Text testID="history-window-duration" style={{ color: c.ink, fontWeight: '700' }}>{windowLabel(window)}</Text>
        </View>
        {button('+', () => applyWindow(buttonZoom(window, limits, 'in', selected?.receivedAt)), false, loading || window.end - window.start <= MIN_WINDOW, 'Zoom in')}
        {button('Reset', resetZoom, false, loading || !zoomed, 'Reset zoom')}
      </View>
      <Text testID="history-window-range" style={{ color: c.muted, fontSize: 12, textAlign: 'center' }}>
        {timeLabel(window.start, 7, window.end - window.start <= HOUR)} – {timeLabel(window.end, 7, window.end - window.start <= HOUR)}
      </Text>
      {zoomed && <View style={s.row}>
        {button('‹ Pan earlier', () => applyWindow(panWindow(window, -(window.end - window.start) / 2, limits)), false, loading || window.start <= limits.start, 'Pan earlier')}
        {button('Pan later ›', () => applyWindow(panWindow(window, (window.end - window.start) / 2, limits)), false, loading || window.end >= limits.end, 'Pan later')}
      </View>}
      {result && <Text style={[s.source, { color: c.accent, backgroundColor: c.soft }]}>{result.demo ? 'ILLUSTRATIVE DEMO · not saved device history' : result.source === 'simulated' ? 'SIMULATED DATA · Windows BMV simulator' : 'RECORDED DATA · BMV'}</Text>}
      <View style={[s.card, { borderColor: c.line, backgroundColor: c.panel }]}>
        <Text accessibilityRole="header" style={[s.heading, { color: c.ink }]}>{info.title}</Text>
        {battery === 'house' && <View style={s.row}>{(['houseCharge', 'houseVoltage', 'houseCurrent'] as Metric[]).map(key => button(key === 'houseCharge' ? 'Charge' : metrics[key].title, () => setHouseMetric(key), metric === key))}</View>}
        {loading && <View style={s.loading}><ActivityIndicator color={c.accent} /><Text style={{ color: c.muted }}>Loading saved readings…</Text></View>}
        {error ? <View style={s.empty}><Text accessibilityRole="alert" style={{ color: c.amber }}>{error}</Text>{button('Retry', reload)}</View>
          : !bounds ? <View style={[s.empty, { minHeight: 250 }]}>{!loading && <><Text style={[s.heading, { color: c.ink }]}>No {battery === 'starter' ? 'starter voltage' : 'readings'} recorded</Text><Text style={{ color: c.muted }}>{zoomed ? 'Pan or zoom out to find recorded readings.' : 'Try another date or time range. Connect the simulator on Electrical to record new readings.'}</Text><HistoryChart points={points} metric={metric} window={window} limits={limits} selected={null} onSelect={setSelectedTime} onWindow={applyWindow} onActive={setGesturing} disabled={false} /></>}</View>
            : <>
              <Text style={[s.value, { color: c.ink }]}>{reading(selected?.[metric], info.digits, metric === 'houseCurrent')} <Text style={s.unit}>{info.unit}</Text></Text>
              <Text style={{ color: c.muted }}>Selected reading · {selected ? timeLabel(selected.receivedAt, days, window.end - window.start <= HOUR) : '—'}</Text>
              <HistoryChart points={points} metric={metric} window={window} limits={limits} selected={selected} onSelect={setSelectedTime} onWindow={applyWindow} onActive={setGesturing} disabled={loading} />
              <View style={s.row}>{button('Previous reading', () => setSelectedTime(selectedPoints[selectedIndex - 1].receivedAt), false, loading || selectedIndex <= 0)}{button('Next reading', () => setSelectedTime(selectedPoints[selectedIndex + 1].receivedAt), false, loading || selectedIndex < 0 || selectedIndex >= selectedPoints.length - 1)}</View>
            </>}
      </View>
      {bounds && !loading && !error && <>
        <Text style={{ color: c.muted, fontSize: 12, fontWeight: '700' }}>{zoomed ? 'VISIBLE RANGE' : 'RECORDED RANGE'}</Text>
        <View style={s.row}>{[['Lowest', bounds.low], ['Highest', bounds.high]].map(([label, value]) => <View key={label} style={[s.summary, { backgroundColor: c.panel, borderColor: c.line }]}><Text style={{ color: c.muted }}>{label}</Text><Text style={[s.heading, { color: c.ink }]}>{reading(value as number, info.digits, metric === 'houseCurrent')} {info.unit}</Text></View>)}</View>
        {battery === 'house' && selected && <View style={{ gap: 12 }}><Text style={[s.heading, { color: c.ink }]}>At selected time</Text><View style={s.row}><Text style={{ color: c.ink, flex: 1 }}>Voltage  {reading(selected.houseVoltage, 1)} V</Text><Text style={{ color: c.ink, flex: 1 }}>Current  {reading(selected.houseCurrent, 1, true)} A</Text></View><Text style={{ color: c.muted, fontSize: 12 }}>− Discharging / + Charging</Text></View>}
      </>}
      {battery === 'starter' && <View style={[s.card, { backgroundColor: c.soft, borderColor: c.line }]}><Text style={[s.heading, { color: c.ink }]}>Why voltage only?</Text><Text style={{ color: c.muted }}>The auxiliary input measures starter voltage. Starter charge and current are unavailable.</Text></View>}
      <Text style={{ color: c.muted, fontSize: 12 }}>Recorded history · not a live battery reading.{ '\n' }Only periods when the receiver was active are shown.</Text>
      {button('Refresh history', reload, false, loading)}
    </ScrollView>
  </SafeAreaView>;
}
const s = StyleSheet.create({
  safe: { flex: 1, backgroundColor: '#153b5e' }, page: { padding: 22, gap: 18, maxWidth: 650, width: '100%', alignSelf: 'center', paddingBottom: 36 },
  title: { fontSize: 30, fontWeight: '700' }, heading: { fontSize: 19, fontWeight: '600' }, row: { flexDirection: 'row', gap: 8 },
  button: { flex: 1, borderWidth: 1, borderRadius: 10, paddingHorizontal: 8, paddingVertical: 12, minHeight: 44, justifyContent: 'center' },
  source: { padding: 12, borderRadius: 9, fontSize: 12, fontWeight: '600' }, card: { borderWidth: 1, borderRadius: 16, padding: 16, gap: 15 },
  value: { fontSize: 38, fontWeight: '700', fontVariant: ['tabular-nums'] }, unit: { fontSize: 19, fontWeight: '400' },
  loading: { flexDirection: 'row', alignItems: 'center', gap: 10 },
  empty: { paddingVertical: 28, gap: 16 }, summary: { flex: 1, padding: 16, gap: 12, borderRadius: 12, borderWidth: 1 },
});
