//! Persistent maintenance records. Dates are local calendar dates, hours are manual.
use super::{storage, ObservationStore};
use ainavlog_data_model::AppResult;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Equipment {
    pub id: String,
    pub name: String,
    pub system: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServiceEntry {
    pub id: String,
    pub equipment_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub date: String,
    pub kind: String,
    pub description: String,
    pub provider: String,
    pub hours: Option<f64>,
    pub parts: String,
    pub cost_minor: Option<i64>,
    pub currency: String,
    pub reference: String,
    pub notes: String,
    pub next_date: Option<String>,
    pub next_hours: Option<f64>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedEntry {
    #[serde(flatten)]
    pub entry: ServiceEntry,
    pub created_at: i64,
}
#[derive(Serialize)]
pub struct Snapshot {
    pub equipment: Vec<Equipment>,
    pub entries: Vec<SavedEntry>,
}
fn text(value: &str, max: usize, required: bool) -> bool {
    value.len() <= max && (!required || !value.trim().is_empty()) && !value.contains('\0')
}
pub fn valid_date(value: &str) -> bool {
    let b = value.as_bytes();
    if b.len() != 10
        || b[4] != b'-'
        || b[7] != b'-'
        || b.iter()
            .enumerate()
            .any(|(i, c)| i != 4 && i != 7 && !c.is_ascii_digit())
    {
        return false;
    }
    let year = value[0..4].parse::<u32>().unwrap_or(0);
    let month = value[5..7].parse::<usize>().unwrap_or(0);
    let day = value[8..10].parse::<u32>().unwrap_or(0);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = [
        0,
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    (1900..=9999).contains(&year) && (1..=12).contains(&month) && day > 0 && day <= days[month]
}
impl Equipment {
    pub fn validate(&self) -> AppResult<()> {
        if !text(&self.id, 80, true)
            || !text(&self.name, 120, true)
            || !matches!(self.system.as_str(), "engine" | "electrical" | "both")
        {
            return Err(storage("invalid equipment"));
        }
        Ok(())
    }
}
impl ServiceEntry {
    pub fn validate(&self) -> AppResult<()> {
        if !text(&self.id, 80, true)
            || self
                .system
                .as_ref()
                .is_some_and(|s| !matches!(s.as_str(), "engine" | "electrical"))
            || !text(&self.equipment_id, 80, true)
            || !valid_date(&self.date)
            || !matches!(
                self.kind.as_str(),
                "Service" | "Inspection" | "Repair" | "Replacement"
            )
            || !text(&self.description, 2000, true)
            || !text(&self.provider, 200, false)
            || !text(&self.parts, 2000, false)
            || !text(&self.reference, 200, false)
            || !text(&self.notes, 4000, false)
            || self
                .hours
                .is_some_and(|h| !h.is_finite() || !(0.0..=1e9).contains(&h))
            || self.next_hours.is_some_and(|h| {
                !h.is_finite()
                    || !(0.0..=1e9).contains(&h)
                    || self.hours.is_some_and(|current| h <= current)
            })
            || self
                .next_date
                .as_ref()
                .is_some_and(|d| !valid_date(d) || d < &self.date)
            || self
                .cost_minor
                .is_some_and(|c| !(0..=999_999_999_999).contains(&c))
            || !self.currency.is_empty()
                && (self.currency.len() != 3
                    || !self.currency.bytes().all(|c| c.is_ascii_uppercase()))
            || self.cost_minor.is_some() && self.currency.is_empty()
        {
            return Err(storage(
                "invalid service record; check dates, hours and cost/currency",
            ));
        }
        Ok(())
    }
}
impl ObservationStore {
    pub fn maintenance_snapshot(&self) -> AppResult<Snapshot> {
        let mut equipment_query = self
            .connection
            .prepare("SELECT id,name,system FROM maintenance_equipment ORDER BY name,id")
            .map_err(storage)?;
        let equipment = equipment_query
            .query_map([], |r| {
                Ok(Equipment {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    system: r.get(2)?,
                })
            })
            .map_err(storage)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage)?;
        let mut query=self.connection.prepare("SELECT body,created_at FROM maintenance_entries ORDER BY service_date DESC,created_at DESC,id DESC LIMIT 10001").map_err(storage)?;
        let rows = query
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
            .map_err(storage)?;
        let mut entries = Vec::new();
        for row in rows {
            let (body, created_at) = row.map_err(storage)?;
            entries.push(SavedEntry {
                entry: serde_json::from_str(&body).map_err(storage)?,
                created_at,
            });
        }
        if entries.len() > 10000 {
            return Err(storage(
                "maintenance history exceeds current display limit; no records omitted silently",
            ));
        }
        Ok(Snapshot { equipment, entries })
    }
    pub fn save_equipment(&self, equipment: &Equipment) -> AppResult<()> {
        equipment.validate()?;
        self.connection.execute("INSERT INTO maintenance_equipment(id,name,system) VALUES (?1,?2,?3) ON CONFLICT(id) DO NOTHING",params![equipment.id,equipment.name,equipment.system]).map_err(storage)?;
        let saved = self
            .connection
            .query_row(
                "SELECT name,system FROM maintenance_equipment WHERE id=?1",
                [&equipment.id],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .map_err(storage)?;
        if saved != (equipment.name.clone(), equipment.system.clone()) {
            return Err(storage("equipment ID already belongs to another item"));
        }
        Ok(())
    }
    pub fn save_service(&self, entry: &ServiceEntry) -> AppResult<()> {
        entry.validate()?;
        if let Some(system) = &entry.system {
            let equipment_system: String = self
                .connection
                .query_row(
                    "SELECT system FROM maintenance_equipment WHERE id=?1",
                    [&entry.equipment_id],
                    |r| r.get(0),
                )
                .map_err(storage)?;
            if equipment_system != "both" && equipment_system != *system {
                return Err(storage(
                    "equipment does not belong to the selected maintenance log",
                ));
            }
        }
        let body = serde_json::to_string(entry).map_err(storage)?;
        let now = super::unix_millis(std::time::SystemTime::now())?;
        self.connection.execute("INSERT INTO maintenance_entries(id,equipment_id,service_date,created_at,body) VALUES (?1,?2,?3,?4,?5) ON CONFLICT(id) DO NOTHING",params![entry.id,entry.equipment_id,entry.date,now,body]).map_err(storage)?;
        let saved: String = self
            .connection
            .query_row(
                "SELECT body FROM maintenance_entries WHERE id=?1",
                [&entry.id],
                |r| r.get(0),
            )
            .map_err(storage)?;
        if saved != body {
            return Err(storage("service ID already belongs to another record"));
        }
        Ok(())
    }
}
