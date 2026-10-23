import { AppHeader } from './AppHeader';
import { usePreferences } from '../preferences/Preferences';
import { Link } from 'expo-router';
import { StatusBar } from 'expo-status-bar';
import { ScrollView, StyleSheet, Text, View } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { Feature, FeatureIcon } from './FeatureIcon';

export function ComingSoon({ feature, title, description }: { feature: Feature; title: string; description: string }) {
  const { colors: c } = usePreferences();
  const s = makeStyles(c);
  return <SafeAreaView style={[s.safe, { backgroundColor: '#153b5e' }]} edges={['top', 'bottom']}><StatusBar style="light" /><AppHeader /><ScrollView style={{ backgroundColor: c.background }} contentContainerStyle={s.page}>
    <Link href="/" style={s.back}>← Home</Link><View style={s.icon}><FeatureIcon feature={feature} /></View>
    <Text accessibilityRole="header" style={s.title}>{title}</Text><Text style={s.badge}>Coming next</Text><Text style={s.description}>{description}</Text>
  </ScrollView></SafeAreaView>;
}
const makeStyles = (c: ReturnType<typeof usePreferences>['colors']) => StyleSheet.create({ safe: { flex: 1, backgroundColor: c.background }, page: { padding: 24, gap: 24, maxWidth: 640, width: '100%', alignSelf: 'center' }, back: { color: c.accent, paddingVertical: 12, fontSize: 17 }, icon: { alignSelf: 'center', padding: 20, backgroundColor: '#def2fc', borderRadius: 100 }, title: { fontSize: 32, color: c.ink, fontWeight: '700', textAlign: 'center' }, badge: { color: c.muted, textAlign: 'center', fontSize: 16 }, description: { color: c.muted, fontSize: 18, lineHeight: 28, textAlign: 'center' } });
