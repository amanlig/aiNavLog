import { AppHeader } from '../../components/AppHeader';
import { Link } from 'expo-router';
import Svg, { Circle, Path } from 'react-native-svg';
import { ScrollView, StyleSheet, Text, useWindowDimensions, View } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { StatusBar } from 'expo-status-bar';
import { isFresh, reading } from '../../electrical/model';
import { useElectrical } from '../../electrical/use-electrical';

const C = { background: '#f0f6fb', panel: '#FFFFFF', ink: '#092443', muted: '#536780', line: '#e3edf5', accent: '#16887f', amber: '#925D13' };
function Metric({ label, value, unit }: { label: string; value: string; unit: string }) {
  return <View style={s.metric}><Text style={s.label}>{label}</Text><Text style={s.metricValue}>{value} <Text style={s.unit}>{unit}</Text></Text></View>;
}
export default function ElectricalScreen() {
  const wide = useWindowDimensions().width > 560;
  const { sample, now, error, live } = useElectrical();
  const fresh = isFresh(sample, now);
  const charge = fresh ? sample?.houseCharge : null;
  const current = fresh ? sample?.houseCurrent : null;
  const state = !fresh ? 'Waiting for fresh readings' : current == null ? 'Current unavailable' : current > 0 ? 'Charging · energy entering the house bank' : current < 0 ? 'Discharging · supplying onboard loads' : 'No net current';
  return <SafeAreaView style={s.safe} edges={['top', 'bottom']}><StatusBar style="light" /><AppHeader /><ScrollView style={s.scroll} contentContainerStyle={s.page} keyboardShouldPersistTaps="handled">
    <Link href="/" style={s.back}>← Home</Link>
    <View style={s.shell}>
      <View style={s.body}>
        <Text style={s.eyebrow}>ONBOARD ELECTRICAL</Text><Text accessibilityRole="header" style={[s.title, wide && s.wideTitle]}>Your power, at a glance.</Text><Text style={s.subtitle}>House and starter battery monitoring.</Text>
        <View style={[s.grid, wide && s.wideGrid]}>
          <View style={s.batteryCard}>
            <View style={s.brandRow}><Icon name="house" /><Text accessibilityRole="header" style={s.cardTitle}>House battery</Text></View><Text style={s.purpose}>Cabin lights, fridge & electronics</Text>
            <BatteryGauge charge={charge} label="House" />
            <View style={s.statusRow}><Icon name="arrow" size={14} /><Text style={s.direction}>{state}</Text></View>
            <View style={s.metrics}><Metric label="Voltage" value={reading(fresh ? sample?.houseVoltage : null, 1)} unit="V" /><Metric label="Amperage" value={reading(current, 1, true)} unit="A" /><Metric label="Charge" value={reading(charge, 1)} unit="%" /></View>
          </View>
          <View style={s.batteryCard}>
            <View style={s.brandRow}><Icon name="power" /><Text accessibilityRole="header" style={s.cardTitle}>Starter battery</Text></View><Text style={s.purpose}>Engine starting</Text>
            <BatteryGauge charge={null} label="Starter" />
            <View style={s.statusRow}><Text style={s.direction}>{fresh && sample?.starterVoltage != null ? 'Voltage monitoring · charge unavailable' : 'Waiting for starter voltage'}</Text></View>
            <View style={s.metrics}><Metric label="Voltage" value={reading(fresh ? sample?.starterVoltage : null, 1)} unit="V" /><Metric label="Amperage" value="—" unit="A" /><Metric label="Charge" value="—" unit="%" /></View>
          </View>
        </View>
        <View style={s.footRow}><Text style={s.note}>{live ? fresh ? sample?.source === 'simulated' ? 'Simulated BMV readings · device interface' : 'Live BMV readings' : 'Waiting for fresh BMV readings' : 'Sample readings · no live connection'}</Text><Text style={s.note}>− Discharging / + Charging</Text></View>
        {error && <Text style={s.warning}>{error}</Text>}
        {!fresh && sample && <Text style={s.warning}>Updates stopped. Latest readings are hidden until a fresh sample arrives.</Text>}
      </View>
    </View>
  </ScrollView></SafeAreaView>;
}
function Icon({ name, size = 18 }: { name: 'compass' | 'house' | 'power' | 'arrow' | 'zap'; size?: number }) {
  return <Svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={C.accent} strokeWidth={1.6} strokeLinecap="round" strokeLinejoin="round">
    {name === 'compass' ? <><Circle cx={12} cy={12} r={10} /><Path d="m16.2 7.8-2.8 5.6-5.6 2.8 2.8-5.6Z" /></> : name === 'house' ? <Path d="m3 10 9-7 9 7v10H3Zm6 10v-7h6v7" /> : name === 'power' ? <><Path d="M12 2v10M6 5a9 9 0 1 0 12 0" /></> : name === 'arrow' ? <Path d="M7 17 17 7M7 7h10v10" /> : <Path d="m13 2-9 12h7l-1 8 10-12h-7Z" />}
  </Svg>;
}
function BatteryGauge({ charge, label }: { charge: number | null | undefined; label: string }) {
  const available = charge != null && Number.isFinite(charge);
  return <View accessible accessibilityLabel={available ? `${label} battery: ${charge.toFixed(1)} percent charge` : `${label} battery charge unavailable`} style={s.gauge}>
    <View style={s.terminal} /><View style={s.well}><View style={[s.fill, { width: `${available ? Math.min(100, Math.max(0, charge)) : 0}%` }]} /></View>
    <View style={s.gaugeIcon}>{available ? <Icon name="zap" size={26} /> : <Text style={s.unavailable}>—</Text>}</View>
  </View>;
}
const s = StyleSheet.create({
  safe: { flex: 1, backgroundColor: '#153b5e' }, scroll: { backgroundColor: C.background }, page: { paddingHorizontal: 22, paddingVertical: 32, maxWidth: 1680, width: '100%', alignSelf: 'center', gap: 18 }, back: { color: C.accent, paddingVertical: 12, fontSize: 16 },
  shell: { backgroundColor: C.background }, header: { flexDirection: 'row', flexWrap: 'wrap', justifyContent: 'space-between', alignItems: 'center', gap: 12, paddingVertical: 20, paddingHorizontal: 26, backgroundColor: C.panel, borderBottomWidth: 1, borderBottomColor: C.line }, brandRow: { flexDirection: 'row', alignItems: 'center', gap: 9 }, brand: { color: C.ink, fontSize: 17, fontWeight: '600', letterSpacing: -0.5 }, badge: { color: C.muted, fontSize: 11, fontWeight: '600', letterSpacing: 1 },
  body: { gap: 18 }, eyebrow: { color: C.muted, letterSpacing: 1.8, fontSize: 11, fontWeight: '600', marginTop: 8 }, title: { color: C.ink, fontSize: 32, fontWeight: '700', letterSpacing: -0.8, textAlign: 'center' }, wideTitle: { fontSize: 48 }, subtitle: { color: C.muted, fontSize: 18, lineHeight: 27, textAlign: 'center' }, grid: { gap: 20, marginTop: 10 }, wideGrid: { flexDirection: 'row', gap: 28 }, batteryCard: { flex: 1, minWidth: 0, padding: 28, borderRadius: 12, backgroundColor: C.panel, borderWidth: 1, borderColor: C.line }, cardTitle: { fontSize: 26, fontWeight: '700', color: C.ink }, purpose: { fontSize: 12, color: C.muted, marginTop: 5, marginLeft: 27 },
  gauge: { width: 164, height: 102, borderWidth: 2, borderColor: C.line, borderRadius: 14, padding: 7, alignSelf: 'center', marginTop: 34, marginBottom: 22 }, terminal: { position: 'absolute', right: -9, top: 32, width: 6, height: 32, backgroundColor: C.line, borderTopRightRadius: 4, borderBottomRightRadius: 4 }, well: { flex: 1, backgroundColor: '#e7eeeb', borderRadius: 7, overflow: 'hidden' }, fill: { height: '100%', backgroundColor: C.accent, borderRadius: 6, opacity: 0.85 }, gaugeIcon: { position: 'absolute', top: 0, left: 0, right: 0, bottom: 0, alignItems: 'center', justifyContent: 'center' }, unavailable: { color: C.muted, fontSize: 28 }, statusRow: { flexDirection: 'row', alignItems: 'center', justifyContent: 'center', gap: 6, minHeight: 40, marginBottom: 16 }, direction: { color: C.muted, fontSize: 12, lineHeight: 20, flexShrink: 1 },
  metrics: { flexDirection: 'row', flexWrap: 'wrap', gap: 6, borderTopWidth: 1, borderTopColor: C.line, paddingTop: 20 }, metric: { flexGrow: 1, minWidth: 70, gap: 8 }, label: { color: C.muted, fontSize: 11 }, metricValue: { fontSize: 23, letterSpacing: -0.8, color: C.ink, fontVariant: ['tabular-nums'], fontWeight: '600' }, unit: { fontSize: 12, color: C.muted, fontWeight: '400', letterSpacing: 0 }, footRow: { flexDirection: 'row', justifyContent: 'space-between', flexWrap: 'wrap', gap: 10, paddingTop: 18 }, note: { fontSize: 14, lineHeight: 21, color: '#637c95' }, warning: { color: C.amber, fontSize: 13, lineHeight: 20 },

});
