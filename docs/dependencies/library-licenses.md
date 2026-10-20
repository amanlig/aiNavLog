# Library licenses and redistribution requirements

**Reviewed:** 2026-10-01. **Status:** dependency inventory and requirements summary;
not a complete release notice bundle or legal opinion.

## Scope and evidence

This inventory covers the ten direct Rust dependencies in
[Cargo.toml](../../device-interface/Cargo.toml), all 139 external Rust packages resolved
by [Cargo.lock](../../device-interface/Cargo.lock), native SQLite bundled on all targets and libdbus bundled by the
Linux build, and libraries referenced by the HTML prototypes. The Rust appendix
includes build-time and alternate-platform dependencies; it does not imply every
package is linked into the Windows executable.

Versions and declared Rust licenses were read from installed package manifests
using `cargo metadata --locked --format-version 1`. The shipped license files for
btleplug, Tokio, dbus, and vendored libdbus were also inspected. Transitive rows
record manifest declarations, not a file-by-file license audit. Preserve the
license files from the exact versions shipped; upstream default branches can change.

## Mobile beta distribution policy

**Confirmed:** The beta deployment targets are **iOS and Android**. Dependency
selection must preserve the ability to distribute on both platforms. Windows and
Linux CLI builds support development/evaluation; they do not define beta scope.

**Implementation policy:** Prefer permissive licenses (MIT, BSD, Apache-2.0, ISC,
and compatible additional notices) that support commercial binary distribution
without requiring publication of aiNavLog's application source. Keep platform-only
dependencies target-scoped. For any dependency without a suitable permissive path,
evaluate an alternative or separate commercial license before integrating it into
the beta. This is a project selection policy, not a claim that every copyleft
license is inherently incompatible with mobile platforms.

Ship the required copyright, license, and attribution notices with **beta builds
as well as production builds**, preferably as bundled third-party notices available
from an app licenses screen. Permissive licenses still impose obligations.

### Verified current Rust dependency graphs

| Target reviewed | Reachable external packages | D-Bus / BlueZ included? | Finding |
| --- | --- | --- | --- |
| `aarch64-apple-ios` | 95 | No | Declared licenses offer permissive paths; retain btleplug's mixed notices and additional Unicode notice. |
| `aarch64-linux-android` | 102 | No | Declared licenses offer permissive paths; also account for bundled Java support code outside the Cargo graph. |

Counts follow reachable Cargo resolve nodes, including build/proc-macro dependencies,
with current default features and Cargo.lock. They are not counts of libraries
embedded in a final app. The appendix now identifies membership in these two graphs.
Android has `target_os = "android"`, so our `cfg(target_os = "linux")` vendored D-Bus
dependency does not apply even though Android uses a Linux kernel.

