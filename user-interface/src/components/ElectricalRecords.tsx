import { Link } from 'expo-router';
import { StatusBar } from 'expo-status-bar';
import { ScrollView, StyleSheet, Text, View } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { AppHeader } from './AppHeader';
import { usePreferences } from '../preferences/Preferences';

export function ElectricalRecords() {
  const { colors: c } = usePreferences();
  return <SafeAreaView edges={['top', 'bottom']} style={s.safe}>
    <StatusBar style="light" /><AppHeader />
    <ScrollView style={{ backgroundColor: c.background }} contentContainerStyle={s.page}>
      <View style={s.navigation}><Link href="/" style={[s.link, { color: c.accent }]}>← Home</Link></View>
      <Text style={[s.eyebrow, { color: c.muted }]}>ELECTRICAL</Text>
      <Text accessibilityRole="header" style={[s.title, { color: c.ink }]}>Maintenance Logs</Text>
      <Text style={[s.description, { color: c.muted }]}>Review service and maintenance records for your onboard electrical system.</Text>
      <View style={[s.empty, { backgroundColor: c.panel, borderColor: c.line }]}>
        <Text accessibilityRole="header" style={[s.emptyTitle, { color: c.ink }]}>No maintenance logs available</Text>
        <Text style={[s.description, { color: c.muted }]}>Maintenance recording is not implemented yet. There are no saved service records to display.</Text>
      </View>
    </ScrollView>
  </SafeAreaView>;
}
const s = StyleSheet.create({ safe: { flex: 1, backgroundColor: '#153b5e' }, page: { padding: 22, paddingVertical: 32, gap: 18, maxWidth: 1000, width: '100%', alignSelf: 'center' }, navigation: { flexDirection: 'row', flexWrap: 'wrap', justifyContent: 'space-between', gap: 16 }, link: { paddingVertical: 12, fontSize: 16 }, eyebrow: { fontSize: 11, letterSpacing: 1.6, marginTop: 20 }, title: { fontSize: 32, fontWeight: '700' }, description: { fontSize: 17, lineHeight: 26 }, empty: { padding: 26, gap: 14, borderRadius: 16, borderWidth: 1, marginTop: 12 }, emptyTitle: { fontSize: 22, fontWeight: '600' } });
