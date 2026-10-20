import { Link } from 'expo-router';
import { Pressable, StyleSheet, Text, View } from 'react-native';
import { CompassIcon } from './FeatureIcon';

export function AppHeader() {
  return <View style={s.header}><Link href="/" asChild><Pressable accessibilityRole="link" accessibilityLabel="aiNavLog home" style={StyleSheet.flatten([s.link, s.brand])}><CompassIcon /><Text style={s.brandText}>aiNavLog</Text></Pressable></Link><Text style={s.boat}>My boat</Text></View>;
}
const s = StyleSheet.create({
  header: { backgroundColor: '#153b5e', paddingHorizontal: 24, paddingVertical: 18, minHeight: 85, flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', gap: 12 },
  link: { flexShrink: 1 }, brand: { flexDirection: 'row', alignItems: 'center', gap: 12, flexShrink: 1 }, brandText: { color: 'white', fontSize: 28, fontWeight: '700', letterSpacing: -1 }, boat: { color: 'white', fontSize: 16 },
});
