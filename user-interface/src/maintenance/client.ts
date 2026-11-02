import { desktopConfig, deviceEndpoint } from '../platform/desktop';
import { requireOptionalNativeModule } from 'expo';
import { Platform } from 'react-native';
import { type Entry, type Equipment, type Snapshot } from './model';
const native = Platform.OS==='web' ? null : requireOptionalNativeModule<{maintenance?(request: string): Promise<string>}>('AiNavLog');
async function command(request: object): Promise<unknown> {
  let result: unknown;
  if (Platform.OS!=='web') {
    if (!native?.maintenance) throw Error('Maintenance requires a newer aiNavLog native build.');
    result = JSON.parse(await native.maintenance(JSON.stringify(request)));
  } else {
    const endpoint = deviceEndpoint();
    if (!endpoint) throw Error('Open the installed aiNavLog app to use device-local maintenance records, or configure the local data service.');
    const url = new URL(endpoint);url.pathname='/maintenance';url.search='';
    const controller=new AbortController();const timeout=setTimeout(()=>controller.abort(),15000);
    try {
      const response=await fetch(url.toString(),{method:'POST',headers:{'Content-Type':'application/json',...(desktopConfig()?{'X-AiNavLog-Token':desktopConfig()!.token}:{})},body:JSON.stringify(request),signal:controller.signal});
      if (!response.ok) throw Error('Maintenance storage is unavailable. Check the local data service.');
      result=await response.json();
    } finally {clearTimeout(timeout);}
  }
  if (!result || typeof result!=='object') throw Error('Invalid maintenance response.');
  if ('error' in result) throw Error(String(result.error));
  return result;
}
export async function readMaintenance(): Promise<Snapshot> {
  const result=await command({op:'list'}) as Snapshot;
  if (!Array.isArray(result.equipment) || !Array.isArray(result.entries)) throw Error('Invalid maintenance history.');
  return result;
}
export async function saveService(entry: Entry) {await command({op:'save',entry});}
export async function addEquipment(equipment: Equipment) {await command({op:'equipment',equipment});}
