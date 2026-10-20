import { Link } from 'expo-router';
import { StatusBar } from 'expo-status-bar';
import { ScrollView, StyleSheet, Text, View } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { Feature, FeatureIcon } from './FeatureIcon';

export function ComingSoon({ feature, title, description }: { feature: Feature; title: string; description: string }) {
  return <SafeAreaView style={s.safe}><StatusBar style="dark" /><ScrollView contentContainerStyle={s.page}>
    <Link href="/" style={s.back}>← Home</Link><View style={s.icon}><FeatureIcon feature={feature} /></View>
    <Text accessibilityRole="header" style={s.title}>{title}</Text><Text style={s.badge}>Coming next</Text><Text style={s.description}>{description}</Text>
  </ScrollView></SafeAreaView>;
}
const s = StyleSheet.create({ safe: { flex: 1, backgroundColor: '#f0f6fb' }, page: { padding: 24, gap: 24, maxWidth: 640, width: '100%', alignSelf: 'center' }, back: { color: '#16887f', paddingVertical: 12, fontSize: 17 }, icon: { alignSelf: 'center', padding: 20, backgroundColor: '#def2fc', borderRadius: 100 }, title: { fontSize: 32, color: '#092443', fontWeight: '700', textAlign: 'center' }, badge: { color: '#536780', textAlign: 'center', fontSize: 16 }, description: { color: '#536780', fontSize: 18, lineHeight: 28, textAlign: 'center' } });
