# aiNavLog vertical implementation plan

**Created:** October 5, 2026. **Rebaselined:** October 16, 2026.
**Target:** implementation foundation for the 2027 mobile beta. Dates use
America/Vancouver. Feature windows are two calendar weeks, with Friday reviews.
These are planning targets, not confirmed delivery or public-store release dates.

## Schedule

| Slice | Revised window | Demo / review milestone | Working outcome |
| --- | --- | --- | --- |
| 1. Electrical — native completion | Oct 19–Nov 1, 2026; desktop/core/UI work from Oct 5–18 retained | Fri Oct 23 native packaging gate; Fri Oct 30 end-to-end review | BMV simulator Bluetooth → phone receiver → Rust decoder → phone SQLite → House/Starter UI, with failure and restart checks |
| 2. Store deployment / beta distribution — parallel | Initial build/install milestone achieved Oct 16; updates and qualification ongoing | Fri Oct 30 native-build update review; Fri Nov 13 cross-platform evidence review | Reuse signed build paths; validate updates and data retention once native storage exists; finish Android runtime/device checks |
| 2a. Android + iOS CI/CD — parallel deployment task | Oct 19–23, 2026 | Fri Oct 23 pipeline demonstration | Automated checks and controlled signed builds; Android internal testing and iOS TestFlight delivery paths, subject to store/API credential readiness |
| 3. Engine | Nov 2–15, 2026 | Fri Nov 13 | Selected measurements through the shared pipeline; simulated-only scope if hardware/gateway remains unresolved |
| 4. Route | Nov 16–29, 2026 | Fri Nov 27 | Record, save, reopen and display a route; permissions and location gaps explicit |
| 5. Authentication / authorization | Nov 30–Dec 13, 2026 | Fri Dec 11 | Agreed ownership policy, provider/session flows and enforced local-record access |
| Review / holiday contingency — no feature commitment | Dec 14, 2026–Jan 3, 2027 | Availability and remaining-scope review by Fri Dec 11 | Reviews, rework and optional hardening as capacity permits; no assumed full holiday staffing |
| 6. Route story | Jan 4–17, 2027 | Fri Jan 15 | Select saved route/media, preview and explicitly export/share |
| Integration / beta-readiness review | Jan 18–31, 2027 | Fri Jan 29 | Consolidated regression, installation/update, privacy/accessibility and hardware-evidence review; record remaining release blockers |

### Gantt chart

![aiNavLog revised schedule](images/project-plan-gantt.svg)

[Open the full-size chart](images/project-plan-gantt.svg). The Mermaid version below
is editable alongside this plan. Task end dates in Mermaid are exclusive, so
Nov 2 represents work through Nov 1. Review diamonds occur within their windows.

```mermaid
gantt
    title aiNavLog — October 16 revised baseline
    dateFormat YYYY-MM-DD
    axisFormat %b %d
    todayMarker off
    section Achieved baseline
    Desktop/core/UI foundation          :done, baseline, 2026-10-05, 2026-10-17
    First Android and iPhone installs    :milestone, done, installed, 2026-10-16, 0d
    section Parallel deployment
    Android and iOS CI/CD                :cicd, 2026-10-19, 2026-10-24
    Pipeline review                     :milestone, cicd_review, 2026-10-23, 0d
    Distribution and update checks      :delivery, 2026-10-19, 2027-02-01
    section Feature implementation
    Native Electrical completion        :electrical, 2026-10-19, 2026-11-02
    Native packaging gate               :milestone, native_gate, 2026-10-23, 0d
    Electrical demo                     :milestone, electrical_demo, 2026-10-30, 0d
    Engine                              :engine, 2026-11-02, 2026-11-16
    Engine demo                         :milestone, engine_demo, 2026-11-13, 0d
    Route                               :route, 2026-11-16, 2026-11-30
    Route demo                          :milestone, route_demo, 2026-11-27, 0d
    Authentication and authorization    :auth, 2026-11-30, 2026-12-14
    Authentication demo                 :milestone, auth_demo, 2026-12-11, 0d
    section Contingency
    Review and holiday buffer           :buffer, 2026-12-14, 2027-01-04
    section January completion
    Route story                         :story, 2027-01-04, 2027-01-18
    Route story demo                    :milestone, story_demo, 2027-01-15, 0d
    Integration and beta readiness      :readiness, 2027-01-18, 2027-02-01
    Beta-readiness review               :milestone, readiness_review, 2027-01-29, 0d
```

