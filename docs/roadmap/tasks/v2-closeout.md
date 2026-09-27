# Task packet — v2 S11: gallery, documentation and closeout

Parents: [v2 program](v2-visual-refresh-program.md) ·
[Component Gallery compositions](component-gallery-compositions.md) ·
[DESIGN.md](../../../DESIGN.md) §18.6.

## Outcome

The Gallery presents the v2 tokens and motion moments from the production
components, ARCHITECTURE.md and E2E_ARCHITECTURE.md record the motion-mode
and transient-animation evidence facts, DESIGN §18.6 shows every slice
landed, and the canonical aggregate passes from the integrated commit.

## Scope

- Gallery foundation stories: v2 colour/elevation and motion tokens.
- Documentation facts; program ledger; current-delivery closeout; resume
  RA-MEDIA as next.
- Final canonical run.

## Non-goals

- Performance execution (deferred by Patrick; recorded as unpassed).

## Parallel-lane contract

Not parallelized.

## Constraints

- Gallery uses production components only; no second renderer.

## Evidence

### Unit and contract

- [ ] Gallery story registry tests.

### Real-app E2E

- [ ] `just e2e` aggregate without `e2e-perf`, plus `e2e-peek` and
      `e2e-gallery`, green from the integrated commit.

### Performance

- [ ] Unpassed: execution deferred.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-gallery
just e2e-peek
just e2e              # minus e2e-perf while deferred
just roadmap
```

## Iteration policy

Stories, then documents, then the canonical run; fix regressions at their
owning slice's mechanism.

## Blocked stop condition

If the canonical run fails for a reason outside v2, stop with the failing
suite, its evidence and the owning packet.

## Delivery ledger

```text
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:   unpassed — execution deferred
Worktree state:
```
