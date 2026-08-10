# Renderer WebContent public ownership proof research

Date: 2026-08-10

Target: macOS 26.2 SDK from Xcode 26.2 on macOS 27.0, system WebKit 22625.1.22.11.4

Decision scope: Renderer Isolation Slice 3, Path A only

## Question

Can an ordinary non-root BBR process use supported public macOS/WebKit APIs to
identify a launchd-owned `com.apple.WebKit.WebContent` process as belonging to
that exact BBR application instance, revalidate that identity immediately
before mutation, and terminate only that process without private WebKit SPI or
foreign-process risk?

## Verdict

**No supported public proof was found on this target.**

WebKit internally owns the exact XPC connection, helper PID, and peer audit
token. The public `WKWebView` API exposes none of them. Public AppKit and
libproc metadata identify a WebContent executable but do not expose its exact
UI-process relationship. Kernel coalition and originator information exists,
but the necessary query constants and wrappers are private SDK surfaces; more
importantly, two simultaneous instances of the same WKWebView client were
observed in the same resource and Jetsam coalitions, so coalition equality is
not sufficient to prove an exact BBR-instance relationship. EndpointSecurity
exposes a responsible audit token, but requires a restricted entitlement,
root, and Full Disk Access and therefore is not an ordinary BBR product API.

The accepted app-child proof remains fail-closed and must not be replaced by a
name, time, path, UID, coalition, resource, or candidate-count heuristic.

## Primary-source findings

### Public WebKit has no process identity

The complete public WebKit headers in the Xcode 26.2 macOS SDK contain no
WebContent PID, process identifier, audit token, or XPC connection property on
`WKWebView`. The public termination callback is only
`webViewWebContentProcessDidTerminate:`. `WKProcessPool` is deprecated and the
header states that creating multiple instances no longer has any effect.

SDK surfaces inspected:

- `WebKit.framework/Headers/WKWebView.h`
- `WebKit.framework/Headers/WKNavigationDelegate.h`
- `WebKit.framework/Headers/WKProcessPool.h`
- `WebKit.framework/Headers/WKWebViewConfiguration.h`

### WebKit internally knows the exact helper

Open-source WebKit commit
`0b99ee067b48d93c43d8bb7b375b8c95d64dcc93` shows the Cocoa process launcher
creating an XPC connection, obtaining the launched PID with
`xpc_connection_get_pid`, and storing the connection in the internal IPC
identifier. The helper can obtain the UI-process audit token from that same
connection. These are WebKit internals and are not exported through public
`WKWebView` API.

Relevant source:

- `Source/WebKit/UIProcess/Launcher/cocoa/ProcessLauncherCocoa.mm`
- `Source/WebKit/Platform/IPC/cocoa/ConnectionCocoa.mm`
- `Source/WebKit/Shared/Cocoa/AuxiliaryProcessCocoa.mm`

### Public `NSRunningApplication` is insufficient

`NSRunningApplication` publicly exposes name, bundle/executable URLs, PID and
optional launch date. Its header explicitly says not to rely on PID for process
comparison; the PID may change. It exposes no parent, responsible process,
audit token, PID version, or coalition.

A local WKWebView probe observed its WebContent process with:

- `ppid=1`;
- `bundleIdentifier=com.apple.WebKit.WebContent`;
- a UI-process-derived localized display name;
- no launch date;
- no public link to the creating UI process.

The public executable URL was also inconsistent: for two helpers,
`NSRunningApplication` reported a nonexistent repository-relative path while
`ps`/libproc showed the system XPC path and LaunchServices reported either the
system path or a bare executable name. It is presentation/discovery metadata,
not an authenticated ownership capability.

Two simultaneous independent clients with the same executable/name produced
two publicly indistinguishable `... Web Content` entries. Name, bundle,
executable path, UID and launch-time proximity therefore cannot prove exact
instance ownership.

### Coalition identity is private and not instance-unique

XNU commit `f6217f891ac0bb64f3d375211650a4c1ff8ca1ea` implements
`PROC_PIDCOALITIONINFO` and `proc_listcoalitions`, but declares them only in:

- `bsd/sys/proc_info_private.h`
- `libsyscall/wrappers/libproc/libproc_internal.h`

They are absent from the public macOS SDK's `sys/proc_info.h` and `libproc.h`.
LaunchServices' `lsappinfo` diagnostic can display
`LSApplicationMemberCoalitionIDKey`, but the key is absent from public
LaunchServices headers.

A read-only temporary research call to private proc flavor 20 confirmed an
additional correctness failure: two simultaneous instances of the same
WKWebView client and both WebContent helpers shared the same resource coalition
`1990` and Jetsam coalition `1991`. Coalition equality cannot distinguish the
correct BBR instance even if a private query were tolerated.

### Responsible process is not ordinarily queryable

XNU contains `p_responsible_pid`, but no public libproc getter exposes it for an
arbitrary process. `proc_pidoriginatorinfo` is private, is restricted to the
calling process, and represents voucher/originator state rather than a general
arbitrary-target responsible-process query.

