import { PreferencesProvider } from '../preferences/Preferences';
import { Stack } from 'expo-router';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { ElectricalProvider } from '../electrical/use-electrical';
export default function Layout() {
  return <SafeAreaProvider><PreferencesProvider><ElectricalProvider><Stack screenOptions={{ headerShown: false }} /></ElectricalProvider></PreferencesProvider></SafeAreaProvider>;
}