**Result:** No license obstacle was identified in these current Rust dependency
manifest declarations that requires replacing btleplug for the mobile beta. This is
a scoped license assessment, not App Store/Google Play approval or mobile build
validation. The [btleplug license](https://github.com/deviceplug/btleplug/blob/master/LICENSE.md)
and [MIT](https://spdx.org/licenses/MIT.html)/[Apache-2.0](https://spdx.org/licenses/Apache-2.0.html)
terms permit redistribution subject to their obligations.

### Mobile integration work still requiring coverage

- Android: btleplug 0.13.3 bundles `io.github.gedgygedgy` Java helper classes.
  [jni-utils upstream](https://github.com/deviceplug/jni-utils-rs) identifies a
  BSD-3-Clause license. Preserve its original notices and confirm the exact vendored
  revision when assembling the Android notice bundle; these files are absent from
  Cargo's package inventory. Its packaged Gradle file lists JUnit as test-only,
  not a runtime dependency.
- Review the actual Gradle, CocoaPods/Swift Package Manager, and JavaScript lockfiles
  when the React Native bridge and mobile app are added. They do not exist yet in
  this project and are not covered by this Cargo assessment.
- Repeat the graph and bundled-code review for the final features, architectures,
  native SDKs, assets, and distribution artifacts. ARM64 is the current review
  scope, not a decision to exclude other beta architectures.
- Assemble the exact notice bundle before distributing the beta. This inventory
  records requirements; it is not that bundle.

Reproduce the target-filtered metadata from the `aiNavLog/` directory:

```sh
cargo metadata --manifest-path device-interface/Cargo.toml --locked --filter-platform aarch64-apple-ios --format-version 1 > /tmp/ainavlog-ios-metadata.json
cargo metadata --manifest-path device-interface/Cargo.toml --locked --filter-platform aarch64-linux-android --format-version 1 > /tmp/ainavlog-android-metadata.json
```

Traverse `resolve.root` through `resolve.nodes[].deps[].pkg` to determine reachable
packages; the unfiltered `packages` array can contain packages for other platforms.

The NMEA additions use nmea-kit (MIT OR Apache-2.0) and CANboat (Apache-2.0).
CANboat is enabled with `default-features = false, features = ["decode"]`; its CLI,
serial/CAN device I/O and USB dependencies are excluded. Target graphs were
refreshed for these additions; this does not establish mobile build validation.

## Direct Rust dependencies

| Library | Locked version | Usage | License and action |
| --- | --- | --- | --- |
| [futures-util](https://docs.rs/crate/futures-util/0.3.34/source/) | 0.3.34 | Bluetooth event stream consumption | MIT OR Apache-2.0; retain selected notices. |
| [rusqlite](https://docs.rs/crate/rusqlite/0.40.2/source/) | 0.40.2 | Local SQLite observation storage | MIT; retain notices and review bundled SQLite below. |
| [aes](https://docs.rs/crate/aes/0.9.3/source/) | 0.9.3 | Victron AES-128 primitive | MIT OR Apache-2.0; choose MIT and retain notices. |
| [ctr](https://docs.rs/crate/ctr/0.10.1/source/) | 0.10.1 | Victron CTR decryption mode | MIT OR Apache-2.0; choose MIT and retain notices. |
| [zeroize](https://docs.rs/crate/zeroize/1.9.0/source/) | 1.9.0 | Clear stored keys and temporary plaintext on drop | MIT OR Apache-2.0; choose MIT and retain notices. |
| [nmea-kit](https://docs.rs/crate/nmea-kit/0.8.9/source/) | 0.8.9 | NMEA 0183 decoding and encoding | MIT OR Apache-2.0; choose MIT and retain its notices. |
| [canboat](https://docs.rs/crate/canboat/8.3.0/source/) | 8.3.0 | NMEA 2000 PGN decoding and encoding | Apache-2.0; retain license, copyright and any applicable NOTICE content. |
| [btleplug](https://docs.rs/crate/btleplug/0.13.3/source/LICENSE.md) | 0.13.3 | BLE discovery and connection | Primarily BSD-3-Clause, with Rumble-derived MIT/Apache-2.0 code and blurmac-derived BSD-3-Clause code. Include the complete shipped `LICENSE.md`, its copyright notices, conditions, and disclaimers. Do not imply contributor endorsement. Do not interpret the manifest's slash-separated declaration as permission to discard the BSD notices. |
| [tokio](https://docs.rs/crate/tokio/1.53.1/source/LICENSE) | 1.53.1 | Async runtime, timers, and Ctrl-C handling | MIT. Include the copyright and full MIT license notice with distributed copies. |
| [dbus](https://docs.rs/crate/dbus/0.9.12/source/) | 0.9.12 | Linux-only dependency enabling vendored D-Bus through btleplug's backend | Apache-2.0 OR MIT for the Rust crate. Choose an allowed license and satisfy its notice requirements. This does not cover the bundled native libdbus; see below. |

## Native code bundled on Linux

`dbus` enables `vendored`, causing `libdbus-sys` 0.2.7 to compile and statically link
**libdbus 1.14.4**. Its Rust bindings declare MIT/Apache-2.0, but the bundled native
source has separate terms: **AFL-2.1 OR GPL-2.0-or-later**, according to its
[COPYING file](https://docs.rs/crate/libdbus-sys/0.2.7/source/vendor/dbus/COPYING).
The version and static-link behavior are recorded in
[build_vendored.rs](https://docs.rs/crate/libdbus-sys/0.2.7/source/build_vendored.rs).

For a Linux release, explicitly record the selected native-library license.
Under the AFL path, review source availability, retained attribution and modification
notices, restrictions on endorsement, and the reasonable-effort requirement to
obtain recipients' express assent. Sections 3, 6, and 9 require particular attention;
merely attaching an MIT notice for the Rust wrapper is insufficient. Choosing the
GPL path instead requires a separate assessment of GPL obligations for the linked
binary. The bundled COPYING file also identifies some standalone tools as GPL-only;
check file-level terms if redistributing the source archive or those tools.
See the [AFL-2.1 text](https://spdx.org/licenses/AFL-2.1.html).

This native dependency is Linux-specific and is excluded from the reviewed iOS
and Android beta graphs and our Windows build. It is not a mobile beta license blocker.
No dependency configuration or license election is changed by this document.

## Browser prototype libraries

These references appear in the embedded HTML of
[rout](../../rout/index.html), [elec](../../elec/index.html), and
[mech](../../mech/index.html). Versions are taken from their script URLs;
license families below are checked against upstream project documentation.
Exact CDN package contents and bundled subdependencies still need review when
preparing a release notice bundle.

| Library | Referenced version | Purpose | License requirement |
| --- | --- | --- | --- |
| [Floating UI core](https://github.com/floating-ui/floating-ui/blob/master/LICENSE) | 1.7.3 | Positioning logic | MIT: retain copyright and the full license notice. |
| [Floating UI DOM](https://github.com/floating-ui/floating-ui/blob/master/LICENSE) | 1.7.4 | Browser positioning integration | MIT: retain copyright and the full license notice, including notices for bundled dependencies. |
| [Lucide](https://lucide.dev/license) | 1.17.0 | Icons | ISC; inherited Feather icons carry MIT terms. Retain both applicable sets of copyright and permission notices from the shipped package. |

A CDN URL does not replace attribution. Preserve notices in redistributed bundles
and include relevant notices with the web application. Hosts listed only in a
Content Security Policy are not evidence that a library or font is actually loaded.
No additional font download was identified in the inspected HTML.

## Bundled SQLite

`rusqlite` uses the `bundled` feature: `libsqlite3-sys` 0.38.2 builds SQLite
3.53.2 from its packaged C source. A C compiler is required on each build target.
The Rust wrapper declares MIT; the bundled `sqlite3.c` header dedicates SQLite
to the public domain. Preserve wrapper notices and the upstream native source
header in the release evidence. Mobile graphs include this native build, but
have not been validated as mobile binaries.

## Requirement profiles

The appendix maps each crate to a practical notice-handling profile. These are
summaries; the exact shipped license text controls.

| Profile | Redistribution action |
| --- | --- |
| MIT | Preserve the copyright, permission notice, and disclaimer with copies or substantial portions. See [MIT](https://spdx.org/licenses/MIT.html). |
| Choice: MIT | The declared alternatives include MIT; using that option is a proposed simplification, not an automatic license election. Retain the package's actual MIT notice and any separately applicable notices. |
| BSD / mixed | For btleplug, retain its entire mixed-license file. BSD-3-Clause requires notices/conditions/disclaimer in source and in binary distribution documentation or materials, and prohibits unauthorized endorsement. See [btleplug license](https://github.com/deviceplug/btleplug/blob/master/LICENSE.md). |
| MIT + Unicode | A permissive-code license choice does not remove the additional Unicode obligation. Include the Unicode copyright and permission notice with the software or documentation; do not imply endorsement. See [Unicode-3.0](https://spdx.org/licenses/Unicode-3.0.html). |
| SQLite native review | Retain the MIT wrapper notice and record the bundled SQLite public-domain dedication described above. |
| Native review | In addition to the Rust wrapper's license choice, apply the separate libdbus requirements above. |

When choosing Apache-2.0 instead of MIT, provide the Apache license, preserve
applicable attribution, include relevant upstream NOTICE contents when supplied,
and identify modified files. Patent and trademark provisions also apply. See
[Apache-2.0](https://spdx.org/licenses/Apache-2.0.html). `OR` permits a choice;
`AND` requires both obligations. Slash-separated legacy declarations are reproduced
verbatim below and should be checked against the packaged license files.

## Rust dependency inventory

The source links lead to the exact package version. Direct dependencies are marked;
all remaining entries are transitive. The profile column explains an available
notice-handling approach, not proof that distribution requirements are already met.

<!-- BEGIN RUST INVENTORY -->
| Package | Version | Declared license | Profile | Relation | Mobile graph |
| --- | --- | --- | --- | --- | --- |
| [aes](https://docs.rs/crate/aes/0.9.3/source/) | 0.9.3 | `MIT OR Apache-2.0` | Choice: MIT | Direct | iOS, Android |
| [anyhow](https://docs.rs/crate/anyhow/1.0.104/source/) | 1.0.104 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [async-trait](https://docs.rs/crate/async-trait/0.1.92/source/) | 0.1.92 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [bitflags](https://docs.rs/crate/bitflags/2.13.2/source/) | 2.13.2 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [block-buffer](https://docs.rs/crate/block-buffer/0.12.1/source/) | 0.12.1 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [block2](https://docs.rs/crate/block2/0.6.2/source/) | 0.6.2 | `MIT` | MIT | Transitive | iOS |
| [bluez-async](https://docs.rs/crate/bluez-async/0.8.2/source/) | 0.8.2 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [bluez-generated](https://docs.rs/crate/bluez-generated/0.4.0/source/) | 0.4.0 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [bon](https://docs.rs/crate/bon/3.10.1/source/) | 3.10.1 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [bon-macros](https://docs.rs/crate/bon-macros/3.10.1/source/) | 3.10.1 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [btleplug](https://docs.rs/crate/btleplug/0.13.3/source/) | 0.13.3 | `MIT/Apache-2.0/BSD-3-Clause` | BSD / mixed | Direct | iOS, Android |
| [bumpalo](https://docs.rs/crate/bumpalo/3.20.3/source/) | 3.20.3 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [bytes](https://docs.rs/crate/bytes/1.12.1/source/) | 1.12.1 | `MIT` | MIT | Transitive | iOS, Android |
| [canboat](https://docs.rs/crate/canboat/8.3.0/source/) | 8.3.0 | `Apache-2.0` | Apache-2.0 | Direct | iOS, Android |
| [cc](https://docs.rs/crate/cc/1.5.1/source/) | 1.5.1 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [cfg-if](https://docs.rs/crate/cfg-if/1.0.5/source/) | 1.0.5 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [cipher](https://docs.rs/crate/cipher/0.5.2/source/) | 0.5.2 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [combine](https://docs.rs/crate/combine/4.6.8/source/) | 4.6.8 | `MIT` | MIT | Transitive | Android |
| [cpubits](https://docs.rs/crate/cpubits/0.1.1/source/) | 0.1.1 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [cpufeatures](https://docs.rs/crate/cpufeatures/0.3.1/source/) | 0.3.1 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [crossbeam-utils](https://docs.rs/crate/crossbeam-utils/0.8.23/source/) | 0.8.23 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [crypto-common](https://docs.rs/crate/crypto-common/0.2.2/source/) | 0.2.2 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [ctr](https://docs.rs/crate/ctr/0.10.1/source/) | 0.10.1 | `MIT OR Apache-2.0` | Choice: MIT | Direct | iOS, Android |
| [darling](https://docs.rs/crate/darling/0.24.1/source/) | 0.24.1 | `MIT` | Choice: MIT | Transitive | iOS, Android |
| [darling_core](https://docs.rs/crate/darling_core/0.24.1/source/) | 0.24.1 | `MIT` | Choice: MIT | Transitive | iOS, Android |
| [darling_macro](https://docs.rs/crate/darling_macro/0.24.1/source/) | 0.24.1 | `MIT` | Choice: MIT | Transitive | iOS, Android |
| [dashmap](https://docs.rs/crate/dashmap/6.2.1/source/) | 6.2.1 | `MIT` | MIT | Transitive | iOS, Android |
| [dbus](https://docs.rs/crate/dbus/0.9.12/source/) | 0.9.12 | `Apache-2.0/MIT` | SQLite native review | Retain the MIT wrapper notice and record the bundled SQLite public-domain dedication described above. |
| Native review | Direct | Outside reviewed mobile graphs |
| [dbus-tokio](https://docs.rs/crate/dbus-tokio/0.7.6/source/) | 0.7.6 | `Apache-2.0/MIT` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [deranged](https://docs.rs/crate/deranged/0.5.8/source/) | 0.5.8 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [either](https://docs.rs/crate/either/1.18.0/source/) | 1.18.0 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [equivalent](https://docs.rs/crate/equivalent/1.0.2/source/) | 1.0.2 | `Apache-2.0 OR MIT` | Choice: MIT | Transitive | iOS, Android |
| [errno](https://docs.rs/crate/errno/0.3.14/source/) | 0.3.14 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [fallible-iterator](https://docs.rs/crate/fallible-iterator/0.3.0/source/) | 0.3.0 | `MIT/Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [fallible-streaming-iterator](https://docs.rs/crate/fallible-streaming-iterator/0.1.9/source/) | 0.1.9 | `MIT/Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [find-msvc-tools](https://docs.rs/crate/find-msvc-tools/0.1.14/source/) | 0.1.14 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [futures](https://docs.rs/crate/futures/0.3.34/source/) | 0.3.34 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [futures-channel](https://docs.rs/crate/futures-channel/0.3.34/source/) | 0.3.34 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [futures-core](https://docs.rs/crate/futures-core/0.3.34/source/) | 0.3.34 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [futures-executor](https://docs.rs/crate/futures-executor/0.3.34/source/) | 0.3.34 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [futures-io](https://docs.rs/crate/futures-io/0.3.34/source/) | 0.3.34 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [futures-macro](https://docs.rs/crate/futures-macro/0.3.34/source/) | 0.3.34 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [futures-sink](https://docs.rs/crate/futures-sink/0.3.34/source/) | 0.3.34 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [futures-task](https://docs.rs/crate/futures-task/0.3.34/source/) | 0.3.34 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [futures-util](https://docs.rs/crate/futures-util/0.3.34/source/) | 0.3.34 | `MIT OR Apache-2.0` | Choice: MIT | Direct | iOS, Android |
| [hashbrown](https://docs.rs/crate/hashbrown/0.14.5/source/) | 0.14.5 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [hashbrown](https://docs.rs/crate/hashbrown/0.17.1/source/) | 0.17.1 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [hybrid-array](https://docs.rs/crate/hybrid-array/0.4.15/source/) | 0.4.15 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [ident_case](https://docs.rs/crate/ident_case/1.0.1/source/) | 1.0.1 | `MIT/Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [indexmap](https://docs.rs/crate/indexmap/2.14.2/source/) | 2.14.2 | `Apache-2.0 OR MIT` | Choice: MIT | Transitive | iOS, Android |
| [inout](https://docs.rs/crate/inout/0.2.2/source/) | 0.2.2 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [itertools](https://docs.rs/crate/itertools/0.14.0/source/) | 0.14.0 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [itoa](https://docs.rs/crate/itoa/1.0.18/source/) | 1.0.18 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [jni](https://docs.rs/crate/jni/0.22.4/source/) | 0.22.4 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Android |
| [jni-macros](https://docs.rs/crate/jni-macros/0.22.4/source/) | 0.22.4 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Android |
| [jni-sys](https://docs.rs/crate/jni-sys/0.4.1/source/) | 0.4.1 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Android |
| [jni-sys-macros](https://docs.rs/crate/jni-sys-macros/0.4.1/source/) | 0.4.1 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Android |
| [js-sys](https://docs.rs/crate/js-sys/0.3.106/source/) | 0.3.106 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [libc](https://docs.rs/crate/libc/0.2.189/source/) | 0.2.189 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [libdbus-sys](https://docs.rs/crate/libdbus-sys/0.2.7/source/) | 0.2.7 | `Apache-2.0/MIT` | SQLite native review | Retain the MIT wrapper notice and record the bundled SQLite public-domain dedication described above. |
| Native review | Transitive | Outside reviewed mobile graphs |
| [libsqlite3-sys](https://docs.rs/crate/libsqlite3-sys/0.38.2/source/) | 0.38.2 | `MIT` | SQLite native review | Transitive | iOS, Android |
| [lock_api](https://docs.rs/crate/lock_api/0.4.14/source/) | 0.4.14 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [log](https://docs.rs/crate/log/0.4.34/source/) | 0.4.34 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [memchr](https://docs.rs/crate/memchr/2.8.3/source/) | 2.8.3 | `Unlicense OR MIT` | Choice: MIT | Transitive | iOS, Android |
| [mio](https://docs.rs/crate/mio/1.2.3/source/) | 1.2.3 | `MIT` | MIT | Transitive | iOS, Android |
| [nmea-kit](https://docs.rs/crate/nmea-kit/0.8.9/source/) | 0.8.9 | `MIT OR Apache-2.0` | Choice: MIT | Direct | iOS, Android |
| [num-conv](https://docs.rs/crate/num-conv/0.2.2/source/) | 0.2.2 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [num_threads](https://docs.rs/crate/num_threads/0.1.7/source/) | 0.1.7 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [objc2](https://docs.rs/crate/objc2/0.6.4/source/) | 0.6.4 | `MIT` | MIT | Transitive | iOS |
| [objc2-core-bluetooth](https://docs.rs/crate/objc2-core-bluetooth/0.3.2/source/) | 0.3.2 | `Zlib OR Apache-2.0 OR MIT` | Choice: MIT | Transitive | iOS |
| [objc2-encode](https://docs.rs/crate/objc2-encode/4.1.0/source/) | 4.1.0 | `MIT` | MIT | Transitive | iOS |
| [objc2-foundation](https://docs.rs/crate/objc2-foundation/0.3.2/source/) | 0.3.2 | `MIT` | MIT | Transitive | iOS |
| [once_cell](https://docs.rs/crate/once_cell/1.21.4/source/) | 1.21.4 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [parking_lot_core](https://docs.rs/crate/parking_lot_core/0.9.12/source/) | 0.9.12 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [pin-project-lite](https://docs.rs/crate/pin-project-lite/0.2.17/source/) | 0.2.17 | `Apache-2.0 OR MIT` | Choice: MIT | Transitive | iOS, Android |
| [pkg-config](https://docs.rs/crate/pkg-config/0.3.34/source/) | 0.3.34 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [powerfmt](https://docs.rs/crate/powerfmt/0.2.0/source/) | 0.2.0 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [prettyplease](https://docs.rs/crate/prettyplease/0.3.0/source/) | 0.3.0 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [proc-macro2](https://docs.rs/crate/proc-macro2/1.0.107/source/) | 1.0.107 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [quote](https://docs.rs/crate/quote/1.0.47/source/) | 1.0.47 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [redox_syscall](https://docs.rs/crate/redox_syscall/0.5.18/source/) | 0.5.18 | `MIT` | MIT | Transitive | Outside reviewed mobile graphs |
| [rusqlite](https://docs.rs/crate/rusqlite/0.40.2/source/) | 0.40.2 | `MIT` | MIT | Direct | iOS, Android |
| [rustc_version](https://docs.rs/crate/rustc_version/0.4.1/source/) | 0.4.1 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Android |
| [rustversion](https://docs.rs/crate/rustversion/1.0.23/source/) | 1.0.23 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [same-file](https://docs.rs/crate/same-file/1.0.6/source/) | 1.0.6 | `Unlicense/MIT` | Choice: MIT | Transitive | Android |
| [scopeguard](https://docs.rs/crate/scopeguard/1.2.0/source/) | 1.2.0 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [semver](https://docs.rs/crate/semver/1.0.28/source/) | 1.0.28 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Android |
| [serde](https://docs.rs/crate/serde/1.0.229/source/) | 1.0.229 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [serde-xml-rs](https://docs.rs/crate/serde-xml-rs/0.8.2/source/) | 0.8.2 | `MIT` | MIT | Transitive | Outside reviewed mobile graphs |
| [serde_core](https://docs.rs/crate/serde_core/1.0.229/source/) | 1.0.229 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [serde_derive](https://docs.rs/crate/serde_derive/1.0.229/source/) | 1.0.229 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [shlex](https://docs.rs/crate/shlex/2.0.1/source/) | 2.0.1 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [signal-hook-registry](https://docs.rs/crate/signal-hook-registry/1.4.8/source/) | 1.4.8 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [simd_cesu8](https://docs.rs/crate/simd_cesu8/1.2.0/source/) | 1.2.0 | `Apache-2.0 OR MIT` | Choice: MIT | Transitive | Android |
| [simdutf8](https://docs.rs/crate/simdutf8/0.1.5/source/) | 0.1.5 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Android |
| [slab](https://docs.rs/crate/slab/0.4.12/source/) | 0.4.12 | `MIT` | MIT | Transitive | iOS, Android |
| [smallvec](https://docs.rs/crate/smallvec/1.16.2/source/) | 1.16.2 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [socket2](https://docs.rs/crate/socket2/0.6.5/source/) | 0.6.5 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [static_assertions](https://docs.rs/crate/static_assertions/1.1.0/source/) | 1.1.0 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [strsim](https://docs.rs/crate/strsim/0.11.1/source/) | 0.11.1 | `MIT` | Choice: MIT | Transitive | iOS, Android |
| [syn](https://docs.rs/crate/syn/2.0.119/source/) | 2.0.119 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Android |
| [syn](https://docs.rs/crate/syn/3.0.6/source/) | 3.0.6 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [thiserror](https://docs.rs/crate/thiserror/2.0.21/source/) | 2.0.21 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [thiserror-impl](https://docs.rs/crate/thiserror-impl/2.0.21/source/) | 2.0.21 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [time](https://docs.rs/crate/time/0.3.55/source/) | 0.3.55 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [time-core](https://docs.rs/crate/time-core/0.1.9/source/) | 0.1.9 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [time-macros](https://docs.rs/crate/time-macros/0.2.32/source/) | 0.2.32 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [tokio](https://docs.rs/crate/tokio/1.53.1/source/) | 1.53.1 | `MIT` | MIT | Direct | iOS, Android |
| [tokio-macros](https://docs.rs/crate/tokio-macros/2.7.2/source/) | 2.7.2 | `MIT` | MIT | Transitive | iOS, Android |
| [tokio-stream](https://docs.rs/crate/tokio-stream/0.1.19/source/) | 0.1.19 | `MIT` | MIT | Transitive | iOS, Android |
| [tokio-util](https://docs.rs/crate/tokio-util/0.7.19/source/) | 0.7.19 | `MIT` | MIT | Transitive | iOS, Android |
| [typenum](https://docs.rs/crate/typenum/1.20.1/source/) | 1.20.1 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [unicode-ident](https://docs.rs/crate/unicode-ident/1.0.26/source/) | 1.0.26 | `(MIT OR Apache-2.0) AND Unicode-3.0` | MIT + Unicode | Transitive | iOS, Android |
| [uuid](https://docs.rs/crate/uuid/1.26.1/source/) | 1.26.1 | `Apache-2.0 OR MIT` | Choice: MIT | Transitive | iOS, Android |
| [vcpkg](https://docs.rs/crate/vcpkg/0.2.15/source/) | 0.2.15 | `MIT/Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [vergen](https://docs.rs/crate/vergen/10.0.3/source/) | 10.0.3 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [vergen-gitcl](https://docs.rs/crate/vergen-gitcl/10.0.3/source/) | 10.0.3 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [vergen-lib](https://docs.rs/crate/vergen-lib/10.0.3/source/) | 10.0.3 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | iOS, Android |
| [walkdir](https://docs.rs/crate/walkdir/2.5.0/source/) | 2.5.0 | `Unlicense/MIT` | Choice: MIT | Transitive | Android |
| [wasi](https://docs.rs/crate/wasi/0.11.1+wasi-snapshot-preview1/source/) | 0.11.1+wasi-snapshot-preview1 | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [wasm-bindgen](https://docs.rs/crate/wasm-bindgen/0.2.129/source/) | 0.2.129 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [wasm-bindgen-macro](https://docs.rs/crate/wasm-bindgen-macro/0.2.129/source/) | 0.2.129 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [wasm-bindgen-macro-support](https://docs.rs/crate/wasm-bindgen-macro-support/0.2.129/source/) | 0.2.129 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [wasm-bindgen-shared](https://docs.rs/crate/wasm-bindgen-shared/0.2.129/source/) | 0.2.129 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [winapi-util](https://docs.rs/crate/winapi-util/0.1.11/source/) | 0.1.11 | `Unlicense OR MIT` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [windows](https://docs.rs/crate/windows/0.62.2/source/) | 0.62.2 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [windows-collections](https://docs.rs/crate/windows-collections/0.3.2/source/) | 0.3.2 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [windows-core](https://docs.rs/crate/windows-core/0.62.2/source/) | 0.62.2 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [windows-future](https://docs.rs/crate/windows-future/0.3.2/source/) | 0.3.2 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [windows-implement](https://docs.rs/crate/windows-implement/0.60.2/source/) | 0.60.2 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [windows-interface](https://docs.rs/crate/windows-interface/0.59.3/source/) | 0.59.3 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [windows-link](https://docs.rs/crate/windows-link/0.2.1/source/) | 0.2.1 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [windows-numerics](https://docs.rs/crate/windows-numerics/0.3.1/source/) | 0.3.1 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [windows-result](https://docs.rs/crate/windows-result/0.4.1/source/) | 0.4.1 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [windows-strings](https://docs.rs/crate/windows-strings/0.5.1/source/) | 0.5.1 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [windows-sys](https://docs.rs/crate/windows-sys/0.61.2/source/) | 0.61.2 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [windows-threading](https://docs.rs/crate/windows-threading/0.2.1/source/) | 0.2.1 | `MIT OR Apache-2.0` | Choice: MIT | Transitive | Outside reviewed mobile graphs |
| [xml](https://docs.rs/crate/xml/1.4.0/source/) | 1.4.0 | `MIT` | MIT | Transitive | Outside reviewed mobile graphs |
| [zeroize](https://docs.rs/crate/zeroize/1.9.0/source/) | 1.9.0 | `Apache-2.0 OR MIT` | Choice: MIT | Direct | iOS, Android |
<!-- END RUST INVENTORY -->

## Release preparation and maintenance

1. Recompute the target-specific dependency graph for each release, including
   enabled features, native libraries, and any browser bundles. The appendix is an
   all-target lockfile inventory, not a binary bill of materials.
2. Collect exact upstream copyright/license files and relevant NOTICE text into a
   `THIRD_PARTY_NOTICES` bundle delivered with the executable or application. This
   document supplies an inventory; it does not itself supply those full notices.
3. Resolve the Linux native D-Bus license path before distributing a Linux binary.
4. Recheck this document whenever Cargo.lock, Cargo.toml, or CDN script URLs change.

To refresh the metadata used for the appendix, from the `aiNavLog/` directory:

```sh
cargo metadata --manifest-path device-interface/Cargo.toml --locked --format-version 1 > /tmp/ainavlog-metadata.json
```

Build tools (Rust/Cargo, cargo-xwin, LLVM) and Microsoft SDK/CRT redistribution
terms are a separate toolchain review; they are not Cargo application dependencies.
Rust standard-library components and any bundled runtime redistributables must
also be considered for the actual release artifact. React Native and Expo remain
planned technologies, not installed libraries in this repository. The other Rust
Bluetooth libraries in the research document have not been added as dependencies.
Images and application-owned source are outside this library inventory. The Rust
package currently declares no application license and uses `publish = false`;
that setting is not a license grant and does not replace third-party obligations.
