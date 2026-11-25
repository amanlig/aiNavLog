import {MaintenanceDiagram} from './MaintenanceDiagram';
import type {Snapshot} from '../maintenance/model';
export function ElectricalDiagram(props:{data:Snapshot;onSaved:()=>Promise<void>}) {
  return <MaintenanceDiagram system="electrical" {...props}/>;
}
