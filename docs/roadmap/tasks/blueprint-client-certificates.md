# Task packet — Blueprint D12: Client certificate chooser

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §18.4, §19.

Status: implemented 2026-09-30 (DESIGN §19); position 19 of 24. Paper boards: M-11 · Client certificates, PR-15 · Sign-in challenges (file
“BBR — Browser Shell”).

## Outcome

A sheet listing Keychain identities (name, issuer, expiry), remembered per site and profile; expired identities visible but not selectable.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §18.4 in the same group and flip the
  D12 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: Client-certificate challenges have a contract but no chooser.
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [x] Unit/contract: `bbr-web` `client_certificates` expiry, `bbr-web-macos` `client_identities` metadata from a real SecCertificate (subject, issuer, expiry, fingerprint) and SHA-256 vector, `certificate_choices` round trip and tolerance, `app::client_certificates` remembered answers and first valid row, short dates.

### Real-app E2E

- [x] `just e2e-client-certificates` (`e2e_client_certificates.rs`): sheet names server and profile, expired identity dimmed and unselectable, keyboard over valid rows, Remember + Return answers the engine and writes `client-certificates.txt`, remembered answer without a sheet, queue, Don't Send and Escape, navigation cancels, Settings › Privacy list and Forget; errors, auth, permissions and settings suites green.
- [x] Regression 2026-09-30: every debug build aborted on the first HTTPS page (and any Basic challenge) in the delegate method. WebKit's `WKNSURLAuthenticationChallenge` is an `NSProxy` (`WKObject`) that forwards to the real challenge, and objc2's debug message check found no `protectionSpace` on the proxy class. The injected requests above never reached the native method, and no fixture was served with a challenge. The delegate now messages the forwarding target (`auth_challenge::real_challenge`); `a_native_authentication_challenge_keeps_the_default_handling` loads the fixture server's Basic challenge (`/__fx/auth/basic`) through the real delegate. It failed with the reported panic before the fix. A one-off check against `https://example.com` (server trust, network, not committed) loaded in a debug build.

### Performance

- [ ] Extend `just e2e-perf` with the sheet presentation scenario (deferred).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-client-certificates
just e2e-auth
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D12 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:                WKNavigationDelegate client-certificate challenge with Keychain identity metadata (no key material), the blocking chooser sheet, remembered choices per profile and server, Settings › Privacy list.
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    Unit/contract: `bbr-web` `client_certificates` expiry, `bbr-web-macos` `client_identities` metadata from a real SecCertificate (subject, issuer, expiry, fingerprint) and SHA-256 vector, `certificate_choices` round trip and tolerance, `app::client_certificates` remembered answers and first valid row, short dates.
Real-app E2E:           `just e2e-client-certificates` (`e2e_client_certificates.rs`): sheet names server and profile, expired identity dimmed and unselectable, keyboard over valid rows, Remember + Return answers the engine and writes `client-certificates.txt`, remembered answer without a sheet, queue, Don't Send and Escape, navigation cancels, Settings › Privacy list and Forget; errors, auth, permissions and settings suites green.
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §18.4 Client certificates, §19 D12 row.
Open blockers and non-goals: the live TLS client-certificate path (WebKit challenge → Keychain → NSURLCredential) needs a hand check against a server that requires client certificates — an automated one would add identities to the user's Keychain; performance scenario deferred
Worktree state:         main, clean after commit
```
