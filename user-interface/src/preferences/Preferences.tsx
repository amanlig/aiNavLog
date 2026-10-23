import AsyncStorage from '@react-native-async-storage/async-storage';
import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';

const palettes = {
  white: { background: '#ffffff', panel: '#ffffff', ink: '#092443', muted: '#536780', line: '#dfe9f1', accent: '#16887f', amber: '#925d13', soft: '#e4f4f1' },
  black: { background: '#000000', panel: '#141a20', ink: '#f1f6fb', muted: '#b0bfce', line: '#34414f', accent: '#5dd5c7', amber: '#ffc775', soft: '#173c38' },
};
type Preferences = { boatName: string; background: 'white' | 'black' };
const defaults: Preferences = { boatName: 'My boat', background: 'white' };
const KEY = 'ainavlog.preferences.v1';
const Context = createContext({ ...defaults, colors: palettes.white, ready: false, loadError: '', save: async (_value: Preferences): Promise<void> => {} });
export function PreferencesProvider({ children }: { children: ReactNode }) {
  const [value, setValue] = useState(defaults);
  const [ready, setReady] = useState(false);
  const [loadError, setLoadError] = useState('');
  useEffect(() => {
    let active = true;
    AsyncStorage.getItem(KEY).then(raw => {
      if (!raw || !active) return;
      const stored = JSON.parse(raw);
      setValue({ boatName: typeof stored.boatName === 'string' && stored.boatName.trim() ? stored.boatName.trim().slice(0, 40) : defaults.boatName, background: stored.background === 'black' ? 'black' : 'white' });
    }).catch(() => { if (active) setLoadError('Saved settings could not be loaded. You can save them again.'); })
      .finally(() => { if (active) setReady(true); });
    return () => { active = false; };
  }, []);
  async function save(next: Preferences) {
    await AsyncStorage.setItem(KEY, JSON.stringify(next));
    setValue(next);
    setLoadError('');
  }
  return <Context.Provider value={{ ...value, colors: palettes[value.background], ready, loadError, save }}>{children}</Context.Provider>;
}
export const usePreferences = () => useContext(Context);
