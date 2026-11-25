import type { Equipment } from '../maintenance/model';

export type DiagramSystem = 'engine'|'electrical';
export type DiagramSummary = {id:string;name:string};
export const engineViews = ['Upper view','Port side view','Starboard side view','Bottom view','Front view','Rear view','Other view'] as const;
export const engineComponentTypes = ['Unknown / custom','Engine','Oil filter','Fuel filter','Fuel pump','Fuel injector','Water pump','Impeller','Heat exchanger','Coolant reservoir','Thermostat','Belt','Hose','Alternator','Starter motor','Air filter','Exhaust elbow','Turbocharger','Transmission','Engine mount','Anode','Sensor'] as const;
export const componentTypes = ['Unknown / custom', 'LiFePO₄ battery', 'AGM battery', 'Flooded lead-acid battery', 'Gel battery', 'Battery bank', 'Solar panel', 'MPPT solar charge controller', 'PWM solar charge controller', 'Alternator', 'DC–DC charger', 'Battery charger', 'Inverter', 'Inverter/charger', 'Battery monitor', 'Battery-monitor shunt', 'Solar system monitor', 'DC distribution panel', 'AC distribution panel', 'Busbar', 'Fuse / fuse holder', 'Circuit breaker', 'Battery isolation switch', 'BMS', 'DC load'] as const;
export type Box = {x:number;y:number;width:number;height:number};
export type Detection = Box & {name:string;category:string;model:string;serialNumber:string};
export type ReviewItem = Detection & {key:string;equipmentId:string;newId:string;ignored:boolean;make?:string;manufacturedDate?:string;installedDate?:string};
export type Area = Box & {equipmentId:string};
export type Diagram = {revision:string;imageId:string;imageChunks:number;width:number;height:number;areas:Area[]};
export type ImageDraft = {uri:string;width:number;height:number};
export const normalized = (s:string) => s.toLocaleLowerCase().replace(/[^\p{L}\p{N}]/gu,'');
export function suggestedMatch(item: Detection, equipment: Equipment[], system:DiagramSystem = 'electrical'): string {
  const candidates=equipment.filter(e=>e.system===system||e.system==='both');
  // Repeated models and pictures do not prove identity. Only unambiguous exact
  // serial/name matches are proposed; the user still confirms every link.
  if(item.serialNumber.trim()) {
    const hits=candidates.filter(e=>e.serialNumber && normalized(e.serialNumber)===normalized(item.serialNumber));
    if(hits.length>0) return hits.length===1 && (!item.model || !hits[0].model || normalized(hits[0].model!)===normalized(item.model)) ? hits[0].id : '';
  }
  const hits=candidates.filter(e=>normalized(e.name)===normalized(item.name) && !(item.serialNumber && e.serialNumber && normalized(item.serialNumber)!==normalized(e.serialNumber)));
  return hits.length===1 ? hits[0].id : '';
}
export function validBox(b:Box) {
  return [b.x,b.y,b.width,b.height].every(Number.isFinite) && b.x>=0 && b.y>=0 && b.width>0 && b.height>0 && b.x+b.width<=1.000001 && b.y+b.height<=1.000001;
}
export function parseDetections(value:unknown):Detection[] {
  if(!value || typeof value!=='object' || !('components' in value) || !Array.isArray(value.components) || value.components.length>60)throw Error('The scan returned invalid equipment. Try again or mark equipment manually.');
  return value.components.map((v:unknown)=>{
    if(!v||typeof v!=='object')throw Error('Invalid component returned by scan.');
    const o=v as Record<string,unknown>;
    for(const [field,max] of [['name',120],['category',80],['model',120],['serialNumber',120]] as const)if(typeof o[field]!=='string'||(o[field] as string).length>max)throw Error('Invalid equipment details returned by scan.');
    const b={x:o.x,y:o.y,width:o.width,height:o.height} as Box;
    if(!validBox(b)||!(o.name as string).trim())throw Error('Invalid equipment area returned by scan.');
    return {...b,name:o.name as string,category:o.category as string,model:o.model as string,serialNumber:o.serialNumber as string};
  });
}
export function buildReviewed(items:ReviewItem[], existing:Equipment[], system:DiagramSystem = 'electrical') {
  const equipment=new Map<string,Equipment>();
  const areas:Area[]=[];
  for(const item of items.filter(i=>!i.ignored)) {
    if(!validBox(item))throw Error('Each equipment area must fit inside the diagram.');
    const match=item.equipmentId?existing.find(e=>e.id===item.equipmentId&&(e.system===system||e.system==='both')):undefined;
    if(item.equipmentId&&!match)throw Error('Selected equipment no longer exists. Reload records.');
    const record:Equipment=match?{...match}:{id:item.newId,name:item.name.trim(),system};
    if(!record.name||record.name.length>120)throw Error('Enter an equipment name (up to 120 characters).');
    Object.assign(record,{category:item.category.trim(),model:item.model.trim(),serialNumber:item.serialNumber.trim(),make:(item.make??'').trim(),manufacturedDate:(item.manufacturedDate??'').trim(),installedDate:(item.installedDate??'').trim()});
    if(record.category!.length>80||record.model!.length>120||record.serialNumber!.length>120)throw Error('Equipment details are too long.');
    for(const date of [record.manufacturedDate,record.installedDate]) if(date && !/^\d{4}-\d{2}-\d{2}$/.test(date))throw Error('Dates must use YYYY-MM-DD.');
    const previous=equipment.get(record.id);
    if(previous && JSON.stringify(previous)!==JSON.stringify(record))throw Error('The same equipment has conflicting details. Use separate equipment records or make their details agree.');
    equipment.set(record.id,record);
    areas.push({equipmentId:record.id,x:item.x,y:item.y,width:item.width,height:item.height});
  }
  return {equipment:[...equipment.values()],areas};
}
