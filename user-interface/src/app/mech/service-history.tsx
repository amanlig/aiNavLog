import {useLocalSearchParams} from 'expo-router';
import {MaintenanceRecords} from '../../components/MaintenanceRecords';
export default function EquipmentServiceHistory() {
  const {equipmentId}=useLocalSearchParams<{equipmentId?:string}>();
  return <MaintenanceRecords key={equipmentId??'missing'} system="engine" historyOnly initialEquipmentId={typeof equipmentId==='string'?equipmentId:'__missing-equipment__'}/>;
}
