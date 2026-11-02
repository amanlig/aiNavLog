// Compatibility entry point; both systems use the shared maintenance component.
import { MaintenanceRecords } from './MaintenanceRecords';
export function ElectricalRecords() { return <MaintenanceRecords system="electrical" />; }
