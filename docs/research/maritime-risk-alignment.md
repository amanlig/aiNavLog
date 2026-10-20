# Maritime industry alignment and future marketing

**Direction agreed:** October 12, 2026. Align aiNavLog with maritime operational
needs and collect evidence for a later marketing project. This document records
product direction and research interpretation, not demonstrated safety outcomes.

## Research basis

[ShipIn / NorthStandard, The Maritime Risk Report 2026](https://info.shipin.ai/hubfs/1-Pagers/shipin-north-standard-risk-report%20-%20final.pdf)
describes recurring operational deviations, concentrated vessel risk and the value
of timely observation followed by corrective action (pages 2–10). Its principal
examples concern watchkeeping, mooring, deck safety and PPE. Page 12 describes
separate incident datasets, deployment cohorts and customer case studies.

Our interpretation: the observe → recognize a recurring condition → act → review
loop also offers a useful design direction for instrument monitoring. The report
does not establish that aiNavLog reduces incidents, claims or costs. Its commercial
fleet findings cannot automatically be generalized to our intended boat users.

## Relevance and capability boundaries

| Operational need | aiNavLog contribution | Evidence or gap |
| --- | --- | --- |
| Recognize developing equipment problems | Electrical and future engine telemetry, timestamps and configured alerts | Simulated electrical data reaches the rendered UI; physical accuracy and phone-local integration pending |
| See repeated warning conditions | Persist events and show frequency and duration | Observation persistence exists; event/recurrence workflows proposed |
| Support follow-up and handover | Acknowledgement, action notes and resolution history | Proposed operator workflow, not implemented or a compliance system |
| Retain navigation context | Associate equipment events with recorded route/time where available | Route recording planned; no navigation-risk detection claim |
| Observe unsafe crew behaviour | Current instrument readings cannot establish watch presence, distraction, PPE or mooring safety | No implemented coverage |
| Prioritize risk across commercial fleets | Would require comparable fleet data and validated measures | Outside current local-data scope |

## Product requirements

Extend Electrical and Engine with a common local event model as their measurement
and alert contracts mature. Preserve the six-workstream plan; estimate this work at
slice review and record any schedule change explicitly.

1. **Meaningful event history:** record configured threshold crossings, start/end
   times, duration, source/boat/battery identity, units and the threshold used.
   Keep monitoring interruptions and recovery distinguishable from equipment
   conditions. Missing or stale data is not evidence of a healthy condition or a
   threshold violation. Define debounce/hysteresis before generating alerts.
2. **Response tracking:** allow acknowledgement, timestamped action notes and
   resolution. Acknowledgement does not resolve a condition. Keep operator-entered
   resolution separate from sensor-observed return to normal; retain the history.
   Use the agreed ownership/access policy when identifying an operator.
3. **Recurrence:** show repeat counts and durations over an explicit time window,
   alongside monitoring coverage/gaps. Avoid treating periods without reception
   as zero events. Retain simulated/live provenance and original timestamps.

Acceptance should exercise repeat crossings without alert spam, missing/stale
data, recovery, restart persistence, acknowledgement versus resolution and history
retention. Physical reference checks and usability trials remain separate evidence.
Do not infer engine thresholds before agreeing the supported engine and quantities.

## Future marketing project

**Status:** Planned later; no campaign or publication initiated.

Working positioning to validate: **aiNavLog helps boat operators recognize
developing equipment problems and retain the context needed to act earlier.**
Present-day demonstrations must identify simulator use and the verified platform.
Trend/event/response benefits remain planned until those workflows are implemented.

Before choosing channels or publishing claims:

- Identify the first customer segment and interview operators about instrument
  checks, missed warnings, maintenance follow-up and handover needs. Maritime
  alignment does not itself establish commercial-fleet suitability.
- Run consented trials on representative hardware and boats. Measure reading
  accuracy, monitoring coverage, alert latency, false alerts, recurrence visibility
  and whether operators can understand and respond to events.
- Maintain a claim-to-evidence record: capability, source/build/platform, simulated
  or physical conditions, result, limitations and approved wording.
- Use validated demonstrations and case studies to develop messaging. Any claim
  about prevented incidents, reduced costs, insurance benefits or regulatory
  compliance needs separate evidence; ShipIn's figures are not aiNavLog outcomes.

See the [implementation plan](../project-plan.md) and
[future-work backlog](../backlog.md).
