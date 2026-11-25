import {useCallback,useEffect,useRef,useState} from 'react';
import {ActivityIndicator, Image, Modal, Platform, Pressable, ScrollView, StyleSheet, Text, TextInput, useWindowDimensions, View} from 'react-native';
import * as ImagePicker from 'expo-image-picker';
import {router} from 'expo-router';
import exampleDiagram from '../diagrams/example.json';
import {useVisibleViewport} from '../diagrams/use-visible-viewport';
import {useViewSwipe} from '../diagrams/use-view-swipe';
import {componentAction} from '../diagrams/interaction';
import {usePreferences} from '../preferences/Preferences';
import {inSystem,newId,recordSystem,type Equipment,type Snapshot} from '../maintenance/model';
import {listDiagrams,readDiagram,saveDiagram} from '../diagrams/client';
import {buildReviewed,componentTypes,engineComponentTypes,engineViews,type Box,type Diagram,type DiagramSystem,type DiagramSummary,type ImageDraft,type ReviewItem} from '../diagrams/model';

type Saved={diagram:Diagram;uri:string};
type Draft=ImageDraft&{diagramId:string;name:string;imageId:string;revision:string;baseRevision:string|null;items:ReviewItem[]};
// Regions for the bundled example only; they never represent the user's equipment.
const exampleAreas=[
  {x:.04,y:.12,width:.12,height:.22},{x:.05,y:.36,width:.12,height:.22},
  {x:.35,y:.27,width:.18,height:.21},{x:.50,y:.11,width:.12,height:.15},
  {x:.63,y:.25,width:.16,height:.23},{x:.81,y:.19,width:.12,height:.32},
  {x:.55,y:.59,width:.13,height:.08},{x:.83,y:.59,width:.14,height:.20},
  {x:.17,y:.76,width:.14,height:.19},{x:.36,y:.76,width:.14,height:.19},
].map(a=>({...a,equipmentId:''}));
export function MaintenanceDiagram({system,data,onSaved}:{system:DiagramSystem;data:Snapshot;onSaved:()=>Promise<void>}) {
  const isEngine=system==='engine';
  const title=isEngine?'Engine diagram':'Electrical diagram';
  const types=isEngine?engineComponentTypes:componentTypes;
  const [diagrams,setDiagrams]=useState<DiagramSummary[]>([]);
  const [activeId,setActiveId]=useState('main');
  const {colors:c}=usePreferences();
  const windowSize=useWindowDimensions();
  const viewport=useVisibleViewport(windowSize.width,windowSize.height);
  const {width:screenWidth,height:screenHeight}=viewport;
  const backdropViewport=Platform.OS==='web'?{position:'absolute' as const,flex:0,left:viewport.left,top:viewport.top,width:screenWidth,height:screenHeight}:{};
  const modalMargin=screenWidth<480?12:24;
  const modalSize={width:Math.max(0,screenWidth-2*modalMargin),maxWidth:720,maxHeight:Math.max(0,screenHeight-2*modalMargin)};
  const [saved,setSaved]=useState<Saved|null>(null);
  const [draft,setDraft]=useState<Draft|null>(null);
  const [error,setError]=useState('');
  const [notice,setNotice]=useState('');
  const [busy,setBusy]=useState(false);
  const [loading,setLoading]=useState(true);
  const [progress,setProgress]=useState('');
  const [search,setSearch]=useState('');
  const [selected,setSelected]=useState<Equipment|null>(null);
  const [selectedArea,setSelectedArea]=useState<number|null>(null);
  const [focused,setFocused]=useState('');
  const [showTypes,setShowTypes]=useState(false);
  const [viewWidth,setViewWidth]=useState(1);
  const mounted=useRef(true);
  const touchPress=useRef(false);
  const imageRef=useRef<View>(null);
  const hoverSequence=useRef(0);
  const [hoverHeight,setHoverHeight]=useState(240);
  const [hovered,setHovered]=useState<{equipment:Equipment|null;left:number;anchorY:number;imageTop:number;width:number}|null>(null);
  const hideHover=useCallback(()=>setHovered(null),[]);
  useEffect(()=>{hoverSequence.current++;},[draft,focused,selected,busy,loading]);
  function showHover(equipment:Equipment|null,area:Box,pointer:{pageY?:number;clientY?:number}) {
    if(busy||loading||focused||selected)return;
    const sequence=++hoverSequence.current;
    imageRef.current?.measureInWindow((_left,top,width,height)=>{
      if(!mounted.current||sequence!==hoverSequence.current||width<=0)return;
      const cardWidth=Math.min(260,width);
      const pointerY=pointer.clientY??pointer.pageY;
      // Use the actual pointer/visible marker position, not a fixed percentage
      // of an image which may extend far above or below the viewport.
      const anchorY=Number.isFinite(pointerY)?pointerY!:top+(area.y+area.height/2)*height;
      setHovered({equipment,left:Math.max(0,Math.min(area.x*width,width-cardWidth)),anchorY,imageTop:top,width:cardWidth});
    });
  }
  const hoverVisible=!!hovered;
  useEffect(()=>{
    if(Platform.OS!=='web'||!hoverVisible)return;
    const hide=()=>{hoverSequence.current++;hideHover();};
    window.addEventListener('scroll',hide,true);window.addEventListener('resize',hide);
    return ()=>{window.removeEventListener('scroll',hide,true);window.removeEventListener('resize',hide);};
  },[hoverVisible,hideHover]);
  useEffect(()=>{mounted.current=true;(async()=>{const items=await listDiagrams(system);const id=items[0]?.id??'main';const value=await readDiagram(system,id);if(mounted.current){setDiagrams(items);setActiveId(id);setSaved(value);}})().catch(e=>{if(mounted.current)setError(e.message);}).finally(()=>{if(mounted.current)setLoading(false);});return()=>{mounted.current=false;};},[system]);
  const equipment=data.equipment.filter(e=>inSystem(e,system));
  const current=draft??(saved?{...saved.diagram,uri:saved.uri}:isEngine?null:exampleDiagram);
  function createEditDraft():Draft {
    if(saved) return {diagramId:activeId,name:diagrams.find(d=>d.id===activeId)?.name??title,...saved.diagram,uri:saved.uri,revision:newId(),baseRevision:saved.diagram.revision,items:saved.diagram.areas.map(a=>{const e=data.equipment.find(e=>e.id===a.equipmentId);return {...a,key:newId(),newId:newId(),equipmentId:a.equipmentId,name:e?.name??'',category:e?.category??'Unknown / custom',model:e?.model??'',serialNumber:e?.serialNumber??'',make:e?.make??'',manufacturedDate:e?.manufacturedDate??'',installedDate:e?.installedDate??'',ignored:false};})};
    return {diagramId:activeId,name:title,...exampleDiagram,imageId:newId(),revision:newId(),baseRevision:null,items:exampleAreas.map((a,i)=>({...a,key:newId(),newId:newId(),name:['Solar panel 1','Solar panel 2','Solar charge controller','Solar system monitor','Alternator','Electrical distribution panel','Battery monitor shunt','Battery monitor','Battery 1','Battery 2'][i],category:'Unknown / custom',model:'',serialNumber:'',ignored:false}))};
  }
  function editArea(index:number) {
    const next=createEditDraft();
    const item=next.items[index];
    if(!item)return;
    setSelected(null);setSelectedArea(null);hideHover();setShowTypes(false);setDraft(next);setFocused(item.key);
  }
  async function choose(replace=false) {
    setError('');setNotice('');
    try {
      const result=await ImagePicker.launchImageLibraryAsync({mediaTypes:['images'],base64:true,quality:1,allowsEditing:false});
      if(result.canceled)return;
      const asset=result.assets[0];
      if(asset.fileSize&&asset.fileSize>6_000_000)throw Error('Choose a JPEG or PNG smaller than 6 MB.');
      let encoded=asset.base64;
      if(!encoded&&Platform.OS==='web') {
        const blob=await (await fetch(asset.uri)).blob();
        if(blob.size>6_000_000)throw Error('Choose a JPEG or PNG smaller than 6 MB.');
        const uri=await new Promise<string>((resolve,reject)=>{const reader=new FileReader();reader.onload=()=>resolve(String(reader.result));reader.onerror=()=>reject(Error('Could not read image.'));reader.readAsDataURL(blob);});
        encoded=uri.split(',')[1];
      }
      if(!encoded||encoded.length>8_000_000)throw Error('Could not read the image. Choose a JPEG or PNG smaller than 6 MB.');
      const mime=encoded.startsWith('/9j/')?'jpeg':encoded.startsWith('iVBORw0KGgo')?'png':null;
      if(!mime)throw Error('Choose a JPEG or PNG diagram. Export other formats as an image first.');
      if(asset.width<1||asset.height<1||asset.width>20000||asset.height>20000)throw Error('Image dimensions must be between 1 and 20,000 pixels.');
      setDraft({diagramId:!isEngine||replace?activeId:newId(),name:replace?(diagrams.find(d=>d.id===activeId)?.name??title):isEngine?engineViews.find(view=>!diagrams.some(d=>d.name===view))??'Other view':title,uri:`data:image/${mime};base64,${encoded}`,width:asset.width,height:asset.height,imageId:newId(),revision:newId(),baseRevision:(!isEngine||replace)?saved?.diagram.revision??null:null,items:[]});
      setFocused('');
    }catch(e){setError(e instanceof Error?e.message:'Could not open the diagram.');}
  }
  function patch(key:string,values:Partial<ReviewItem>) {setDraft(old=>old?{...old,items:old.items.map(i=>i.key===key?{...i,...values}:i)}:old);}
  function mark(x:number,y:number) {
    if(!draft||busy||draft.items.length>=60)return;
    const key=newId();
    setDraft({...draft,items:[...draft.items,{key,newId:newId(),equipmentId:'',ignored:false,name:'',category:'Unknown / custom',model:'',serialNumber:'',x:Math.max(0,Math.min(.85,x-.075)),y:Math.max(0,Math.min(.85,y-.075)),width:.15,height:.15}]});
    setShowTypes(false);setFocused(key);
  }
  async function save() {
    if(!draft)return;
    setError('');setNotice('');setBusy(true);
    try {
      const reviewed=buildReviewed(draft.items,data.equipment,system);
      const diagram:Diagram={revision:draft.revision,imageId:draft.imageId,imageChunks:Math.ceil(draft.uri.length/16000),width:draft.width,height:draft.height,areas:reviewed.areas};
      await saveDiagram(diagram,draft.uri,reviewed.equipment,draft.baseRevision,setProgress,{system,diagramId:draft.diagramId,name:draft.name});
      setSaved({diagram,uri:draft.uri});setActiveId(draft.diagramId);setFocused('');setShowTypes(false);setDiagrams(old=>old.some(d=>d.id===draft.diagramId)?old.map(d=>d.id===draft.diagramId?{id:draft.diagramId,name:draft.name}:d):[...old,{id:draft.diagramId,name:draft.name}]);setDraft(null);setNotice('Diagram and reviewed equipment saved on this device.');
      try {await onSaved();}catch{setError('Diagram saved. Refresh maintenance records to reload equipment.');}
    }catch(e){setError(e instanceof Error?e.message:'Could not save the diagram. Your review is still here.');}finally{setBusy(false);setProgress('');}
  }
  async function reload() {
    setError('');setLoading(true);
    try{const items=await listDiagrams(system);const id=items.some(d=>d.id===activeId)?activeId:items[0]?.id??'main';const value=await readDiagram(system,id);setDiagrams(items);setActiveId(id);setSaved(value);}catch(e){setError(e instanceof Error?e.message:'Could not reload diagram.');}finally{setLoading(false);}
  }
  async function selectDiagram(id:string) {
    setError('');setLoading(true);setSelected(null);hideHover();setFocused('');
    try {const value=await readDiagram(system,id);if(!value)throw Error('Diagram not found. Reload diagrams.');setSaved(value);setActiveId(id);}catch(e){setError(e instanceof Error?e.message:'Could not load diagram.');}finally{setLoading(false);}
  }
  const activeIndex=diagrams.findIndex(d=>d.id===activeId);
  function changeView(direction:1|-1) {
    const next=diagrams[activeIndex+direction];
    if(next)void selectDiagram(next.id);
  }
  const swipe=useViewSwipe(isEngine&&!draft&&!busy&&!loading&&!selected&&diagrams.length>1,changeView);
  function button(label:string,action:()=>void,active=false,disabled=busy||loading,key=label) {return <Pressable key={key} accessibilityRole="button" accessibilityLabel={label==='×'?'Close equipment details':label} accessibilityState={{disabled,selected:active}} disabled={disabled} onPress={action} style={[s.button,{borderColor:active?c.accent:c.line,backgroundColor:active?c.soft:c.panel}]}><Text style={{color:c.accent,fontWeight:'600'}}>{label}</Text></Pressable>;}
  function input(label:string,value:string,change:(value:string)=>void,max=120) {return <View style={s.field}><Text style={{color:c.ink,fontWeight:'600'}}>{label}</Text><TextInput accessibilityLabel={label} value={value} onChangeText={change} editable={!busy} maxLength={max} style={[s.input,{borderColor:c.line,color:c.ink,backgroundColor:c.background}]}/></View>;}
  const selectedEntries=selected?data.entries.filter(e=>e.equipmentId===selected.id&&recordSystem(e,selected)===system).sort((a,b)=>b.date.localeCompare(a.date)||b.createdAt-a.createdAt||b.id.localeCompare(a.id)):[];
  const latest=selectedEntries[0];
  function close() {setSelected(null);setSelectedArea(null);}
  const hoverEntries=hovered?.equipment?data.entries.filter(e=>e.equipmentId===hovered.equipment!.id&&recordSystem(e,hovered.equipment!)===system).sort((a,b)=>b.date.localeCompare(a.date)||b.createdAt-a.createdAt||b.id.localeCompare(a.id)):[];
  const hoverLatest=hoverEntries[0];
  return <View style={[s.panel,{backgroundColor:c.panel,borderColor:c.line}]}>
    <Text accessibilityRole="header" style={[s.heading,{color:c.ink}]}>{isEngine&&!draft&&saved?diagrams.find(d=>d.id===activeId)?.name??title:title}</Text>
    <Text style={{color:c.muted}}>Hover or tap for the latest service. On a phone or tablet, touch and hold to edit.</Text>
    {loading&&<ActivityIndicator accessibilityLabel={`Loading ${system} diagram`} color={c.accent}/>}
    {error!==''&&<Text accessibilityRole="alert" style={{color:c.amber}}>{error}</Text>}
    {notice!==''&&<Text accessibilityLiveRegion="polite" style={{color:c.accent}}>{notice}</Text>}
    {!draft&&<View style={s.row}>{button(saved?'Add/update diagram':'Add diagram',()=>choose(),false,loading||busy)}{button('Reload diagram',reload,false,loading||busy)}{(!isEngine||saved)&&button('Edit equipment links',()=>{hideHover();setDraft(createEditDraft());},false,loading||busy)}</View>}
    {isEngine&&!current&&!loading&&<Text style={{color:c.muted}}>No engine diagram added yet.</Text>}
    {current&&<View {...(isEngine?swipe.handlers:{})} testID="maintenance-diagram-image" ref={imageRef} onLayout={event=>setViewWidth(event.nativeEvent.layout.width)} style={{width:'100%',aspectRatio:current.width/current.height,backgroundColor:'#ffffff',position:'relative',zIndex:hovered?10:0}}>
      <Image accessibilityLabel={draft?'Diagram under review':isEngine?'Saved engine view':'Saved electrical system diagram'} source={{uri:current.uri}} resizeMode="contain" style={StyleSheet.absoluteFill}/>
      {draft&&<Pressable accessibilityRole="button" accessibilityLabel="Mark equipment on diagram" disabled={busy} style={StyleSheet.absoluteFill} onPress={event=>mark(event.nativeEvent.locationX/viewWidth,event.nativeEvent.locationY/(viewWidth*current.height/current.width))}/>}
      {(draft?draft.items.filter(i=>!i.ignored):saved?saved.diagram.areas:isEngine?[]:exampleAreas).map((area,index)=>{
        const item=draft?area as ReviewItem:null;
        const e=equipment.find(e=>e.id===area.equipmentId);
        const open=(event:{nativeEvent:unknown})=>{if(swipe.blockPress.current)return;hideHover();const native=event.nativeEvent as {pointerType?:string};const isTouch=touchPress.current||native.pointerType==='touch';const mouseInput=Platform.OS==='web'&&!isTouch;const action=componentAction(!!draft,!!saved,mouseInput);if(action==='edit-review'&&item&&mouseInput){setShowTypes(false);setFocused(item.key);}else if(action==='show-service'&&item){setSelectedArea(draft!.items.findIndex(i=>i.key===item.key));setSelected(e??{id:'',name:item.name||'Unknown equipment',system,make:item.make,model:item.model,serialNumber:item.serialNumber,manufacturedDate:item.manufacturedDate,installedDate:item.installedDate});}else if(action==='edit-new')editArea(index);else {setSelectedArea(index);setSelected(e??{id:'',name:'Unknown equipment',system});}};
        return <Pressable key={item?.key??`${area.equipmentId}-${index}`} disabled={busy||loading} hitSlop={4} accessibilityRole="button" accessibilityLabel={item?`Review ${item.name||'unknown component'} ${index+1}`:e?`Service details for ${e.name}`:`Service details for unknown equipment ${index+1}`} onPressIn={event=>{const native=event.nativeEvent as {type?:string;pointerType?:string};touchPress.current=Platform.OS!=='web'||native.type==='touchstart'||native.pointerType==='touch';}} onPress={open} onLongPress={event=>{if(swipe.blockPress.current)return;const native=event.nativeEvent as {type?:string;pointerType?:string};if(!touchPress.current&&native.type!=='touchstart'&&native.pointerType!=='touch')return;hideHover();if(item){setShowTypes(false);setFocused(item.key);}else editArea(index);}} onHoverIn={event=>{const details=item?{...(e??{id:'',name:item.name||'Unknown equipment',system}),category:item.category,model:item.model,serialNumber:item.serialNumber,make:item.make,manufacturedDate:item.manufacturedDate,installedDate:item.installedDate}:e??null;showHover(details,area,event.nativeEvent);}} onHoverOut={()=>{hoverSequence.current++;hideHover();}} style={[s.area,{left:`${area.x*100}%`,top:`${area.y*100}%`,width:`${area.width*100}%`,height:`${area.height*100}%`,borderColor:focused===item?.key?'#d97706':'#16887f',backgroundColor:draft?'rgba(22,136,127,0.12)':'rgba(22,136,127,0.035)'}]}><Text style={s.marker}>{index+1}</Text></Pressable>;
      })}
      {hovered&&<View pointerEvents="none" accessibilityLabel="Latest equipment service" onLayout={event=>{const height=event.nativeEvent.layout.height;if(height>0&&Math.abs(height-hoverHeight)>.5)setHoverHeight(height);}} style={[s.hoverCard,{left:hovered.left,top:Math.max(viewport.top+12,Math.min(hovered.anchorY+12,viewport.top+screenHeight-hoverHeight-12))-hovered.imageTop,width:hovered.width,backgroundColor:c.background,borderColor:c.line}]}>
        <Text style={[s.hoverTitle,{color:c.ink}]}>{hovered.equipment?.name||'Unknown equipment'}</Text>
        <Text style={{color:c.ink}}>Make: {hovered.equipment?.make||'N/A'}</Text>
        <Text style={{color:c.ink}}>Model: {hovered.equipment?.model||'N/A'}</Text>
        <Text style={{color:c.ink}}>Serial number: {hovered.equipment?.serialNumber||'N/A'}</Text>
        <Text style={{color:c.ink}}>Manufactured: {hovered.equipment?.manufacturedDate||'N/A'}</Text>
        <Text style={{color:c.ink}}>Installed: {hovered.equipment?.installedDate||'N/A'}</Text>
        <Text style={{color:c.ink}}>Latest service: {hoverLatest?.date||'N/A'}</Text>
        <Text style={{color:c.ink}} numberOfLines={3}>Description: {hoverLatest?.description||'No service history'}</Text>
      </View>}
    </View>}
    {isEngine&&!draft&&diagrams.length>1&&<Text accessibilityLiveRegion="polite" style={{color:c.muted,fontSize:12,textAlign:'center'}}>{activeIndex+1} / {diagrams.length}</Text>}
    {isEngine&&draft&&<View style={{gap:8}}>{input('View name',draft.name,value=>setDraft(old=>old?{...old,name:value}:old))}<View style={s.row}>{engineViews.slice(0,4).map(view=>button(view,()=>setDraft(old=>old?{...old,name:view}:old),draft.name===view))}</View>{saved&&draft.imageId!==saved.diagram.imageId&&<View style={s.row}>{button('Add as new view',()=>setDraft(old=>old?{...old,diagramId:old.imageId,baseRevision:null}:old),draft.baseRevision===null)}{button('Replace current view',()=>setDraft(old=>old?{...old,diagramId:activeId,baseRevision:saved.diagram.revision,name:diagrams.find(d=>d.id===activeId)?.name??old.name}:old),draft.baseRevision!==null)}</View>}</View>}
    {!draft&&saved&&<View style={s.row}>{saved.diagram.areas.map((area,index)=>{const e=equipment.find(e=>e.id===area.equipmentId);return e?button(`${index+1}. ${e.name}`,()=>{setSelectedArea(index);setSelected(e);}):null;})}</View>}
    {draft&&<View style={{gap:16}}>
      {button('Add component manually',()=>mark(.5,.5),false,busy||draft.items.length>=60)}
      <Text style={{color:c.muted}}>Select an outlined component to review its details, or select a location to add equipment. Save when the equipment is ready.</Text>
      {draft.items.every(i=>i.ignored)&&<Text style={{color:c.muted}}>You can save the image now and add equipment links later.</Text>}
      {progress!==''&&<Text accessibilityLiveRegion="polite" style={{color:c.accent}}>{progress}</Text>}
      <View style={s.row}>{button('Save diagram and equipment',save)}{button('Cancel diagram changes',()=>{setDraft(null);setFocused('');setError('');setNotice('');})}</View>
    </View>}
    <Modal visible={!!draft&&!!focused} transparent animationType="none" onRequestClose={()=>setFocused('')}>
      <View style={[s.backdrop,backdropViewport,{padding:modalMargin}]}><View accessibilityViewIsModal style={[s.dialog,modalSize,{height:Math.max(0,screenHeight-2*modalMargin),backgroundColor:c.background,borderColor:c.line}]}>
        <View style={[s.row,{justifyContent:'space-between',alignItems:'center',flexWrap:'nowrap',flexShrink:0}]}>
          <Text accessibilityRole="header" style={[s.heading,{color:c.ink,flex:1}]}>Equipment details</Text>
          {button(busy?'Saving…':'Save changes',save)}
          <Pressable accessibilityRole="button" accessibilityLabel="Close review" accessibilityState={{disabled:busy}} disabled={busy} onPress={()=>setFocused('')} style={[s.button,{borderColor:c.line,backgroundColor:c.panel,width:44,padding:8}]}><Text style={{color:c.accent,fontSize:24,textAlign:'center'}}>×</Text></Pressable>
        </View>
        {error!==''&&<Text accessibilityRole="alert" style={{color:c.amber}}>{error}</Text>}
        {progress!==''&&<Text accessibilityLiveRegion="polite" style={{color:c.accent}}>{progress}</Text>}
        <ScrollView style={[s.dialogScroll,{flex:1,minHeight:0}]} contentContainerStyle={{gap:16,paddingVertical:12}} keyboardShouldPersistTaps="handled">
      {draft?.items.map((item,index)=>item.key!==focused?null:<View key={item.key} style={[s.panel,{borderColor:focused===item.key?c.accent:c.line,opacity:item.ignored?.6:1}]}>
        <Text style={[s.heading,{color:c.ink}]}>{index+1}. {item.name||'Unknown component'}</Text>
        {button('Remove component',()=>{setDraft(old=>old?{...old,items:old.items.filter(i=>i.key!==item.key)}:old);setFocused('');})}
        {!item.ignored&&<>
          {!item.equipmentId&&input('Equipment name',item.name,value=>patch(item.key,{name:value}))}
          {isEngine&&<>
            <Text style={{color:c.muted}}>Link a component already recorded on another view, or create a separate component.</Text>
            <View style={s.row}>{button('Create separate equipment',()=>patch(item.key,{equipmentId:''}),!item.equipmentId)}{equipment.map(e=>button(`Link ${e.name}`,()=>patch(item.key,{equipmentId:e.id,name:e.name,category:e.category||'Unknown / custom',model:e.model??'',serialNumber:e.serialNumber??'',make:e.make??'',manufacturedDate:e.manufacturedDate??'',installedDate:e.installedDate??''}),item.equipmentId===e.id))}</View>
          </>}
          {input('Component type',item.category,value=>patch(item.key,{category:value}),80)}
          {showTypes&&<>
            {input('Find common component types',search,setSearch,80)}
            <View style={s.row}>{types.filter(t=>t.toLowerCase().includes(search.toLowerCase())).map(type=>button(type,()=>patch(item.key,{category:type}),item.category===type))}</View>
          </>}
          {button(showTypes?'Hide common types':'Choose common type',()=>{setShowTypes(!showTypes);setSearch('');})}
          {input('Make',item.make??'',value=>patch(item.key,{make:value}))}
          {input('Date manufactured (YYYY-MM-DD)',item.manufacturedDate??'',value=>patch(item.key,{manufacturedDate:value}),10)}
          {input('Date installed (YYYY-MM-DD)',item.installedDate??'',value=>patch(item.key,{installedDate:value}),10)}
          {input('Model',item.model,value=>patch(item.key,{model:value}))}
          {input('Serial number',item.serialNumber,value=>patch(item.key,{serialNumber:value}))}
          <Text style={{color:c.muted}}>Leave model and serial number blank if unknown. A component type does not identify a particular piece of equipment.</Text>
          <View style={s.row}>{(['x','y','width','height'] as const).map(key=><View style={{flexGrow:1,minWidth:100}} key={key}>{input(`${key} (%)`,String(Math.round(item[key]*10000)/100),value=>{const number=Number(value);if(Number.isFinite(number))patch(item.key,{[key]:number/100});},8)}</View>)}</View>
        </>}
      </View>)}
        </ScrollView>
      </View></View>
    </Modal>
    <Modal visible={selected!==null} transparent animationType="none" onRequestClose={close}>
      <View style={[s.backdrop,backdropViewport,{padding:modalMargin}]}><View accessibilityViewIsModal style={[s.dialog,modalSize,{backgroundColor:c.background,borderColor:c.line}]}><ScrollView style={s.dialogScroll} contentContainerStyle={{gap:16}}>
        <View style={[s.row,{justifyContent:'space-between',alignItems:'center'}]}><Text accessibilityRole="header" style={[s.heading,{color:c.ink,flex:1}]}>{selected?.name}</Text>{button('×',close,false,false)}</View>
        <Text style={{color:c.ink}}>Make: {selected?.make||'N/A'}</Text>
        <Text style={{color:c.ink}}>Date manufactured: {selected?.manufacturedDate||'N/A'}</Text>
        <Text style={{color:c.ink}}>Date installed: {selected?.installedDate||'N/A'}</Text>
        <Text style={{color:c.ink}}>Model: {selected?.model||'N/A'}</Text>
        <Text style={{color:c.ink}}>Serial number: {selected?.serialNumber||'N/A'}</Text>
        <Text style={{color:c.ink}}>Service date: {latest?.date??'N/A'}</Text>
        <Text style={{color:c.ink}}>Description: {latest?.description??'No service history'}</Text>
        {selectedArea!==null&&button('Edit equipment details',()=>{if(draft){const item=draft.items[selectedArea];setSelected(null);setSelectedArea(null);setShowTypes(false);if(item)setFocused(item.key);}else editArea(selectedArea);},false,false)}
        {!!selected?.id&&button('View service history',()=>{const id=selected?.id;close();if(id)router.push({pathname:isEngine?'/mech/service-history':'/elec/service-history',params:{equipmentId:id}});},false,false)}
      </ScrollView></View></View>
    </Modal>
  </View>;
}
const s=StyleSheet.create({panel:{borderWidth:1,borderRadius:14,padding:16,gap:14},heading:{fontSize:20,fontWeight:'700'},row:{flexDirection:'row',flexWrap:'wrap',gap:8},button:{minHeight:44,minWidth:44,padding:12,borderWidth:1,borderRadius:9,justifyContent:'center',alignItems:'center'},input:{minHeight:48,borderWidth:1,borderRadius:8,padding:12,fontSize:16},field:{gap:6},area:{position:'absolute',borderWidth:2,borderRadius:5,zIndex:1},marker:{backgroundColor:'#16887f',color:'#fff',fontWeight:'700',fontSize:12,alignSelf:'flex-start',paddingHorizontal:4},backdrop:{flex:1,backgroundColor:'rgba(0,0,0,.7)',justifyContent:'center'},dialog:{padding:16,borderRadius:16,borderWidth:1,alignSelf:'center',flexShrink:1},dialogScroll:{flexShrink:1},hoverCard:{position:'absolute',width:260,maxWidth:'100%',padding:12,borderWidth:1,borderRadius:10,gap:3,zIndex:10},hoverTitle:{fontWeight:'700',fontSize:15}});
