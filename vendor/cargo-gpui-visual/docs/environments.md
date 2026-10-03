# Rendering environment and provenance

Unmarked environment JSON remains legacy evidence and compares in full. There is
no implicit field ignore list, OS-specific normalization or legacy migration.

Consumers may explicitly opt into `comparison_schema: 1` via
`environment::partition(observation, contract, pointers)`. The envelope contains:

- `layer`: gpui, ios or android, also retained in the rendering observation;
- `contract`: nonempty source-authored object identifying rendering semantics;
- `rendering`: all observation fields except explicitly named scalar leaves;
- `provenance`: canonical JSON pointer → exact removed scalar value.

Only existing object leaves can be partitioned. Unknown envelope fields, schemas,
invalid pointers, missing parents, overlapping rendering/provenance, null/array/
object provenance values and layer changes fail. Reconstruction must produce the
complete original observation. Consumers own stricter device/schema validation
and must not classify rendering conditions as provenance. The portable tool has
no knowledge that a particular GPU ID or build revision is safe to partition.

Comparison is exact over layer, contract, rendering and the provenance pointer
set. Only the values at equally classified provenance leaves may differ without
an environment mismatch. Such differences remain in `provenance_difference` in
the sealed review and in the original HTML report. They do not change pixel or
coverage results. Both full observations remain available for investigation.

Capture-time before/after checks still compare the **whole** environment, including
provenance. Acceptance rechecks exact review/capture identity, rendering mismatch
and provenance difference against the store. Accepting image changes with only
provenance drift does not silently replace the approved store environment.
Rendering changes still require explicitly reviewed environment adoption.

## Explicit migration of existing approved metadata

Generate a target envelope with the consumer's source-authored contract, then:

```sh
cargo gpui-visual partition-environment --baselines <store> \
  --expected-hash <sha256-of-baseline.json> --environment <target.json> --dry-run --json
# Inspect classification, all old/new facts and predicted output hash, then repeat
# the exact command without --dry-run only with explicit authorization.
```

The target must reconstruct the **exact** original approved environment. The
command refuses an already partitioned store; it cannot change observed facts,
accept a new capture or use current sources to rewrite historical provenance.
Every reference is hashed/decoded and any pixel fingerprint verified. Pending
writes, locks, invalid inventories, stale metadata or concurrent state changes
refuse mutation. An ordinary baseline lock brackets the atomic metadata write.
All non-environment JSON values, stored schema and reference bytes are preserved.

The JSON result records complete before/after environments, input/output hashes,
case count and whether it wrote. Retain it with the consumer's migration evidence.
Prior review baseline-state seals become stale; regenerate reports, never edit or
reseal historical ones. Migration does not grant visual or coverage acceptance.
Older tools cannot honor the partition: they compare envelopes exactly and thus
fail closed on provenance drift. Keep consumer exports immutable and pinned.