The delivery bar spans the programme but does not assume full holiday staffing.
The Oct 23 review checks both CI/CD progress and its impact on native Electrical.
Public-store release is outside these scheduled milestones.

### October 16 checkpoint and reason for change

- **Android distribution:** signed 1.0.0/code 1 accepted by Google Play internal
  testing; owner confirmed emulator installation and supplied Home launch evidence.
- **iOS distribution:** Apple membership active; signed 1.0.0/build 1 reached
  TestFlight; owner confirmed physical iPhone 13 installation and passed Electrical
  demo, navigation and close/reopen checks.
- **Electrical integration:** desktop/web replay and host-tested Rust/C bridge
  remain verified. The released mobile demo does not include native Rust/Bluetooth
  reception or phone-local SQLite. The original Oct 16 end-to-end milestone is
  therefore **not complete** and moves to Oct 30.

The revised baseline adds two focused weeks for native Electrical completion,
then shifts Engine, Route and Authentication two weeks later. Route Story moves
to January rather than depending on uninterrupted holiday capacity. The original
Oct 5–Dec 13 schedule remains historical; it is superseded by this table.

Jan 29 is a **beta-readiness review**, not a promised public release. Physical
BMV/engine access, device coverage, native integration, event-history scope and
store processing may require further work. Reassess dates at Oct 23 and Oct 30;
do not move to Engine with an unaccepted Electrical gap. No new cloud build,
submission or purchase is triggered by this plan update.

### Immediate execution gates

| Target | Evidence required |
| --- | --- |
| Oct 23 | Android/iOS CI/CD: passing check run, traceable signed build on each platform and gated internal distribution demonstrated; any missing submission credential remains explicit |
| Oct 23 | Rust target-library packaging and Expo native adapter build feasibility demonstrated; platform permission/scanner design chosen; Electrical event-history additions sized |
| Oct 30 | Physical iPhone simulator-to-native-UI path, denied Bluetooth/stale/recovery behaviour, and phone-local persistence across restart; remaining Android/physical-BMV gaps explicitly recorded |
| Oct 30 | Deliver a native-enabled build through established distribution; verify updating the installed demo. Claim stored-reading retention only after phone storage exists and an update test passes |
| Nov 13 | Review Android native/device evidence independently; do not treat the emulator's install or iPhone pass as Android radio qualification |

## Delivery approach

**Maritime alignment — agreed October 12:** extend Electrical and Engine with
meaningful local event history, operator response tracking and recurrence views
as their measurement/alert contracts mature. Use these to support earlier recognition
and follow-up of equipment conditions. This is planned work, not demonstrated
incident reduction. Estimate the additions at slice review and explicitly revise
dates if needed. A later marketing project will validate the customer segment and
build claims from product evidence. See [research and requirements](research/maritime-risk-alignment.md).

Each slice connects source → decode/interpret → local persistence → UI, with
permission/error handling and a repeatable demo. Week one establishes the smallest
working path; week two completes behavior, verifies failures and persistence,
and prepares the demo. The deployment workstream begins alongside Electrical
and supports every subsequent slice. Start the next feature slice after the
preceding feature demo works or its remaining gaps have been explicitly recorded
and accepted; track distribution blockers separately.

Existing HTML pages are design references. The product direction remains React
Native on iOS and Android; Windows simulators and Rust CLI tools support development.
Prove the first electrical path on the available iPhone, with Android build and
functional verification included when a test device/emulator is available. Missing
platform verification must stay visible as a beta qualification gap.

## 1. Electrical

**Status — October 16, 2026:** **Native completion in progress; Oct 16
end-to-end goal rebaselined to Oct 30.** The mobile shell is distributed and
installed. The iPhone 13 demo, navigation and reopen checks passed by owner
confirmation. Demo readings do not establish Bluetooth reception or persistence.

