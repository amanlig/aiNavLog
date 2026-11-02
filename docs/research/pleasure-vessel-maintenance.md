# Shared maintenance records — Canada and US pleasure vessels

Research date: October 30, 2026 (America/Vancouver).
Owner scope: private pleasure use in Canada and the US; shared Engine and
Electrical implementation. This is a product-design research baseline, not a
vessel-specific legal opinion or insurer acceptance of our records.

## Legal baseline

No general prescribed engine/electrical maintenance-log form was identified in
the federal recreational-boating sources reviewed. This is a bounded finding,
not a conclusion that every pleasure vessel has no recordkeeping duties. Vessel
size, flag, equipment, voyage and applicable state/provincial rules can change
the analysis. Commercial/charter/fishing operations and large-yacht pollution or
official logbooks need separate assessment.

- **Canada:** Small Vessel Regulations section 5 requires applicable safety
  equipment to be serviceable, accessible and maintained/replaced according to
  manufacturer guidance (with the stated life-raft exception). Section 201 sets
  the pleasure-craft operator's safety obligation. These do not prescribe a
  general engine/battery service-history table. Do not import manufacturer record
  duties or non-pleasure-craft provisions into the owner's ordinary service log.
  Source: [Small Vessel Regulations](https://laws-lois.justice.gc.ca/eng/regulations/SOR-2010-91/FullText.html).
  The retrieved consolidation was current to September 30, 2026; recheck before
  making a compliance claim.
- **US:** 33 CFR 175.310 requires serviceable portable extinguishers. Its special
  pre-1965 extinguisher provision includes inspection-tag/weight-check conditions.
  The recreational-vessel rule effective April 20, 2022 removed general NFPA 10
  monthly inspection and recordkeeping burdens; avoid presenting those commercial
  requirements as universal recreational obligations. Sources:
  [current section 175.310](https://www.ecfr.gov/current/title-33/chapter-I/subchapter-S/part-175/subpart-E/section-175.310)
  and [Coast Guard final rule](https://www.federalregister.gov/documents/2021/10/22/2021-22578/fire-protection-for-recreational-vessels).

## Manufacturer practice and format

There is no universal pleasure-boat service-log schema established by these
sources. A recognizable manufacturer format is Yanmar's maintenance log:
date, operating hours, work performed, dealer name and stamp/signature. The
current example is its [YM operation manual, printed page 100](https://www.yanmar.com/marine/wp-content/uploads/2020/03/0AYMM-M0002E_2025.02.pdf).
Yanmar also describes dealer-recorded service for its extended warranty:
[manufacturer warranty guidance](https://www.yanmar.com/marine/service/yanmar-warranty/).
These manufacturer conditions are distinct from general boating law.

Transport Canada's [sample maintenance schedule](https://tc.canada.ca/en/marine-transportation/marine-safety/sample-maintenance-schedule)
illustrates calendar and engine-hour triggers. It is published in the commercial
vessel guidance context, useful as a design reference rather than a prescribed
private-pleasure log. Use each equipment manual's intervals; do not hard-code
generic engine/battery service intervals as mandatory.

## Insurance evidence

No universal Canadian/US insurer-mandated maintenance-log format was established.
Actual obligations depend on the policy, endorsements and underwriting requests;
none of the owner's policy documents have been reviewed.

- **Canadian example:** Gallagher Skippers' Plan says some applicants must provide
  surveys/photos before coverage is issued, and explains contractual policy
  warranties. This supports recording the actual requirement and evidence, not
  inventing a universal survey interval.
  [Broker guidance](https://skippersplan.com/docs/).
- **US example:** BoatUS/GEICO's FAQ says its particular mechanical-breakdown
  endorsement requires manufacturer maintenance on the outdrive/outboard lower
  unit. This is specific to that coverage, not every engine/electrical claim.
  [Coverage FAQ](https://boatus.com/faq/boat-insurance).
- Brown & Brown recommends maintenance records and surveys for yacht underwriting,
  including confirmation that survey recommendations were addressed.
  [Broker's yacht-insurance guide](https://www.risk-strategies.com/hubfs/Navigating-the-Yacht-Insurance-Market-Brown-Brown_WEB.pdf?hsLang=en).

A log can support evidence of maintenance; it does not guarantee coverage,
replace required professional certification or prove all work was performed.
Verify the applicable policy before setting insurer-specific deadlines or
claiming an electronic record/export is accepted. No fixed universal retention
period or insurance discount is established by this research.

## Proposed shared product model

These are design recommendations inferred from the research, not mandated fields.

| Record | Proposed information |
| --- | --- |
| Equipment | Stable boat/equipment IDs, Engine/Electrical classification, name, manufacturer/model/serial; allow equipment relevant to both systems |
| Completed service | Work date, equipment, service/inspection/repair/replacement, description, outcome, performer/provider, optional meter reading with units and provenance |
| Supporting details | Parts/quantities, optional cost and currency, invoice/reference, notes and receipt/photo/report attachments |
| Planned task | Equipment, task, due date and/or equipment-hour threshold, source of interval; earliest applicable trigger wins when both apply |
| External requirement | Optional policy/survey/manual reference, recommendation ID, deadline, linked completed work and supporting evidence |
| Record provenance | Stable entry ID, creation time, author and visible correction history distinct from actual service date |

Use one reusable maintenance UI, service and SQLite model with Engine/Electrical
filtered entry points. Equipment-hour readings are optional and equipment-specific;
unknown is not zero, and BMV voltage/current samples do not imply engine hours.
A single completed job may satisfy several tasks or survey recommendations.
Planned work, an observed defect and completed work must remain distinguishable.

Manual maintenance entry is an application command: validate it in a shared
maintenance service and persist through Local data store. It does not pass through
the Bluetooth Device Interface. Data capture remains responsible for instrument
ingestion; history/maintenance reads use storage queries.

Initial workflow: select equipment → record completed work → review service
history, shared across both sections. Schema migration must preserve existing
telemetry. Design attachment retention/backup and readable export with the records;
SQLite alone on one phone is not a durable evidence/backup strategy. Scheduling,
notifications, signatures and full audit/export implementation require an explicit
feature scope before being marked complete.

Next design step: agree the initial fields and mockup, then implement the shared
maintenance flow. Research first was requested; no maintenance runtime or schema
change is included in this document update.


Implementation follow-up, October 30: the first shared completed-work flow is now
implemented in source. See [maintenance implementation](../maintenance-implementation.md)
for actual fields, schema-2 migration, verification and deferred attachments,
corrections/export/reminders. This supersedes the research-only checkpoint above.