EndpointSecurity's public data model includes `responsible_audit_token` and
states that `(pid, pidversion)` identifies a specific process execution.
However, `es_new_client` requires:

- `com.apple.developer.endpoint-security.client`;
- user Full Disk Access approval;
- root execution.

That is a separately provisioned security-product architecture, not an
ordinary browser/testkit seam, and does not satisfy the existing BBR boundary.

### BBR-owned resource markers do not bind the WebContent process

A local nonce-named file was loaded through `WKWebView.loadFileURL` and an
active streamed `fetch`. During that runtime observation, global and
process-specific `lsof` queries for the WebContent, Network and GPU processes
returned zero nonce/payload-path matches. WebKit mediates resource access; a
BBR-owned file or local URL is not a public process capability carried by the
target WebContent helper.

## Rejected proofs

| Candidate | Rejection |
|---|---|
| `ppid == app_pid` | Safe and accepted, but current WebContent helpers are reparented/launchd-owned with `ppid=1`. |
| Lowest/newest matching PID | Race-prone; can select foreign WebKit clients and fails under concurrent launches/PID reuse. |
| Localized helper name | Presentation metadata derived from UI-process name; two clients can intentionally or naturally collide. |
| Bundle ID/executable/UID | Shared by all clients using system WebKit. |
| Launch/check-in time | Not exposed by `NSRunningApplication` for these helpers and remains a heuristic under concurrent clients. |
| Process group/session | Helpers have their own groups and the same login session; no UI-process binding. |
| Coalition equality | Getter is private and observed coalition membership is shared across two same-client instances. |
| Open BBR nonce file/local resource | No public FD/capability appeared on the WebContent helper. |
| `WKProcessPool` | Deprecated/no-effect and exposes no helper identity. |
| WebKit internal XPC PID/audit token | Correct identity exists, but access requires private/internal WebKit state. |
| EndpointSecurity responsible token | Requires restricted entitlement, root and Full Disk Access; separate privileged architecture. |

## Fail-closed contract consequences

No process may be terminated unless all of the following are available from a
supported boundary and revalidated immediately before mutation:

1. immutable target execution identity, including PID-reuse protection;
2. an OS-authenticated relationship from that execution to the exact current
   BBR application execution;
3. deterministic selection of at most one target;
4. zero mutation on missing, ambiguous, stale, drifted or foreign evidence;
5. no per-WebView PID promise and no assumption that tabs have distinct
   processes.

Public BSD start-time information can contribute to item 1 by detecting PID
reuse, but it does not establish ownership. No ordinary public API discovered
here supplies item 2. Therefore Path A does not authorize a Renderer writer or
contract implementation.

## Minimal safe unblockers

One of these external/product-level inputs is required:

1. Apple adds or documents a public WKWebView/UI-process-to-WebContent audit
   token or process-execution relationship that can be immediately revalidated;
2. an explicitly approved safe environment exposes helpers as direct BBR
   children, allowing the existing reviewed parent-PID contract unchanged;
3. Patrick explicitly authorizes a new privileged helper architecture using
   an Apple-approved EndpointSecurity entitlement, root service and Full Disk
   Access, followed by a separate security/product packet and review. This is
   not recommended for the first MVP and is not implicitly authorized by this
   research.

Until then, retain the completed non-process renderer stability behavior and
leave the real helper-termination/reconciliation acceptance item open.

## Reproducibility evidence

Temporary probes only; no repository product state or system helper was
mutated or terminated.

- Public probe output SHA-256:
  `ade003ec083bec43ddc1e8d5ba052fcc6458e47570c5ef45bbe427bc1c7f1973`
- LaunchServices before/after SHA-256:
  `b3ab3f724a336fae805769aab07b03a8c6c5e52f4da52227a81a88df8bb11c78` /
  `ce540c8b5aa46a55307d27851f772de32a828969c0e79f39dfbe17b03a3c1987`
- Same-name dual-client LaunchServices SHA-256:
  `a30a5fcdb305bd8f9b4b541b43ab6988549f6cbadbcfd737c96fc0517b477485`
- Resource-marker LaunchServices SHA-256:
  `390c7f91d69f34e2c4f46c15fe6ba073586ad3e99dcdd1ffef8245f60b040e4f`
- Fresh raw resource-marker FD/lsof evidence SHA-256:
  `5459dde71c43a1bbf1e151eb481eee5862f4183685ce51aba421d06fe58d5b02`
- Fresh raw same-client private-coalition counterexample SHA-256:
  `1263d999b6a624d207c92e426cd798f7eb63422552fbe94d29942d6ccaca82b9`
- Public single-client Coalition/LaunchServices probe SHA-256:
  `f11b2cdcaa1ce452f184436f0e39c819d7a85b4f03236290208c653bd112c1b3`

The private coalition flavor was used only in a disposable read-only research
binary to falsify coalition uniqueness. It is explicitly not proposed for BBR
product or testkit code.