**Prerequisites resolved:** Apple Developer membership is active; Google account
verification and first signed build/distribution paths are established. Both
platforms use `com.kodekomplete.com.ainavlog`. iPhone install is no longer the
blocking prerequisite. Native packaging, receiver/adapter and storage are next.

**Oct 19–25:** package the shared Rust core for mobile targets, implement the Expo
native boundary and platform Bluetooth receiver, and pass one decoded sample to
phone-local SQLite and `/elec`. Verify the iOS signing/build path includes the
native components. Preserve source identity, nullable measurements and original
receipt time. Define ownership boundaries and size event/response/recurrence work
at the Oct 23 gate; do not silently expand the two-week commitment.

**Oct 26–Nov 1:** complete House/Starter behaviour, freshness/recovery and
permissions checks on native data; verify restart persistence and a native-enabled
app update. Review Oct 30 with explicit iOS/Android and physical/simulator evidence.
Starter auxiliary trends and configured voltage alerts remain acceptance work;
unavailable current/SOC must not be fabricated. If the native packaging gate fails,
replan downstream dates at that point rather than treating demo mode as completion.

**Done:** Windows BMV simulator → phone Bluetooth → BMV decoder → local store →
electrical UI works end to end. House shows volts, signed amps and charge %;
Starter shows auxiliary volts, trend and alert status. Starter amps and SOC are
unavailable, never invented or shown as zero. The selector changes views, not
electrical wiring. Alert checks use configured thresholds and unavailable readings
cannot trigger fabricated alerts.

**Baseline:** BMV decoding, normalization, SQLite persistence and synthetic fixtures
already pass software tests. Simulated BLE reception on iPhone has been demonstrated.
Starter auxiliary-voltage simulation was added and fixture-checked on October 6.
The local desktop/web path from captured simulator packets through the Rust monitor,
SQLite and HTTP to the rendered electrical page was verified on October 6, including
changing SOC, stale hiding and recovery. Native mobile reception/bridge integration
remains to implement. No local Mac is available; EAS cloud builds now provide
the signed iOS build path. Apple membership, signing and the first iPhone 13
UI/demo installation are confirmed as of October 16. See the
[mobile workflow](mobile-development-workflow.md).
One BMV-712 permanently monitors the house shunt; its auxiliary input measures
starter voltage. Physical BMV accuracy remains unverified. See the
[physical architecture](orion-physical-architecture.md).

**October 6 checkpoint:** the user confirmed the local Device Interface-to-UI
objective is met. Services were stopped at wrap-up. Resume October 7 with the
verified replay baseline, then begin the native mobile reception/Rust integration
and resolve build prerequisites. See [session handoff](status/status-2026-10-06.md).

