import { useState } from 'react';
import { AppHeader } from '../components/AppHeader';
import { Link } from 'expo-router';
import { StatusBar } from 'expo-status-bar';
import { Pressable, ScrollView, StyleSheet, Text, useWindowDimensions, View } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { Feature, FeatureIcon } from '../components/FeatureIcon';
import { usePreferences } from '../preferences/Preferences';
import { useElectrical } from '../electrical/use-electrical';
import { isFresh, reading } from '../electrical/model';

const features: { id: Feature; title: string; href: '/elec' | '/mech' | '/rout' }[] = [
  { id: 'electrical', title: 'Electrical', href: '/elec' },
  { id: 'engine', title: 'Engine', href: '/mech' },
  { id: 'routes', title: 'Routes', href: '/rout' },
];
export default function HomeScreen() {
  const wide = useWindowDimensions().width > 900;
  const { colors: c } = usePreferences();
  const { sample, now, error, live } = useElectrical();
  const [battery, setBattery] = useState<'house' | 'starter'>('house');
  const house = battery === 'house';
  const fresh = isFresh(sample, now);
  const current = fresh && house ? sample?.houseCurrent : null;
  const charge = fresh && house ? sample?.houseCharge : null;
  const state = !fresh ? 'Waiting for fresh readings' : !house ? 'Voltage monitoring only' : current == null ? 'Current unavailable' : current > 0 ? '↗ Charging' : current < 0 ? '↘ Discharging' : 'No net current';
  return <SafeAreaView style={s.safe} edges={['top', 'bottom']}>
    <StatusBar style="light" /><AppHeader />
    <ScrollView contentContainerStyle={[s.scroll, { backgroundColor: c.background }]}>
      <View style={s.main}>
        <Text accessibilityRole="header" style={[s.title, { color: c.ink }]}>Your boat at a glance</Text>
        <Text style={[s.subtitle, { color: c.muted }]}>Key readings here. Open a card for the full picture.</Text>
        <View style={[s.cards, wide && s.wideCards]}>
          {features.map(feature => <View key={feature.id} style={[s.card, { backgroundColor: c.panel, borderColor: c.line }, wide && s.wideCard]}>
              <View style={s.row}><View style={[s.icon, { backgroundColor: '#def5f5' }]}><FeatureIcon feature={feature.id} size={28} /></View><Link href={feature.href} accessibilityLabel={`View ${feature.title}`} style={[s.cardTitle, { color: c.ink }]}>{feature.title}</Link></View>
              {feature.id === 'electrical' ? <>
                <Text style={[s.source, { color: c.muted }]}>{!live ? 'Sample readings · no live connection' : fresh ? sample?.source === 'simulated' ? 'Simulated BMV readings' : 'Live BMV readings' : 'Waiting for BMV readings'}</Text>
                <View accessibilityRole="tablist" style={s.row}>{(['house', 'starter'] as const).map(option => <Pressable key={option} accessibilityRole="tab" accessibilityState={{ selected: battery === option }} aria-selected={battery === option} onPress={() => setBattery(option)} style={{ minHeight: 44, paddingVertical: 12, borderBottomWidth: 2, borderBottomColor: battery === option ? c.accent : 'transparent' }}><Text style={{ color: battery === option ? c.accent : c.muted, fontWeight: battery === option ? '700' : '400' }}>{option === 'house' ? 'House battery' : 'Starter battery'}</Text></Pressable>)}</View>
                <View style={s.row}><Text style={[s.charge, { color: c.ink }]}>{reading(charge, 1)}{house && <Text style={s.unit}> %</Text>}</Text><Text style={{ color: c.accent, flexShrink: 1 }}>{state}</Text></View>
                {house && <View style={[s.track, { backgroundColor: c.line }]}><View style={{ height: '100%', width: `${charge == null ? 0 : Math.max(0, Math.min(100, charge))}%`, backgroundColor: c.accent }} /></View>}
                <View style={s.metrics}><View><Text style={{ color: c.muted }}>Voltage</Text><Text style={[s.value, { color: c.ink }]}>{reading(fresh ? house ? sample?.houseVoltage : sample?.starterVoltage : null, 1)} V</Text></View><View><Text style={{ color: c.muted }}>Current</Text><Text style={[s.value, { color: c.ink }]}>{reading(current, 1, true)} A</Text></View></View>
                <Text style={{ color: c.muted, fontSize: 12 }}>{house ? 'House bank · charge, voltage and signed current' : 'Starter battery · current and charge unavailable'}</Text>
                {!!error && <Text style={{ color: c.amber }}>{error}</Text>}
              </> : <View style={s.placeholder}><Text style={[s.cardTitle, { color: c.ink }]}>{feature.id === 'engine' ? 'Not connected' : 'No tracking active'}</Text><Text style={[s.description, { color: c.muted }]}>{feature.id === 'engine' ? 'Engine hours, checks and maintenance are coming next.' : 'Trip recording, destinations and time underway are coming next.'}</Text><Text style={[s.label, { color: c.accent }]}>COMING SOON</Text></View>}
              <View style={[s.cardLinks, { borderColor: c.line }]}>{feature.id === 'electrical' ? <>
                <Link href="/elec/history" style={[s.link, { color: c.accent }]}>Historical Usage</Link>
                <Link href="/elec/maintenance" style={[s.link, { color: c.accent }]}>Maintenance Logs</Link>
              </> : <Link href={feature.href} style={[s.link, { color: c.accent }]}>View {feature.title.toLowerCase()} →</Link>}</View>
          </View>)}
        </View>
        <Text style={[s.footer, { color: c.muted }]}>Concept preview · Engine and Routes are not connected.</Text>
      </View>
    </ScrollView>
  </SafeAreaView>;
}
const s = StyleSheet.create({
  safe: { flex: 1, backgroundColor: '#153b5e' }, scroll: { flexGrow: 1 }, main: { width: '100%', maxWidth: 1440, alignSelf: 'center', padding: 22, paddingVertical: 32 }, title: { fontSize: 32, fontWeight: '700', letterSpacing: -0.8 }, subtitle: { fontSize: 17, lineHeight: 25, marginTop: 12, marginBottom: 28 }, cards: { gap: 20 }, wideCards: { flexDirection: 'row', alignItems: 'stretch' }, card: { width: '100%', borderWidth: 1, borderRadius: 16, padding: 22, gap: 14 }, wideCard: { flexGrow: 1, flexShrink: 1, flexBasis: 0, width: 0, minWidth: 0 }, row: { flexDirection: 'row', alignItems: 'center', gap: 14, flexWrap: 'wrap' }, icon: { width: 40, height: 40, borderRadius: 10, alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }, cardTitle: { fontSize: 23, fontWeight: '700' }, source: { fontSize: 12 }, label: { fontSize: 11, fontWeight: '600', letterSpacing: 1 }, charge: { fontSize: 40, fontWeight: '700', fontVariant: ['tabular-nums'] }, unit: { fontSize: 19, fontWeight: '400' }, track: { height: 9, borderRadius: 5, overflow: 'hidden' }, metrics: { flexDirection: 'row', justifyContent: 'space-between', gap: 18, flexWrap: 'wrap' }, value: { fontSize: 24, marginTop: 6, fontWeight: '600', fontVariant: ['tabular-nums'] }, starter: { borderTopWidth: 1, paddingTop: 12, flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between' }, placeholder: { gap: 24, flexGrow: 1, paddingVertical: 18 }, description: { fontSize: 16, lineHeight: 25 }, cardLinks: { borderTopWidth: 1, paddingTop: 8, marginTop: 'auto', flexDirection: 'row', flexWrap: 'wrap', justifyContent: 'space-between', columnGap: 12, rowGap: 4 }, link: { paddingVertical: 12, minHeight: 44, fontSize: 13, fontWeight: '600' }, footer: { marginTop: 28, fontSize: 13 },
});
