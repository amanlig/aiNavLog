import { Link } from 'expo-router';
import { useState } from 'react';
import { Modal, Pressable, ScrollView, StyleSheet, Text, TextInput, View } from 'react-native';
import { CompassIcon } from './FeatureIcon';
import { usePreferences } from '../preferences/Preferences';

export function AppHeader() {
  const { boatName, background, colors: c, ready, loadError, save } = usePreferences();
  const [dialog, setDialog] = useState<'boat' | 'settings' | null>(null);
  const [name, setName] = useState('');
  const [appearance, setAppearance] = useState(background);
  const [error, setError] = useState('');
  const [saving, setSaving] = useState(false);
  function open(which: 'boat' | 'settings') { setName(boatName); setAppearance(background); setError(''); setDialog(which); }
  async function submit() {
    if (!name.trim()) { setError('Enter a boat name.'); return; }
    setSaving(true);
    try { await save({ boatName: name.trim(), background: appearance }); setDialog(null); }
    catch { setError('Settings could not be saved. Please try again.'); }
    finally { setSaving(false); }
  }
  return <>
    <View style={s.header}>
      <Link href="/" asChild><Pressable accessibilityRole="link" accessibilityLabel="aiNavLog home" style={s.brand}><CompassIcon /><Text style={s.brandText}>aiNavLog</Text></Pressable></Link>
      <View style={s.actions}>
        <Pressable disabled={!ready} onPress={() => open('boat')} accessibilityRole="button" accessibilityLabel={`Boat profile: ${boatName}`} style={s.headerButton}><Text numberOfLines={1} style={s.boat}>{boatName} ▾</Text></Pressable>
      </View>
    </View>
    <Modal visible={dialog !== null} transparent animationType="fade" onRequestClose={() => { if (!saving) setDialog(null); }}>
      <View style={s.overlay}><View accessibilityViewIsModal style={[s.dialog, { backgroundColor: c.panel, borderColor: c.line }]}>
        <ScrollView keyboardShouldPersistTaps="handled" contentContainerStyle={{ gap: 18 }}>
          <Text accessibilityRole="header" style={[s.title, { color: c.ink }]}>{dialog === 'boat' ? 'Boat profile' : 'Settings'}</Text>
          <Text nativeID="boat-name-label" style={{ color: c.muted }}>Boat name</Text>
          <TextInput accessibilityLabel="Boat name" accessibilityLabelledBy="boat-name-label" value={name} onChangeText={setName} maxLength={40} editable={!saving} autoFocus={dialog === 'boat'} autoCapitalize="words" placeholder="My boat" placeholderTextColor={c.muted} style={[s.input, { color: c.ink, borderColor: c.line }]} />
          {dialog === 'boat' && <Pressable disabled={saving} accessibilityRole="button" onPress={() => setDialog('settings')} style={[s.input, { borderColor: c.line }]}><Text style={{ color: c.accent, fontWeight: '600' }}>Settings →</Text><Text style={{ color: c.muted, marginTop: 6 }}>Choose a white or black background</Text></Pressable>}
          {dialog === 'settings' && <><Text style={{ color: c.muted }}>Background</Text><View style={s.options}>{(['white', 'black'] as const).map(option => <Pressable key={option} disabled={saving} accessibilityRole="radio" aria-checked={appearance === option} accessibilityState={{ checked: appearance === option }} onPress={() => setAppearance(option)} style={[s.option, { borderColor: appearance === option ? c.accent : c.line, backgroundColor: appearance === option ? c.soft : c.panel }]}><Text style={{ color: c.ink }}>{appearance === option ? '● ' : '○ '}{option === 'white' ? 'White' : 'Black'}</Text></Pressable>)}</View></>}
          {!!(error || loadError) && <Text accessibilityRole="alert" style={{ color: c.amber }}>{error || loadError}</Text>}
          <View style={s.options}><Pressable disabled={saving} onPress={() => setDialog(null)} style={s.option} accessibilityRole="button"><Text style={{ color: c.muted }}>Cancel</Text></Pressable><Pressable disabled={saving} onPress={() => { void submit(); }} style={[s.option, { backgroundColor: c.soft }]} accessibilityRole="button"><Text style={{ color: c.accent, fontWeight: '700' }}>{saving ? 'Saving…' : 'Save'}</Text></Pressable></View>
        </ScrollView>
      </View></View>
    </Modal>
  </>;
}
const s = StyleSheet.create({
  header: { backgroundColor: '#153b5e', paddingHorizontal: 22, paddingVertical: 16, minHeight: 85, flexDirection: 'row', flexWrap: 'wrap', alignItems: 'center', justifyContent: 'space-between', gap: 12 },
  brand: { flexDirection: 'row', alignItems: 'center', gap: 8 }, brandText: { color: 'white', fontSize: 25, fontWeight: '700', letterSpacing: -1 }, boat: { color: 'white', fontSize: 15 },
  actions: { flexDirection: 'row', alignItems: 'center', gap: 12, flexShrink: 1 }, headerButton: { minHeight: 44, justifyContent: 'center', maxWidth: 200 },
  overlay: { flex: 1, backgroundColor: '#00000099', justifyContent: 'center', alignItems: 'center', padding: 24 }, dialog: { padding: 24, borderRadius: 18, borderWidth: 1, width: '100%', maxWidth: 440, maxHeight: '90%' }, title: { fontSize: 26, fontWeight: '700' }, input: { borderWidth: 1, borderRadius: 10, padding: 14, fontSize: 18 }, options: { flexDirection: 'row', gap: 12 }, option: { flex: 1, alignItems: 'center', padding: 15, borderRadius: 10, borderWidth: 1, borderColor: 'transparent', minHeight: 48 },
});