**October 8 continuation:** preparing the native boundary. The Rust BMV decoder,
monitor, normalizer and SQLite store can now build with `--no-default-features`,
without desktop Bluetooth/D-Bus or NMEA dependencies. Default desktop behavior
is retained, and NMEA remains separately available. This is a host-tested core
boundary, not a native module or a phone build. The user confirmed no Mac and has
initiated the D‑U‑N‑S process for Apple Developer organization enrollment. Next:
the Expo native module/FFI and target packaging while enrollment progresses.
See [mobile workflow](mobile-development-workflow.md#october-8-continuation--shared-rust-core).

**October 8 handoff:** shared-core preparation and the simulator-to-screen
regression check are complete. Apple/Google organization setup awaits D‑U‑N‑S.
Continue native integration and release-environment preparation in parallel.
See [accomplishments and next steps](status/status-2026-10-08.md).

### Electrical vertical-slice sequence

The slice starts when the boater opens `/elec` and extends through the readings
adapter to Device Interface, its decoder and local persistence. Instrument
reception supplies this path continuously; opening the page reads the latest
stored packet rather than commanding a new BMV measurement. Device Interface
supplies data, and `ElectricalScreen` returns and updates the React Native view.

```mermaid
sequenceDiagram
    actor Boater
    participant Page as UI: ElectricalScreen (/elec)
    participant Client as UI: electrical data client
    participant Bridge as Readings adapter (HTTP now, native planned)
    participant Store as Device Interface: local SQLite
    participant Monitor as Device Interface: reception / decoder / normalizer
    participant Source as BMV or Windows BMV simulator

    Note over Monitor,Source: Reception lifecycle runs independently of opening the page
    Boater->>Page: Open /elec
    Page->>Client: Subscribe to sample and availability state
    Page-->>Boater: Render battery cards with waiting state
    par Receive and persist observations
        loop While instrument monitoring is active
            Source->>Monitor: Encrypted Bluetooth advertisement
            Note over Source,Monitor: Verified replay injects captured packets here. Phone radio path remains pending
            Monitor->>Monitor: Decode BMV fields and normalize units / quality
            Monitor->>Store: Atomically save packet and observations
            Store-->>Monitor: Commit success or storage failure
        end
    and Read and render the electrical view
        loop While the electrical page is foregrounded
            Client->>Bridge: Request latest sample (GET /electrical every second today)
            Bridge->>Store: Read latest complete BMV packet for selected device
            Store-->>Bridge: Nullable measurements and original receipt time
            Bridge-->>Client: Sample with source label, no sample, or error
            Client->>Client: Validate sample and evaluate freshness
            Client-->>Page: Readings and availability / error state
            alt Fresh sample
                Page-->>Boater: House volts / amps / SOC and starter volts
            else No sample, stale sample, or request failure
                Page-->>Boater: Hide unavailable readings and show waiting / retry state
            end
        end
    end
```

**Verified implementation:** captured encrypted packets from the existing Windows
BMV simulator → production Rust monitor/decoder/normalizer → SQLite → local
`serve-ui` HTTP endpoint → UI polling/validation → rendered `/elec`. The browser
test checks changing values, stale hiding and recovery. Simulator readings are
explicitly labelled, and starter amps/SOC are unavailable.

**Remaining mobile milestone:** Windows BMV Bluetooth broadcast → phone reception
→ native Rust bridge/decoder → phone-local persistence → `/elec`, including denied
permissions, Bluetooth unavailable, recovery and persistence across app restart.
The native adapter's API and update mechanism remain to be selected; the diagram's
current one-second HTTP polling does not commit the mobile design to HTTP.

See [architecture section 5.4.1](../architecture.md#541-user-interface) for component
responsibilities and the sample contract, and [the root README](../README.md#demonstrate-simulator-data-reaching-the-ui)
for the repeatable demonstration and browser test.

## 2. Store deployment and beta distribution

**Status — October 16, 2026:** **First signed distribution/install milestone
achieved; repeat-update and native-data retention qualification remain open.**
Google Play internal release 1.0.0/code 1 is accepted; Android emulator installation
and Home launch are confirmed. Apple membership is active; TestFlight build
1.0.0 (1) was installed on a physical iPhone 13, and the owner confirmed Electrical
demo/navigation/reopen checks passed. Installed iOS version and app build number
were not separately read from the phone. Android physical-device and full runtime
checks have not been reported.

The shared package/bundle ID is `com.kodekomplete.com.ainavlog`. App Store Connect
ID is `6823786033`. EAS builds both platforms; Android uses manual Play Console
AAB upload, while iOS uses EAS Submit and TestFlight. Preserve the accepted signing
keys/team and verify counters before future builds. The initial Android signing
mismatch is historical and resolved, not a current blocker.

See [October 16 evidence](status/status-2026-10-16.md),
[build/deployment architecture](mobile-build-deployment.md),
[Android deployment](android-manual-deployment.md) and
[iOS workflow](ios-testflight.md).

**Objective:** establish an environment where a tested change can be built, installed
and released to testers quickly and repeatedly. This workstream starts with the app
shell and continues through all feature slices; it does not wait for Electrical completion.

**Initial setup (Oct 5–16):** completed account/signing configuration, scoped UI
upload wrappers, signed artifacts, Android emulator install and physical iPhone
UI/demo validation. These are build/install milestones, not native Electrical completion.

**Oct 19–Nov 1:** distribute the native Electrical increment, confirm installed-app
updates on iPhone and the Android test environment, and test durable readings
across restart/update once phone storage is implemented. Record Android physical
hardware and untested lifecycle conditions separately. Target the native-update
review for Oct 30; cross-platform qualification review follows Nov 13.

### Next week: Android and iOS CI/CD (Oct 19–23)

**October 20 implementation update:** shared UI check workflow and manual iOS EAS build / optional TestFlight workflow are prepared locally. Typecheck, lint and tests pass locally. Publication, CI credentials, first hosted run and iPhone update evidence remain pending. Android delivery automation follows the iOS proof. See [CI/CD setup](mobile-ci-cd.md).

Continue this under deployment workstream 2, in parallel
with native Electrical. Proposed tooling is GitHub Actions for repository checks
and orchestration, with Expo EAS Build/Submit for native builds and store uploads.
Confirm repository access and runner/credential requirements during implementation.
The existing scoped UI wrappers and accepted signing identities are the baseline.

| Day / checkpoint | Planned work |
| --- | --- |
| Mon Oct 19 | Define workflow triggers, paths, environments, artifact scope and credential ownership; establish protected internal-release configuration |
| Tue Oct 20 | CI: UI typecheck/lint/tests and relevant JS exports; Rust formatting/Clippy/tests and bridge smoke checks on affected changes |
| Wed Oct 21 | CD build: signed Android AAB and iOS IPA through EAS; deliberate version/build increments, serialized releases and traceable artifact/build IDs |
| Thu Oct 22 | Connect gated delivery to Google Play internal testing and App Store Connect/TestFlight; preserve manual fallback where store API access is unfinished |
| Fri Oct 23 | Demonstrate check failure blocking delivery, successful builds, approved internal delivery and failure reporting; record remaining gaps and capacity impact |

**CI behaviour:** run on relevant pull requests and main-branch changes, with
monorepo path filters for `aiNavLog/`. Separate checks that require no secrets
from authenticated release jobs. Build archives must remain rooted in the UI;
never upload sibling investment data, model credentials or private local files.
Add mobile Rust-target checks as native integration becomes available; the
host-tested C bridge alone is not proof of mobile compilation.

**CD behaviour:** begin with explicit/manual or release-tag triggering and a
protected approval gate for internal distribution. Build and submit the same
verified artifact, record source commit, package ID, version/build, checksum,
EAS build/submission IDs and destination track/group. Retain internal releases
on Google Play and TestFlight; public production publication is a separate
owner decision. Do not silently select the latest unrelated build for submission.
A failed check, build or upload must not be reported as tester availability.

**Credentials and prerequisites:** scope Expo authentication and any Google Play
service account/App Store Connect API key to the required jobs and stores. Keep
secrets in protected CI/EAS configuration, not Git or logs. Reuse existing Android
upload signing and the iOS team/bundle ID `com.kodekomplete.com.ainavlog`;
iOS App Store Connect ID is `6823786033`. Configure service/API access during
next week's implementation; no new credentials or external account changes are
created by this planning entry. Android's current manual upload remains the
fallback until automated Play submission is verified.

**Acceptance:** show a passing CI run and a deliberately failing check blocking
release; verify signed artifacts for both platforms and internal-track/TestFlight
delivery where credentials allow. Confirm an installed-app update as the native
increment becomes ready. Test preservation of phone-local records only after
that storage exists. If credentials prevent automated upload, record build-only
completion and an open CD gap rather than calling the pipeline finished.

**Capacity:** this is additional work next week, not spare capacity assumed free.
Review its impact on the Oct 23 native-packaging gate and Oct 30 Electrical demo;
if either is threatened, explicitly replan priorities/dates at the Oct 23 review.

**Repeatable release workflow:** validate a concrete change → produce a versioned,
signed artifact → install and smoke-test → distribute to the selected testers →
record build/source version, results and known limitations. Keep build configuration
in the repository and signing credentials in protected tooling. Record build and
installation times so we can measure and reduce release turnaround. Add automation
for repeatable checks/build steps as they are validated. Reuse this workflow during
each feature slice rather than reserving distribution for its end.

**Done for initial setup:** reproducible build commands/configuration are documented;
traceable signed artifacts install on the target devices; testers can receive builds;
and a second build updates successfully with local data retained. Target-platform
gaps remain explicit. Completion depends on observed installs/updates, not only a
successful cloud build. Store metadata, privacy declarations and review access are
prepared when required by the selected distribution channel.

**Ongoing:** distribute usable feature increments, test fresh installs and updates,
maintain release notes and known limitations, and verify permissions/lifecycle after
native changes. Qualify the complete beta after slice 6. Upload, review approval and
public publication are distinct states. Actual external builds/uploads/publication
require authorization when the concrete result is ready; this plan does not perform them.

**Store-controlled timing:** Apple's review-status documentation targets 90% of
submissions within 48 hours; Google says reviews can take up to seven days or longer
in exceptional cases. Neither is an approval guarantee.
[Apple review status](https://developer.apple.com/help/app-review/after-submitting-for-review/review-status),
[Google review/publishing](https://support.google.com/googleplay/android-developer/answer/9859654).

**Google account applicability:** this project uses a confirmed organization
account. Personal-account production-access testing requirements are not the
current internal-testing blocker. Recheck applicable Play requirements before
public release rather than imposing an unrelated waiting period on this plan.

**Contingency:** account activation is resolved; certificate renewal, native build
changes, processing/review and tester access remain operational risks. Keep those
separate from feature completion. Dec 14–Jan 3 is review/holiday contingency;
final integration/beta-readiness review targets Jan 18–31. Capacity and external
review timelines remain uncertain.

## 3. Engine

**Shared operational history:** Electrical and Engine should use the common event,
response and recurrence requirements in the [maritime alignment research](research/maritime-risk-alignment.md#product-requirements).
Electrical introduces the model around agreed voltage conditions; Engine reuses it
after its measurements and thresholds are selected. Track monitoring gaps separately,
distinguish acknowledgement from resolution and verify persistence and repeat events.

**Week 1:** agree the first engine quantities, units, alert meanings and compatible
source/protocol. Build a simulator for the selected measurements and reuse the
electrical slice's reception, storage and UI contracts.

**Week 2:** complete the engine view with latest values, timestamps, missing/stale
states, reconnection and configured alerts. Verify decoding and persisted history
with representative good, missing and fault samples.

**Done:** selected engine measurements update through a complete simulated path and
survive restart. Supported measurements and physical hardware validation gaps are
documented. Real engine compatibility is not inferred from simulator success.

**Dependency:** engine hardware/gateway and measurement set are not selected yet.
Resolve them before committing to a physical demo; beta instrument connectivity
remains Bluetooth-only and Wi-Fi remains shelved.
This is an open high-priority risk (RISK-03 below), not a confirmed integration
path. At the Oct 23 discovery gate, identify the engine model and available interfaces;
make the scope decision by Oct 30 before Engine starts Nov 2. If unresolved,
explicitly scope the Nov 13 milestone as simulated-only with physical integration
deferred. Any necessary hardware procurement has its own lead time.

## 4. Route

**Week 1:** establish the route/track data model, location source and permissions;
implement start/stop recording and local position persistence. Select map rendering
and attribution requirements. Preserve timestamps and distinguish simulated tracks.

**Week 2:** display live and saved tracks, reopen routes, and handle denied location
permission, signal loss, cancellation and app restart without inventing geometry.
Provide an explicit local route/media association for the later story slice.

**Done:** a short route can be recorded, saved, reopened and displayed. Available
offline recording/display behavior is documented; map availability and background
recording are validated or clearly identified as unsupported. No cloud log upload.

## 5. Authentication / authorization

**Week 1:** finalize the ownership/access policy designed in slice 1 and choose the
identity/session architecture. Implement the first verified provider flow and
protected credential storage; add remaining Google, Facebook and Apple adapters
within the agreed scope.

**Week 2:** enforce authorization at data access boundaries, test provider callbacks,
sign-out, expired sessions, account switching and policy-defined offline access.
Verify access to pre-sign-in local records without data loss or cross-account leaks.

**Done:** supported sign-in flows and session state work, permissions are enforced
beyond hiding UI controls, and unauthorized local-record access is rejected.
Provider coverage and outstanding console/review requirements are explicit.

**Dependencies:** provider setup, signing credentials, any trusted authentication
service, and account/boat access rules must be resolved early. This plan does not
introduce cloud boat-data synchronization or assume multi-user sharing is selected.
See [authentication architecture](../architecture.md#47-authentication-with-google-facebook-and-apple).

## 6. Route story

**Week 1:** select a saved route and local notes/media, compose a story and preview
exactly what will be included. Define the export format and platform handoff.

**Week 2:** implement explicit export/share, cancellation and failure handling,
temporary-file cleanup and missing-route/media behavior. Verify the entire sequence
from recorded route through authorized selection to exported story.

**Done:** a user can preview and export a story built from actual saved content.
No invented track, automatic publishing or claim of successful social publication
based only on a share-sheet handoff. Direct Facebook/Instagram support is validated
per platform; local image export is the fallback when direct handoff is unavailable.

## Reviews and dependencies

### External dependency and risk register

Entries below distinguish resolved initial setup from remaining delivery risks. Priority expresses
potential delivery impact, not a measured probability. Proposed owners are roles
to assign; access, budgets and supplier commitments are not assumed confirmed.

| ID / priority | External dependency | Risk and affected outcome | Mitigation / fallback | Proposed owner / resolution checkpoint |
| --- | --- | --- | --- | --- |
| RISK-01 / High | Apple Developer Program, App Store Connect, signing and TestFlight/App Review. **Initial setup resolved Oct 16:** membership active, signed TestFlight build and iPhone 13 install/demo checks confirmed; updates and later review remain open. | Enrollment or account access, signing failures, review rejection or resubmission can delay iPhone installs and distribution. Reviewers may be unable to exercise hardware-dependent screens. | Establish account ownership and signing during Electrical; prove a signed install early. Prepare accurate privacy/permission disclosures and reviewer demo access with clearly labeled simulated data. Reserve review/rework contingency; distinguish internal testing, external beta approval and public publication. | Product owner: accounts/access; developer: signing/submission. Initial install achieved Oct 16; native-build/update review Oct 30; final readiness review Jan 29. |
| RISK-02 / High | Google Play Console, account verification, app signing, testers and review. **Confirmed Oct 13:** organization developer account created, paid and verified; internal release available Oct 14; emulator install/Home launch confirmed Oct 16; physical Android, native reception and production review remain pending. | Later native builds, signing compatibility, tester access or store review can delay Android delivery. This is an organization account; do not apply personal-account waiting periods as a current project blocker. | Preserve the accepted upload key and package. Test installed-app updates, complete Android runtime/device evidence and recruit representative beta testers. Review production requirements when public submission is planned. | Product owner: account/tester coordination; developer: artifacts/submission. Initial account/distribution achieved; native-update checkpoint Oct 30; Android qualification review Nov 13; production readiness separately. |
| RISK-03 / High | Engine manufacturer, installed engine/ECU, available sensors and compatible gateway | **The engine device interface is currently unknown.** Signals may be analog, proprietary or available only through an unconfirmed gateway. Missing public protocol documentation, unsupported measurements, no beta-compatible Bluetooth path, cost or procurement delays can prevent a real engine vertical slice within two weeks. | Identify engine make/model/year, existing instruments, connectors, protocols and required measurements. Verify vendor documentation and a feasible read-only Bluetooth gateway before claiming support. Build a simulator behind the same interface. If unresolved, deliver a clearly simulated engine UI and explicitly defer/replan physical integration; do not silently substitute Wi-Fi or infer compatibility. | Product owner: boat/engine details and access; developer: protocol/gateway feasibility. Discovery Oct 23; source/simulated-only scope decision Oct 30 before Engine starts Nov 2. |
| RISK-04 / High | Victron BMV-712 hardware, firmware, owner setup and boat access | Simulator success may not match actual broadcasts, auxiliary starter voltage or installed shunt measurements. Key provisioning, configuration or unavailable boat access can delay qualification. | Validate the actual model/firmware and advertisement key; configure auxiliary voltage and battery parameters; compare house and starter readings with reference measurements. Keep simulator and live data visibly distinct. Physical qualification remains a release prerequisite, with dates assigned once access is known. | Product owner: hardware/access; developer: interoperability. Access plan Oct 23; physical session target assigned at Oct 30 review; evidence required before beta acceptance. |
| RISK-05 / High | iOS/Android native Bluetooth/location APIs, device permissions and mobile build tooling | Rust bridge/packaging or OS permission/lifecycle restrictions may prevent reliable reception, recording or background behavior despite passing desktop tests. Build service availability or required native tooling may delay installs. | Choose the build/install workflow in Electrical; demonstrate BLE-to-UI on the physical iPhone early and validate Android separately. Test denied permissions, stale data, reconnect and restart. Explicitly limit unsupported background behavior rather than assume parity. | Developer. Native packaging gate Oct 23; end-to-end Electrical Oct 30; location/lifecycle Nov 27. |
| RISK-06 / High | Apple, Google and Facebook identity providers; developer consoles and any selected authentication service | Provider setup, callback/signing configuration, external approval requirements or an unresolved session/access design can prevent complete authentication in its window. | Establish provider accounts/configuration ahead of slice 5; settle local-record ownership/access before implementation. Verify each intended provider independently and record incomplete coverage. Keep boat data local; do not imply account switching or cloud synchronization is solved by sign-in. | Product owner: provider access; developer: identity/access implementation. Setup checkpoint Nov 27; verified coverage/access review Dec 11. |
| RISK-07 / Medium | Map provider/SDK, location source, map data availability and attribution terms | Rendering, licensing/cost, offline map availability or poor position data can delay the route demo or make saved routes unusable offline. | Select the map/source approach before Route begins; confirm attribution and offline behavior. Persist recorded positions locally and expose gaps. Use a documented local track view if map access is unavailable; never invent geometry. | Developer, with product owner approving costs. Selection checkpoint Nov 13; route acceptance Nov 27. |
| RISK-08 / Medium | Facebook/Instagram platform handoff and mobile share facilities | Target apps, supported formats or platform restrictions may prevent direct story handoff. A completed handoff does not establish that a story was published. | Verify platform handoff on target devices; retain previewed local image export as fallback. Report cancellation/failure accurately and require explicit user sharing. | Developer. Handoff feasibility Dec 11; reconfirm platform behaviour Jan 4; story acceptance Jan 15. |
| RISK-09 / Medium | External beta testers, physical devices, vendor support and holiday availability | Lack of representative devices/testers or reduced December availability can leave defects undetected and consume review/rework time. | Recruit testers early, schedule boat/device sessions and confirm December availability. Keep platform/hardware evidence explicit; revise dates or scope if prerequisites are missing. | Product owner: coordination; developer: test evidence. Device/boat availability Oct 23; holiday capacity review Dec 11; beta-readiness review Jan 29. |

Apple/Google processing and review references are in slice 2 above. Other entries describe project uncertainties to
validate, not claims that a particular vendor restriction or compatibility problem
already exists. Recheck external requirements before submission.

At each checkpoint, assign an owner, update status and evidence, and record any
scope/date change. An unresolved high-priority dependency must not be reported as
completed merely because its simulated path works. Escalate engine-interface
uncertainty at the Electrical review so it cannot remain hidden until the Engine demo.

At each demo, record what works, evidence from the target platform, unresolved
limitations and next-slice readiness. Preserve simulator fixtures as regression
inputs and run checks appropriate to each change.

The first iPhone build/install dependency is resolved. The largest remaining
early dependency is Rust/native Bluetooth and phone-local storage integration. Engine source selection, maps/location behavior and identity
provider setup are the next uncertainties. Resolve these ahead of their slices;
if they exceed a window, explicitly adjust scope or dates at review.

Post-slice beta work includes physical BMV/engine validation, iOS/Android lifecycle
and background behavior, local data protection/retention, accessibility, installation
and distribution verification, and a release acceptance pass. The Jan 18–31 review window is provisional;
size the remaining work at each slice gate and confirm public-release timing separately.
