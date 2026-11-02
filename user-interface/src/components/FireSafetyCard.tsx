import { useState } from 'react';
import { Image, Modal, Pressable, ScrollView, StyleSheet, Text, useWindowDimensions, View, type StyleProp, type ViewStyle } from 'react-native';
import Svg, { Path } from 'react-native-svg';
import { usePreferences } from '../preferences/Preferences';

const sampleExtinguishers = [
  { id: '1', left: '23%', top: '45%', color: '#16834a', condition: 'Good / healthy' },
  { id: '2', left: '47%', top: '73%', color: '#c52f35', condition: 'Expired' },
  { id: '3', left: '72%', top: '35%', color: '#c85a08', condition: 'Nearing end of life' },
] as const;

function BoatDiagram({ enlarged = false }: { enlarged?: boolean }) {
  return <View style={s.diagram}>
    <Image source={require('../../assets/boat-layout.png')} resizeMode="contain" accessibilityLabel="Illustrative top-down boat layout with three sample extinguisher markers. Conditions are listed below." style={s.image} />
    {sampleExtinguishers.map(item => <View key={item.id} accessible accessibilityLabel={`Sample extinguisher ${item.id}: ${item.condition}`} style={[s.marker, enlarged && s.largeMarker, { left: item.left, top: item.top, backgroundColor: item.color }]}>
      <Text style={s.markerNumber}>{item.id}</Text>
    </View>)}
  </View>;
}
function ConditionLegend() {
  const { colors: c } = usePreferences();
  return <View style={s.legend}>{sampleExtinguishers.map(item => <View key={item.id} style={s.legendItem}>
    <View style={[s.legendDot, { backgroundColor: item.color }]}><Text style={s.legendNumber}>{item.id}</Text></View>
    <Text style={[s.legendText, { color: c.ink }]}>{item.condition}</Text>
  </View>)}</View>;
}

/** Home-page preview; extinguisher locations and inspection records are planned. */
export function FireSafetyCard({ style }: { style?: StyleProp<ViewStyle> }) {
  const { colors: c } = usePreferences();
  const { width } = useWindowDimensions();
  const [hovered, setHovered] = useState(false);
  const [opened, setOpened] = useState(false);
  return <View testID="fire-safety-card" style={[s.card, { backgroundColor: c.panel, borderColor: c.line, zIndex: hovered ? 10 : 0 }, style]}>
    <View style={s.heading}>
      <View style={s.icon}>
        <Svg width={28} height={28} viewBox="0 0 32 32" fill="none" stroke="#a43d21" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round">
          <Path d="M17 3c1 7-7 8-5 14 1-2 3-3 4-5 0 4 5 5 5 9a5 5 0 0 1-10 0M17 3c2 5 9 10 9 17a10 10 0 0 1-20 0c0-4 2-7 5-10-1 4 0 5 1 7" />
        </Svg>
      </View>
      <Text accessibilityRole="header" style={[s.title, { color: c.ink }]}>Fire Safety</Text>
    </View>
    <Text style={[s.description, { color: c.muted }]}>Your boat layout, fire extinguisher locations and condition.</Text>
    <Pressable accessibilityRole="button" accessibilityLabel="Enlarge fire safety boat layout"
      onHoverIn={() => setHovered(true)} onHoverOut={() => setHovered(false)}
      onFocus={() => setHovered(true)} onBlur={() => setHovered(false)}
      onPress={() => { setHovered(false); setOpened(true); }}
      style={[s.layout, { borderColor: c.line }]}>
      <BoatDiagram />
    </Pressable>
    <Text style={[s.note, { color: c.muted }]}>Hover or tap the boat to enlarge</Text>
    {hovered && !opened && <View testID="fire-safety-hover-preview" pointerEvents="none" style={[s.preview, { width: Math.min(width - 44, 900), backgroundColor: c.panel, borderColor: c.line }]}>
      <View style={[s.layout, { borderColor: c.line }]}><BoatDiagram enlarged /></View>
      <ConditionLegend />
      <Text style={[s.note, { color: c.muted }]}>Sample locations and conditions</Text>
    </View>}
    <Text style={[s.note, { color: c.muted }]}>Sample locations and conditions · illustrative layout</Text>
    <View style={[s.status, { borderColor: c.line }]}>
      <Text style={[s.statusLabel, { color: c.ink }]}>Extinguisher condition</Text>
      <Text style={[s.statusValue, { color: c.muted }]}>Sample data</Text>
    </View>
    <ConditionLegend />
    <Text style={[s.note, { color: c.muted }]}>Inspection records and editable locations are coming next.</Text>
    <Modal visible={opened} transparent animationType="fade" onRequestClose={() => setOpened(false)}>
      <View style={s.backdrop}>
        <ScrollView style={[s.modal, { backgroundColor: c.panel }]} contentContainerStyle={s.modalContent}>
          <View style={s.modalHeader}>
            <Text accessibilityRole="header" style={[s.title, { color: c.ink }]}>Fire Safety</Text>
            <Pressable accessibilityRole="button" accessibilityLabel="Close boat layout" onPress={() => setOpened(false)} style={s.close}><Text style={{ color: c.accent, fontWeight: '700' }}>Close</Text></Pressable>
          </View>
          <View style={[s.layout, { borderColor: c.line }]}><BoatDiagram enlarged /></View>
          <ConditionLegend />
          <Text style={[s.note, { color: c.muted }]}>Sample locations and conditions · illustrative layout</Text>
        </ScrollView>
      </View>
    </Modal>
  </View>;
}

