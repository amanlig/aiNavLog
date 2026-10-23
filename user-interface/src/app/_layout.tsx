import { PreferencesProvider } from '../preferences/Preferences';
import { Stack } from 'expo-router';
import { SafeAreaProvider } from 'react-native-safe-area-context';
export default function Layout() {
  return <SafeAreaProvider><PreferencesProvider><Stack screenOptions={{ headerShown: false }} /></PreferencesProvider></SafeAreaProvider>;
}
