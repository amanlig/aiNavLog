import { useEffect, useState } from 'react';
import { StatusBar } from 'expo-status-bar';
import { Pressable, ScrollView, StyleSheet, Text, TextInput, View } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { AppHeader } from './AppHeader';
import { usePreferences } from '../preferences/Preferences';
import { addEquipment, readMaintenance, saveService } from '../maintenance/client';
import { costMinor, inSystem, recordSystem, localDate, newId, optionalNumber, validate, type Entry, type Snapshot } from '../maintenance/model';

const kinds = ['Service','Inspection','Repair','Replacement'] as const;
type Form = {id:string;equipmentId:string;date:string;kind:Entry['kind'];description:string;provider:string;hours:string;parts:string;cost:string;currency:string;reference:string;notes:string;nextDate:string;nextHours:string};
const blank = ():Form => ({id:newId(),equipmentId:'',date:localDate(),kind:'Service',description:'',provider:'',hours:'',parts:'',cost:'',currency:'CAD',reference:'',notes:'',nextDate:'',nextHours:''});
export function MaintenanceRecords({ system }: {system:'engine'|'electrical'}) {
  const {colors:c}=usePreferences();
  const [data,setData]=useState<Snapshot>({equipment:[],entries:[]});
  const [showLegacy,setShowLegacy]=useState(false);
  const [form,setForm]=useState<Form>(blank);
  const [editing,setEditing]=useState(false);
  const [busy,setBusy]=useState(false);
  const [loaded,setLoaded]=useState(false);
  const [error,setError]=useState('');
  const [notice,setNotice]=useState('');
  const [equipmentName,setEquipmentName]=useState('');
  const [equipmentId,setEquipmentId]=useState(newId);
  const [adding,setAdding]=useState(false);
  const [equipmentFilter,setEquipmentFilter]=useState('');
  useEffect(()=>{let active=true;readMaintenance().then(result=>{if(active){setData(result);setLoaded(true);}}).catch(e=>{if(active)setError(String(e.message||e));});return()=>{active=false;};},[]);
  async function refresh() {setBusy(true);setError('');try {setData(await readMaintenance());setLoaded(true);}catch(e){setError(e instanceof Error?e.message:'Could not load records.');}finally{setBusy(false);}}
  const visibleEquipment=data.equipment.filter(e=>inSystem(e,system));
  const matchingEntries=data.entries.filter(e=>visibleEquipment.some(item=>item.id===e.equipmentId) && (!equipmentFilter || e.equipmentId===equipmentFilter));
  const scopeOf=(entry: Entry)=>recordSystem(entry,data.equipment.find(e=>e.id===entry.equipmentId)!);
  const currentEntries=matchingEntries.filter(e=>scopeOf(e)===system);
  const legacyEntries=matchingEntries.filter(e=>scopeOf(e)==='legacy');
  const entries=[...currentEntries,...(showLegacy?legacyEntries:[])];
  const patch=(key:keyof Form,value:string)=>setForm(old=>({...old,[key]:value}));
  async function save() {
    setError('');setNotice('');
    try {
      const entry:Entry={...form,system,hours:optionalNumber(form.hours,'Equipment hours'),costMinor:costMinor(form.cost),currency:form.currency.trim().toUpperCase(),nextDate:form.nextDate.trim()||null,nextHours:optionalNumber(form.nextHours,'Next service hours')};
      // Form-only cost is not part of the strict storage contract.
      const {cost: _cost,...payload}=entry as Entry & {cost?:string};
      validate(payload);setBusy(true);
      await saveService(payload);
      setEditing(false);setForm(blank());setEquipmentFilter('');
      setNotice('Service record saved on this device.');
      try {setData(await readMaintenance());} catch {setError('Record saved, but history could not refresh. Tap Refresh records.');}
    } catch(e) {setError(e instanceof Error?e.message:'Could not save. Your entry is still here.');}
    finally {setBusy(false);}
  }
  async function createEquipment() {
    if (!equipmentName.trim()) {setError('Enter an equipment name.');return;}
    setBusy(true);setError('');
    try {await addEquipment({id:equipmentId,name:equipmentName.trim(),system});const result=await readMaintenance();setData(result);patch('equipmentId',equipmentId);setEquipmentId(newId());setEquipmentName('');setAdding(false);}
    catch(e){setError(e instanceof Error?e.message:'Could not add equipment.');}finally{setBusy(false);}
  }
  function button(label:string,onPress:()=>void,selected=false) {return <Pressable key={label} accessibilityRole="button" accessibilityState={{selected,disabled:busy}} disabled={busy} onPress={onPress} style={[s.button,{borderColor:selected?c.accent:c.line,backgroundColor:selected?c.soft:c.panel,opacity:busy?.5:1}]}><Text style={{color:c.accent,fontWeight:'600'}}>{label}</Text></Pressable>;}
  function field(label:string,key:keyof Form,options:{multiline?:boolean;numeric?:boolean;max?:number}={}) {return <View style={s.field}><Text style={[s.label,{color:c.ink}]}>{label}</Text><TextInput accessibilityLabel={label} editable={!busy} value={form[key]} onChangeText={text=>patch(key,text)} multiline={options.multiline} keyboardType={options.numeric?'decimal-pad':'default'} autoCapitalize={key==='currency'?'characters':'sentences'} maxLength={options.max??200} placeholderTextColor={c.muted} style={[s.input,{color:c.ink,borderColor:c.line,backgroundColor:c.background},options.multiline&&s.multiline]} /></View>;}
  return <SafeAreaView edges={['top','bottom']} style={s.safe}><StatusBar style="light"/><AppHeader/><ScrollView keyboardShouldPersistTaps="handled" style={{backgroundColor:c.background}} contentContainerStyle={s.page}>
    <Text accessibilityRole="header" style={[s.title,{color:c.ink}]}>{system==='electrical'?'Electrical Maintenance':'Engine Maintenance'}</Text>
    <Text style={[s.description,{color:c.muted}]}>Completed work for your {system==='electrical'?'electrical system':'engine equipment'}.</Text>
    {error!==''&&<Text accessibilityRole="alert" style={{color:c.amber,lineHeight:22}}>{error}</Text>}
    {notice!==''&&<Text accessibilityLiveRegion="polite" style={{color:c.accent}}>{notice}</Text>}
    <View style={s.row}>{loaded&&button('Record completed work',()=>{setEditing(true);setNotice('');})}{button('Refresh records',refresh)}</View>
    {editing&&<View style={[s.panel,{borderColor:c.line,backgroundColor:c.panel}]}>
      <Text accessibilityRole="header" style={[s.heading,{color:c.ink}]}>Record completed work</Text>
      <Text style={[s.label,{color:c.ink}]}>Equipment *</Text>
      <View style={s.row}>{visibleEquipment.map(e=>button(e.name,()=>patch('equipmentId',e.id),form.equipmentId===e.id))}{button('Add equipment',()=>setAdding(value=>!value))}</View>
      {adding&&<View style={[s.panel,{borderColor:c.line}]}>
        <Text style={[s.label,{color:c.ink}]}>Equipment name</Text><TextInput accessibilityLabel="Equipment name" value={equipmentName} onChangeText={setEquipmentName} editable={!busy} maxLength={120} style={[s.input,{borderColor:c.line,color:c.ink}]}/>
        {button('Save equipment',createEquipment)}
      </View>}
      {field('Work date * (YYYY-MM-DD)','date',{max:10})}
      <Text style={[s.label,{color:c.ink}]}>Work type</Text><View style={s.row}>{kinds.map(kind=>button(kind,()=>patch('kind',kind),form.kind===kind))}</View>
      {field('Work performed *','description',{multiline:true,max:2000})}
      {field('Performed by / service provider','provider')}
      {field('Equipment hours (optional, manual)','hours',{numeric:true,max:20})}
      {field('Parts / quantities','parts',{multiline:true,max:2000})}
      {field('Cost (optional)','cost',{numeric:true,max:20})}{field('Currency (CAD / USD / other)','currency',{max:3})}
      {field('Invoice / survey reference','reference')}
      {field('Notes / findings','notes',{multiline:true,max:4000})}
      <Text style={[s.heading,{color:c.ink}]}>Next service (optional)</Text><Text style={{color:c.muted}}>If both are entered, service is due when either threshold is reached. These are your recorded thresholds; no reminder is scheduled.</Text>
      {field('Next service date (YYYY-MM-DD)','nextDate',{max:10})}{field('Next service at equipment hours','nextHours',{numeric:true,max:20})}
      <View style={s.row}>{button(busy?'Saving…':'Save service record',save)}{button('Cancel',()=>{setEditing(false);setAdding(false);setForm(blank());setError('');})}</View>
    </View>}
    <Text accessibilityRole="header" style={[s.heading,{color:c.ink}]}>Service history</Text>
    {loaded&&<View style={s.row}>{button('All equipment',()=>setEquipmentFilter(''),!equipmentFilter)}{visibleEquipment.map(e=>button(`Show ${e.name}`,()=>setEquipmentFilter(e.id),equipmentFilter===e.id))}</View>}
    {legacyEntries.length>0&&<View style={{gap:8}}>
      {button(`Earlier shared records (${legacyEntries.length})`,()=>setShowLegacy(value=>!value),showLegacy)}
      <Text style={[s.note,{color:c.muted}]}>These entries predate separate logs and have not been assigned to Engine or Electrical.</Text>
    </View>}
    {loaded&&currentEntries.length===0&&<Text style={{color:c.muted}}>No completed work recorded for this selection.</Text>}
    {entries.map(entry=><View key={entry.id} style={[s.panel,{backgroundColor:c.panel,borderColor:c.line}]}>
      <Text style={[s.heading,{color:c.ink}]}>{data.equipment.find(e=>e.id===entry.equipmentId)?.name??'Equipment'}</Text>
      {scopeOf(entry)==='legacy'&&<Text style={{color:c.amber}}>Earlier shared record · log not assigned</Text>}
      <Text style={{color:c.muted}}>{entry.date} · {entry.kind}{entry.hours!==null?` · ${entry.hours} h (manual)`:''}</Text>
      <Text style={[s.description,{color:c.ink}]}>{entry.description}</Text>
      {entry.provider!==''&&<Text style={{color:c.ink}}>Performed by: {entry.provider}</Text>}
      {entry.parts!==''&&<Text style={{color:c.ink}}>Parts: {entry.parts}</Text>}
      {entry.costMinor!==null&&<Text style={{color:c.ink}}>Cost: {entry.currency} {(entry.costMinor/100).toFixed(2)}</Text>}
      {entry.reference!==''&&<Text style={{color:c.ink}}>Reference: {entry.reference}</Text>}
      {entry.notes!==''&&<Text style={{color:c.ink}}>{entry.notes}</Text>}
      {(entry.nextDate||entry.nextHours!==null)&&<Text style={{color:c.accent}}>Next service: {[entry.nextDate,entry.nextHours!==null?`${entry.nextHours} h`:null].filter(Boolean).join(' or ')}</Text>}
      <Text style={[s.note,{color:c.muted}]}>Recorded {new Date(entry.createdAt).toLocaleString()}</Text>
    </View>)}
    <Text style={[s.note,{color:c.muted}]}>Stored locally for this boat. Keep original receipts and service documents. File attachments, exports and automatic reminders are not available yet.</Text>
  </ScrollView></SafeAreaView>;
}
const s=StyleSheet.create({safe:{flex:1,backgroundColor:'#153b5e'},page:{padding:22,gap:18,maxWidth:900,width:'100%',alignSelf:'center',paddingBottom:48},title:{fontSize:32,fontWeight:'700'},description:{fontSize:16,lineHeight:25},heading:{fontSize:20,fontWeight:'700'},row:{flexDirection:'row',flexWrap:'wrap',gap:10},button:{minHeight:44,borderWidth:1,borderRadius:10,padding:12,justifyContent:'center'},panel:{borderWidth:1,borderRadius:14,padding:18,gap:14},field:{gap:8},label:{fontSize:15,fontWeight:'600'},input:{borderWidth:1,borderRadius:8,minHeight:48,padding:12,fontSize:16},multiline:{minHeight:90,textAlignVertical:'top'},note:{fontSize:12,lineHeight:19}});