const s = StyleSheet.create({
  card: { borderWidth: 1, borderRadius: 16, padding: 22, gap: 14, width: '100%' },
  heading: { flexDirection: 'row', flexWrap: 'wrap', alignItems: 'center', gap: 14 },
  icon: { width: 40, height: 40, backgroundColor: '#fff0e9', borderRadius: 10, alignItems: 'center', justifyContent: 'center' },
  title: { fontSize: 23, fontWeight: '700' },
  description: { fontSize: 16, lineHeight: 25 },
  layout: { backgroundColor: '#fff', borderWidth: 1, borderRadius: 12, padding: 12 },
  diagram: { width: '100%', aspectRatio: 2161 / 728, maxWidth: 790, alignSelf: 'center' },
  image: { width: '100%', height: '100%' },
  marker: { position: 'absolute', width: 30, height: 30, borderRadius: 15, borderWidth: 3, borderColor: '#18242f', transform: [{ translateX: -15 }, { translateY: -15 }], alignItems: 'center', justifyContent: 'center' },
  largeMarker: { width: 40, height: 40, borderRadius: 20, transform: [{ translateX: -20 }, { translateY: -20 }] },
  markerNumber: { color: '#fff', fontSize: 14, fontWeight: '800' },
  legend: { flexDirection: 'row', flexWrap: 'wrap', gap: 14 },
  legendItem: { flexDirection: 'row', alignItems: 'center', gap: 8 },
  legendDot: { width: 24, height: 24, borderRadius: 12, borderWidth: 2, borderColor: '#18242f', alignItems: 'center', justifyContent: 'center' },
  legendNumber: { color: '#fff', fontSize: 12, fontWeight: '800' },
  legendText: { fontSize: 14 },
  status: { borderTopWidth: 1, paddingTop: 16, flexDirection: 'row', flexWrap: 'wrap', justifyContent: 'space-between', gap: 10 },
  statusLabel: { fontSize: 15, fontWeight: '600' },
  statusValue: { fontSize: 15 },
  preview: { position: 'absolute', top: 105, right: 0, zIndex: 20, borderWidth: 2, borderRadius: 14, padding: 18, gap: 14, boxShadow: '0 12px 32px rgba(0,0,0,0.22)' },
  backdrop: { flex: 1, justifyContent: 'center', alignItems: 'center', padding: 22, backgroundColor: 'rgba(0,0,0,0.55)' },
  modal: { width: '100%', maxWidth: 1000, maxHeight: '90%', flexGrow: 0, borderRadius: 16 },
  modalContent: { padding: 20, gap: 18 },
  modalHeader: { flexDirection: 'row', justifyContent: 'space-between', alignItems: 'center' },
  close: { minHeight: 44, minWidth: 60, alignItems: 'center', justifyContent: 'center' },
  note: { fontSize: 13, lineHeight: 20 },
});
