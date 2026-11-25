//! Diagram images are chunked to fit the native command bridge. Only reviewed
//! diagrams become current, atomically with equipment metadata. History is untouched.
use super::{maintenance::Equipment, storage, ObservationStore};
use ainavlog_data_model::AppResult;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Area {
    pub equipment_id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Diagram {
    pub revision: String,
    pub image_id: String,
    pub image_chunks: usize,
    pub width: u32,
    pub height: u32,
    pub areas: Vec<Area>,
}
fn identifier(s: &str) -> bool {
    !s.is_empty() && s.len() <= 80 && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}
impl Diagram {
    pub fn validate(&self) -> AppResult<()> {
        if !identifier(&self.revision)
            || !identifier(&self.image_id)
            || !(1..=512).contains(&self.image_chunks)
            || !(1..=20000).contains(&self.width)
            || !(1..=20000).contains(&self.height)
            || self.areas.len() > 60
        {
            return Err(storage("Invalid diagram dimensions or size"));
        }
        for a in &self.areas {
            if a.equipment_id.is_empty()
                || a.equipment_id.len() > 80
                || ![a.x, a.y, a.width, a.height].iter().all(|v| v.is_finite())
                || a.x < 0.0
                || a.y < 0.0
                || a.width <= 0.0
                || a.height <= 0.0
                || a.x + a.width > 1.000001
                || a.y + a.height > 1.000001
            {
                return Err(storage("Invalid equipment area"));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagramSummary {
    pub id: String,
    pub name: String,
}
fn scope(system: &str, id: &str) -> AppResult<()> {
    if !matches!(system, "engine" | "electrical") || !identifier(id) {
        return Err(storage("Invalid diagram system or ID"));
    }
    Ok(())
}
impl ObservationStore {
    pub fn read_diagram(&self) -> AppResult<Option<Diagram>> {
        self.read_system_diagram("electrical", "main")
    }
    pub fn list_diagrams(&self, system: &str) -> AppResult<Vec<DiagramSummary>> {
        scope(system, "main")?;
        let mut query = self
            .connection
            .prepare("SELECT id,name FROM maintenance_diagrams WHERE system=?1 ORDER BY rowid")
            .map_err(storage)?;
        let rows = query
            .query_map([system], |r| {
                Ok(DiagramSummary {
                    id: r.get(0)?,
                    name: r.get(1)?,
                })
            })
            .map_err(storage)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage)?;
        Ok(rows)
    }
    pub fn read_system_diagram(&self, system: &str, id: &str) -> AppResult<Option<Diagram>> {
        scope(system, id)?;
        let body: Option<String> = self
            .connection
            .query_row(
                "SELECT body FROM maintenance_diagrams WHERE system=?1 AND id=?2",
                params![system, id],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage)?;
        body.map(|s| serde_json::from_str(&s).map_err(storage))
            .transpose()
    }
    fn image_in_use(&self, id: &str) -> AppResult<bool> {
        self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM maintenance_diagrams WHERE json_extract(body,'$.imageId')=?1)",
            [id], |r| r.get(0)
        ).map_err(storage)
    }
    pub fn put_diagram_chunk(&self, id: &str, index: usize, body: &str) -> AppResult<()> {
        if !identifier(id)
            || index >= 512
            || body.is_empty()
            || body.len() > 16000
            || !body.is_ascii()
        {
            return Err(storage("Invalid image chunk"));
        }
        // Uploaded assets are immutable, including retries and already-published images.
        self.connection
            .execute(
                "INSERT INTO electrical_diagram_assets VALUES(?1,?2,?3) ON CONFLICT DO NOTHING",
                params![id, index as i64, body],
            )
            .map_err(storage)?;
        let saved: String = self
            .connection
            .query_row(
                "SELECT body FROM electrical_diagram_assets WHERE asset_id=?1 AND chunk_index=?2",
                params![id, index as i64],
                |r| r.get(0),
            )
            .map_err(storage)?;
        if saved != body {
            return Err(storage("Image ID already contains different data"));
        }
        Ok(())
    }
    pub fn read_diagram_chunk(&self, id: &str, index: usize) -> AppResult<String> {
        if !identifier(id) || index >= 512 {
            return Err(storage("Invalid image request"));
        }
        self.connection
            .query_row(
                "SELECT body FROM electrical_diagram_assets WHERE asset_id=?1 AND chunk_index=?2",
                params![id, index as i64],
                |r| r.get(0),
            )
            .map_err(storage)
    }
    pub fn discard_diagram_asset(&self, id: &str) -> AppResult<()> {
        if !identifier(id) {
            return Err(storage("Invalid image ID"));
        }
        if self.image_in_use(id)? {
            return Err(storage("Cannot discard the current diagram"));
        }
        self.connection
            .execute(
                "DELETE FROM electrical_diagram_assets WHERE asset_id=?1",
                [id],
            )
            .map_err(storage)?;
        Ok(())
    }
    pub fn save_diagram(
        &self,
        diagram: &Diagram,
        equipment: &[Equipment],
        base: Option<&str>,
    ) -> AppResult<()> {
        self.save_system_diagram(
            "electrical",
            "main",
            "Electrical diagram",
            diagram,
            equipment,
            base,
        )
    }
    pub fn save_system_diagram(
        &self,
        system: &str,
        id: &str,
        name: &str,
        diagram: &Diagram,
        equipment: &[Equipment],
        base: Option<&str>,
    ) -> AppResult<()> {
        scope(system, id)?;
        if name.trim().is_empty() || name.chars().count() > 120 {
            return Err(storage("Enter a diagram name (up to 120 characters)"));
        }
        diagram.validate()?;
        if equipment.len() > 60 {
            return Err(storage("Too many equipment records"));
        }
        for e in equipment {
            e.validate()?;
            if e.system != system && e.system != "both" {
                return Err(storage(
                    "Diagram equipment must belong to the selected system",
                ));
            }
        }
        let transaction = self.connection.unchecked_transaction().map_err(storage)?;
        let current = self.read_system_diagram(system, id)?;
        if current.is_none() && self.list_diagrams(system)?.len() >= 50 {
            return Err(storage("Up to 50 diagrams per system are supported"));
        }
        // Exact retry after a lost acknowledgement succeeds without duplicate equipment.
        if current.as_ref() == Some(diagram) {
            let snapshot = self.maintenance_snapshot()?;
            let saved_name: String = transaction
                .query_row(
                    "SELECT name FROM maintenance_diagrams WHERE system=?1 AND id=?2",
                    params![system, id],
                    |r| r.get(0),
                )
                .map_err(storage)?;
            if saved_name == name
                && equipment
                    .iter()
                    .all(|e| snapshot.equipment.iter().any(|saved| saved == e))
            {
                return Ok(());
            }
            return Err(storage("Diagram was already saved with different equipment details. Reload before editing."));
        }
        if current.as_ref().map(|d| d.revision.as_str()) != base {
            return Err(storage(
                "Diagram changed. Reload before saving your update.",
            ));
        }
        let mut query = transaction.prepare("SELECT chunk_index,body FROM electrical_diagram_assets WHERE asset_id=?1 ORDER BY chunk_index").map_err(storage)?;
        let chunks = query
            .query_map([&diagram.image_id], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(storage)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage)?;
        if chunks.len() != diagram.image_chunks
            || chunks
                .iter()
                .enumerate()
                .any(|(i, (index, _))| i as i64 != *index)
        {
            return Err(storage("Diagram image upload is incomplete"));
        }
        let image: String = chunks.into_iter().map(|(_, chunk)| chunk).collect();
        let payload = image
            .strip_prefix("data:image/jpeg;base64,")
            .or_else(|| image.strip_prefix("data:image/png;base64,"))
            .ok_or_else(|| storage("Only JPEG and PNG diagrams are supported"))?;
        if payload.len() > 8_000_000
            || payload.is_empty()
            || payload.len() % 4 != 0
            || !payload
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"+/=".contains(&b))
        {
            return Err(storage("Invalid diagram image"));
        }
        drop(query);
        for e in equipment {
            let existing: Option<(String, String)> = transaction
                .query_row(
                    "SELECT name,system FROM maintenance_equipment WHERE id=?1",
                    [&e.id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()
                .map_err(storage)?;
            if let Some((name, system)) = existing {
                if name != e.name || system != e.system {
                    return Err(storage(
                        "Existing equipment identity changed; reload and match again",
                    ));
                }
            } else {
                transaction
                    .execute(
                        "INSERT INTO maintenance_equipment VALUES(?1,?2,?3)",
                        params![e.id, e.name, e.system],
                    )
                    .map_err(storage)?;
            }
            transaction.execute("INSERT INTO maintenance_equipment_details(equipment_id,category,model,serial_number,make,manufactured_date,installed_date) VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(equipment_id) DO UPDATE SET category=excluded.category,model=excluded.model,serial_number=excluded.serial_number,make=excluded.make,manufactured_date=excluded.manufactured_date,installed_date=excluded.installed_date",params![e.id,e.category,e.model,e.serial_number,e.make,e.manufactured_date,e.installed_date]).map_err(storage)?;
        }
        for area in &diagram.areas {
            let linked_system: String = transaction
                .query_row(
                    "SELECT system FROM maintenance_equipment WHERE id=?1",
                    [&area.equipment_id],
                    |r| r.get(0),
                )
                .map_err(storage)?;
            if linked_system != system && linked_system != "both" {
                return Err(storage("Equipment area links to another system"));
            }
        }
        transaction.execute("INSERT INTO maintenance_diagrams(system,id,name,body) VALUES(?1,?2,?3,?4) ON CONFLICT(system,id) DO UPDATE SET name=excluded.name,body=excluded.body",params![system,id,name,serde_json::to_string(diagram).map_err(storage)?]).map_err(storage)?;
        if let Some(old) = current.filter(|d| d.image_id != diagram.image_id) {
            if !self.image_in_use(&old.image_id)? {
                transaction
                    .execute(
                        "DELETE FROM electrical_diagram_assets WHERE asset_id=?1",
                        [old.image_id],
                    )
                    .map_err(storage)?;
            }
        }
        transaction.commit().map_err(storage)
    }
}
