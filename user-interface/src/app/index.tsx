import { AppHeader } from '../components/AppHeader';
import { Link } from 'expo-router';
import { StatusBar } from 'expo-status-bar';
import { Pressable, ScrollView, StyleSheet, Text, useWindowDimensions, View } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { Feature, FeatureIcon } from '../components/FeatureIcon';

const features: { id: Feature; title: string; description: string; href: '/elec' | '/mech' | '/rout'; color: string }[] = [
  { id: 'electrical', title: 'Electrical', description: 'Track batteries, charging and power use.', href: '/elec', color: '#def5f5' },
  { id: 'engine', title: 'Engine', description: 'Track engine hours, checks and maintenance.', href: '/mech', color: '#def2fc' },
  { id: 'routes', title: 'Routes', description: 'Record trips, destinations and time underway.', href: '/rout', color: '#ddf5ed' },
];
export default function HomeScreen() {
  const wide = useWindowDimensions().width > 760;
  return <SafeAreaView style={s.safe} edges={['top', 'bottom']}>
    <StatusBar style="light" />
    <AppHeader />
    <ScrollView contentContainerStyle={s.scroll}>
      <View style={s.main}>
        <View style={s.intro}><Text accessibilityRole="header" style={[s.title, wide && s.wideTitle]}>Your boat at a glance</Text><Text style={s.subtitle}>Keep your onboard systems and journeys in one place.</Text></View>
        <View style={[s.cards, wide && s.wideCards]}>
          {features.map(feature => <Link key={feature.id} href={feature.href} asChild>
            <Pressable accessibilityRole="link" accessibilityLabel={`${feature.title}. ${feature.description}`} style={({ pressed }) => [s.card, wide ? s.wideCard : s.phoneCard, pressed && s.pressed]}>
              <View style={[s.icon, { backgroundColor: feature.color }]}><FeatureIcon feature={feature.id} /></View>
              <Text accessibilityRole="header" style={s.cardTitle}>{feature.title}</Text><Text style={s.description}>{feature.description}</Text>
            </Pressable>
          </Link>)}
        </View>
      </View>
      <View style={[s.footer, wide && s.wideFooter]}><Text style={s.footerText}>Concept preview</Text><Text style={s.footerText}>Electrical uses demo data. Engine and routes are coming next.</Text></View>
    </ScrollView>
  </SafeAreaView>;
}
const s = StyleSheet.create({
  safe: { flex: 1, backgroundColor: '#153b5e' },
  header: { backgroundColor: '#153b5e', paddingHorizontal: 24, paddingVertical: 18, minHeight: 85, flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', gap: 12 },
  brand: { flexDirection: 'row', alignItems: 'center', gap: 12, flexShrink: 1 }, brandText: { color: 'white', fontSize: 28, fontWeight: '700', letterSpacing: -1 }, boat: { color: 'white', fontSize: 16 },
  scroll: { flexGrow: 1, backgroundColor: '#f0f6fb' }, main: { width: '100%', maxWidth: 1680, alignSelf: 'center', paddingHorizontal: 22, paddingVertical: 32, flexGrow: 1, justifyContent: 'center' },
  intro: { alignItems: 'center', marginBottom: 28, gap: 18 }, title: { color: '#092443', fontSize: 32, fontWeight: '700', letterSpacing: -0.8, textAlign: 'center' }, wideTitle: { fontSize: 48 }, subtitle: { color: '#536780', fontSize: 18, lineHeight: 27, textAlign: 'center' },
  cards: { gap: 20, alignItems: 'center' }, wideCards: { flexDirection: 'row', alignItems: 'stretch', gap: 28 }, card: { backgroundColor: 'white', borderWidth: 1, borderColor: '#e3edf5', borderRadius: 12, padding: 28, alignItems: 'center' }, phoneCard: { width: '100%', maxWidth: 470 }, wideCard: { flex: 1, paddingVertical: 34 }, pressed: { borderColor: '#16887f', opacity: 0.8 },
  icon: { width: 155, height: 155, borderRadius: 78, alignItems: 'center', justifyContent: 'center', marginBottom: 22 }, cardTitle: { color: '#092443', fontSize: 26, fontWeight: '700', letterSpacing: -0.6, marginBottom: 15 }, description: { maxWidth: 330, color: '#536780', fontSize: 18, lineHeight: 27, textAlign: 'center' },
  footer: { paddingHorizontal: 22, paddingTop: 18, paddingBottom: 24, gap: 14 }, wideFooter: { flexDirection: 'row', justifyContent: 'space-between' }, footerText: { color: '#637c95', fontSize: 14 },
});
