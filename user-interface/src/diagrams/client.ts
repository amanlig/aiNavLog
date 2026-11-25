import {command} from '../maintenance/client';
import type {Equipment} from '../maintenance/model';
import {parseDetections, type Diagram,type DiagramSystem,type DiagramSummary} from './model';
export async function listDiagrams(system:DiagramSystem):Promise<DiagramSummary[]> {
  const result=await command({op:'diagram_list',system}) as {diagrams:DiagramSummary[]};
  if(!Array.isArray(result.diagrams)||result.diagrams.length>50||result.diagrams.some(d=>typeof d.id!=='string'||typeof d.name!=='string'))throw Error('Could not load saved diagrams.');
  return result.diagrams;
}
export async function readDiagram(system:DiagramSystem = 'electrical',diagramId = 'main'):Promise<{diagram:Diagram;uri:string}|null> {
  const result=await command({op:'diagram_read',system,diagramId}) as {diagram:Diagram|null};
  if(!result.diagram)return null;
  const diagram=result.diagram;
  if(!Number.isInteger(diagram.imageChunks)||diagram.imageChunks<1||diagram.imageChunks>512)throw Error('Invalid saved diagram.');
  const parts:string[]=[];
  for(let index=0;index<diagram.imageChunks;index++) {
    const chunk=await command({op:'diagram_chunk',imageId:diagram.imageId,index}) as {body:string};
    if(typeof chunk.body!=='string')throw Error('Could not load the diagram image.');
    parts.push(chunk.body);
  }
  return {diagram,uri:parts.join('')};
}
export async function saveDiagram(diagram:Diagram,uri:string,equipment:Equipment[],baseRevision:string|null,onProgress:(message:string)=>void,scope:{system:DiagramSystem;diagramId:string;name:string} = {system:'electrical',diagramId:'main',name:'Electrical diagram'}) {
  const chunkSize=16000;
  if(!/^data:image\/(png|jpeg);base64,/.test(uri)||uri.length>8_000_030)throw Error('Choose a JPEG or PNG smaller than 6 MB.');
  const count=Math.ceil(uri.length/chunkSize);
  if(count!==diagram.imageChunks)throw Error('Invalid image chunk count.');
  for(let index=0;index<count;index++) {
    onProgress(`Saving diagram ${Math.round(index/count*100)}%…`);
    await command({op:'diagram_put',imageId:diagram.imageId,index,body:uri.slice(index*chunkSize,(index+1)*chunkSize)});
  }
  onProgress('Saving reviewed equipment…');
  await command({op:'diagram_save',diagram,equipment,baseRevision,...scope});
}
export async function scanDiagram(uri:string,endpoint:string,token:string) {
  const url=new URL(endpoint);
  if(url.protocol!=='https:' && !(url.protocol==='http:' && ['localhost','127.0.0.1','[::1]'].includes(url.hostname)))throw Error('Use an HTTPS scan server (HTTP is allowed only on this computer’s loopback address).');
  if(url.username||url.password||url.search||url.hash)throw Error('Use a scan server URL without credentials, query parameters or fragments.');
  if(!token.trim())throw Error('Enter the scan server access token.');
  url.pathname='/scan-diagram';
  const controller=new AbortController();const timeout=setTimeout(()=>controller.abort(),90000);
  try {
    // Equipment records and service history stay local. Matching runs on-device.
    const response=await fetch(url.toString(),{method:'POST',headers:{'Content-Type':'application/json',Authorization:`Bearer ${token.trim()}`},body:JSON.stringify({image:uri}),signal:controller.signal});
    if(!response.ok) {
      if(response.status===401)throw Error('The scan server access token was rejected.');
      if(response.status===429)throw Error('The scan server is busy. Please try again shortly.');
      throw Error('The diagram scan failed. Check the server and try again, or mark equipment manually.');
    }
    return parseDetections(await response.json());
  } catch(e) {
    if(e instanceof Error && e.name==='AbortError')throw Error('The scan timed out. Your current diagram has not changed.');
    throw e;
  } finally {clearTimeout(timeout);}
}
