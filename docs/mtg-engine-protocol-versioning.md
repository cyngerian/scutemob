# Protocol Versioning Policy — `Command`, `GameEvent`, and Replay Logs

<!-- last_updated: 2026-07-10 -->

> **Status**: implemented, SR-8 (`scutemob-60`). Binding on all M10 networking work.
>
> **Code**: `crates/engine/src/rules/protocol.rs`
> **Gates**: `crates/engine/tests/protocol_schema.rs` (11 tests), `crates/engine/tests/protocol_roundtrip.rs` (17 tests)

---

## The problem

`Command` is the only way into the engine (invariant #3) and `GameEvent` is the only
way out (invariant #4). M10 puts a WebSocket between them. So those two enums *are*
the wire protocol, and they are what a replay log is made of.

Before this task, both derived `Serialize`/`Deserialize` and carried nothing else. The
whole compatibility story was `#[serde(default)]` on newer fields — which makes *adding
an optional field* survive a round trip, and does nothing at all about a renamed field, a
deleted variant, or a variant whose meaning changed. Two builds with different shapes
would both happily deserialize each other's messages, dropping or defaulting whatever
they did not understand, **and neither would notice**.

`HASH_SCHEMA_VERSION` (`state/hash.rs`) is adjacent but answers a different question: it
versions the *state hash*, not the command/event stream, and nothing forces it to move
when the state shape does.

---

## Policy

### 1. Strict lockstep. Exact match. Reject on mismatch.

Every serialized message is wrapped in an `Envelope` carrying `protocol_version`. A
receiver accepts a message **iff** the declared version equals its own `PROTOCOL_VERSION`.
Anything else — older *or newer* — is rejected with `ProtocolError::VersionMismatch`.

There is no negotiation, no capability handshake, no "ignore unknown fields", and no
best-effort decode of an adjacent version.

**Why, and why not the softer option.** The obvious alternative is additive/forward
compatibility: tolerate unknown variants, default missing fields, let an old client limp
along. That is the right default for most network protocols, and it is wrong here,
because of invariant #9:

> The rewind/replay/pause system depends on a complete and accurate state history from
> turn 1. A card whose abilities silently never fired produces a corrupted history that
> cannot be rewound to correctly.

A client that skips a `GameEvent` variant it does not recognise holds exactly that
corrupted history — and holds it *without knowing*, because tolerating the unknown is
indistinguishable from there being nothing to tolerate. The failure surfaces later, as a
desync or a bad rewind, arbitrarily far from its cause.

A refused connection is a recoverable, legible failure: "this build speaks v3, the server
speaks v4." A corrupted history is neither. In a trusted-playgroup, single-server
deployment (the M10 architecture decision of 2026-02-23), all clients ship from the same
build anyway, so lockstep costs approximately nothing and buys a loud failure.

Corollary, tested in `a_future_version_is_rejected_even_when_its_payload_would_parse`: a
*future* message whose payload happens to be readable today is still refused. "It parsed"
is not evidence that it means what we think.

### 2. The version check runs before the payload is parsed.

`decode()` is staged:

1. deserialize a `VersionProbe` that reads only `protocol_version`;
2. reject a mismatch;
3. only then deserialize the payload.

The payload's *shape* is not knowable until the version is known to match, so parsing it
first is meaningless. Concretely, the one-pass alternative
(`serde_json::from_str::<Envelope<T>>`) also rejects old messages — but reports them as
`unknown variant 'Foo' at line 1 column 42`. A client can act on `VersionMismatch`
("tell the user to upgrade"); it can only guess at the serde error.

This is enforced by `version_is_checked_before_the_payload_is_parsed`, which feeds an
old-version envelope containing a payload this build cannot parse and requires
`VersionMismatch`.

Conversely, a bad payload at *our own* version is `ProtocolError::Payload` — a genuine
bug, not a compatibility problem. The two must not be conflated.

### 3. The version number is machine-checked, not remembered.

This is the part that matters.

A hand-bumped constant sitting next to an enum that grows every milestone is correct
exactly as long as every future author remembers to bump it. That is a *process*
guarantee, and converting process guarantees into machine guarantees is the entire point
of the SR remediation track.

So `PROTOCOL_SCHEMA_FINGERPRINT` pins a blake3 digest of the **transitive type closure**
of the three wire frames — `Command`, `GameEvent`, and `ReplayLog` — recomputed from
workspace source by `tests/protocol_schema.rs`. Change the shape of anything reachable on
the wire and the test fails, names the drift, and prints the new digest along with
instructions.

Attributes are inside the digest, because they are wire format:
`#[serde(rename)]` renames a field, `#[serde(skip)]` deletes one, `#[serde(default)]`
changes what a missing field means. None of those three is visible to `rustc`.

The fourth frame, `Envelope<T>`, is generic, so the source-scanning walk cannot follow it
(`T` resolves to nothing). Its two field names are pinned directly instead, by
`the_envelope_frame_has_exactly_the_expected_fields`.

### 4. Replay logs carry two versions, and both are checked.

A `ReplayLog` is a `Vec<Command>` plus the `hash_schema_version` it was recorded against,
inside the same versioned envelope. It answers two independent questions:

| Question | Guarded by | Failure |
|---|---|---|
| Can this build *read* the commands? | `protocol_version` | `VersionMismatch` |
| Can this build's state hashes be *compared* with the recorded ones? | `hash_schema_version` | `HashSchemaMismatch` |

Passing the first does not imply passing the second. A log can decode perfectly and still
replay into states whose hashes are incomparable with the recorded ones — silently
disabling the desync detection that invariant #9's history rests on. The protocol check
runs first, because a log we cannot read is a different failure from one we can read but
cannot verify.

---

## What is actually on the wire

The closure is **90 types, not 3**.

`GameEvent::CreatureDied` carries `Option<Characteristics>` (added for LKI correctness,
CR 603.10a). `Characteristics` holds `Vector<AbilityInstance>`, which reaches `Effect`,
`TargetFilter`, `Cost`, `KeywordAbility` — the whole card DSL, spanning both
`crates/engine` and `crates/card-types`.

Two consequences worth stating plainly:

- **Adding an `Effect` variant is a wire change.** Most primitive batches (PB-*) will
  therefore bump `PROTOCOL_VERSION`. That is not gate noise; it is what strict lockstep
  means. The bump is two edits and the failing test tells you both.
- **The closure bottoms out cleanly.** `GameState`, `PlayerState`, `StackObject` and
  `CardDefinition` are *not* reachable from `Command`/`GameEvent`. That is exactly why
  protocol versioning and `HASH_SCHEMA_VERSION` can remain separate concerns — and it is
  asserted (`CLOSURE_MUST_NOT_CONTAIN`), so if whole-state sync ever leaks into an event,
  it fails here first and gets decided on purpose.

---

## Bump procedure

When `protocol_schema_fingerprint_is_pinned` fails, in one commit:

1. Bump `PROTOCOL_VERSION` in `crates/engine/src/rules/protocol.rs` and add a `History`
   line saying what moved.
2. Paste the new digest (printed by the failing test) into `PROTOCOL_SCHEMA_FINGERPRINT`.
3. Update the `protocol_version_sentinel` test's expected literal.

For a **semantic** change that does not move the shape — redefining what an existing
`u32` counts, say — the fingerprint will *not* fire. Bump `PROTOCOL_VERSION` by hand.
The sentinel test then fails, which is the intended forcing function: a bump is always
one deliberate, reviewable edit.

There is exactly one case where you re-pin the digest **without** bumping the version:
when you widen the *definition* of the closure itself — adding a `SCAN_ROOTS` entry, a
`PROTOCOL_ROOTS` entry, or an `EXTERNAL_TYPES` entry. The digest moves because coverage
grew, not because the wire did. Say so in the commit message.

---

## Known holes

Stated on the record rather than papered over.

| Hole | Consequence | Disposition |
|---|---|---|
| **Semantic drift.** Same shape, new meaning. | Fingerprint stays put; two builds silently disagree about what a field means. | No mechanical fix exists. Bump by hand; the sentinel makes the bump reviewable. |
| **External types.** `im::OrdMap`, `Vec`, `Option` are allowlisted in `EXTERNAL_TYPES`. | An `im` upgrade that changes its serialized form moves the wire without moving the digest. | Accepted. Pinning `im`'s serialized shape means vendoring it. Note it when bumping `im` (see SR-10, `scutemob-62`). |
| **Variant reordering is a deliberate false positive.** | serde's external tagging keys on names, so a pure reorder is wire-*compatible*, yet the digest moves and forces a bump. | Accepted. The cost is one needless bump; the alternative — a variant-sorting normalizer — is more code that can be wrong in the *unsafe* direction. |
| **Formatting churn is a false positive too.** Whitespace is normalized, but rustfmt rewrapping a long field type *inserts a trailing comma*, which is a token. | A rustfmt version change can move the digest with no wire change. Verified, not theorized. | Accepted, and mitigated: `cargo fmt --check` is a CI gate, so the tree is canonical for the pinned toolchain. This can only fire when rustfmt's version changes — which is what `scutemob-63` (SR-11, pin the toolchain) prevents. |
| **`#[serde(skip)]` on `PendingTrigger`.** `kind`, `data`, `embedded_effect` are skipped. | A serialized pending keyword trigger deserializes as an anonymous `Normal` trigger with no payload. | Open: `scutemob-68` (SR-16). `PendingTrigger` is not in the `Command`/`GameEvent` closure, so this is a *state-sync* bug, not a protocol one — but M10 state sync will hit it. |
| **JSON only.** `decode`'s staged probe re-reads the same bytes by field name. | The roadmap floats MessagePack as an optional upgrade. `rmp-serde` in compact mode encodes structs as arrays, so a field-name probe breaks. | If MessagePack is adopted, use its named-field (struct-as-map) mode, or move the version tag out of band into the WebSocket subprotocol string. |
| **No build identity on the wire.** Two builds at the same `PROTOCOL_VERSION` with uncommitted local changes both claim v1. | A developer testing against a colleague's server can still desync. | `PROTOCOL_SCHEMA_FINGERPRINT` is `pub` precisely so a future handshake can exchange it. Not built; no networked client exists yet. |

---

## Adversarial demonstration

Per the SR-5 lesson — *demonstrate a gate adversarially, not existentially* — the
question is not "does the gate fire?" but "here are the cheapest ways to change the wire
without tripping it, and here is what catches each." Each attack below was applied to the
tree, pushed **past the compiler** where `rustc` objected, run, and reverted.

| Attack | Compiles? | What catches it |
|---|---|---|
| `#[serde(skip)]` a `Command` field | yes | `protocol_schema_fingerprint_is_pinned` |
| `#[serde(rename_all)]` on `Command` | yes | `protocol_schema_fingerprint_is_pinned` |
| `#[serde(rename)]` one `Command` variant | yes | `protocol_schema_fingerprint_is_pinned` |
| Add a `GameEvent` variant | **no** | `rustc` — `hash.rs`'s exhaustive match |
| Add a `GameEvent` variant **and satisfy that match** | yes | `protocol_schema_fingerprint_is_pinned` — *and nothing else* |
| Add a field to `ReplayLog` | yes | `protocol_schema_fingerprint_is_pinned` (only after review; see below) |
| Blind the declaration scanner | yes | `scanner_is_not_vacuous` + 4 others |
| Blind the closure walk (roots only) | yes | `protocol_closure_is_not_vacuous_and_is_bounded` |
| Bump `PROTOCOL_VERSION` with no shape change | yes | `protocol_version_sentinel` |
| "Simplify" `decode` to a single pass | yes | `version_is_checked_before_the_payload_is_parsed` |
| Reorder two `GameEvent` variants (wire-compatible) | yes | `protocol_schema_fingerprint_is_pinned` (accepted false positive) |

The fifth row is the one that justifies the whole fingerprint apparatus. Adding a
`GameEvent` variant *does* fail to compile — `hash.rs` has an exhaustive match. But that
error only tells the author to **assign a hash byte**. Once they dutifully do so, the
workspace builds clean and all 3 100+ other tests pass. The single thing standing between
that author and two mutually-incomprehensible builds both claiming `protocol_version: 1`
is `protocol_schema_fingerprint_is_pinned`.

This is SR-5's finding one enum over: *"'adding a variant fails to compile' was already
true — but it only forces you to assign a hash byte and a display string. Neither is
behavior."*

### The hole the review found

The first version of this gate rooted the closure at `Command` and `GameEvent` only —
"the two enums that *are* the protocol." But `encode_replay_log` puts a **third** frame on
the wire, `ReplayLog { hash_schema_version, commands }`, and nothing reachable from
`Command`/`GameEvent` mentions it. Its `commands: Vec<Command>` contents were covered; its
own two-field frame was not. Adding a field to `ReplayLog` compiled clean, changed the
replay-log wire format, and tripped nothing — the exact process-not-machine failure this
gate exists to delete, in the one place the acceptance criteria explicitly named.

`ReplayLog` is now a `PROTOCOL_ROOTS` entry, and the attack (add `pub author: String`) was
re-run to confirm it fires. This is the sixth consecutive SR task where the review found
the gap in the *gate* rather than a bug in the code. The pattern is stable enough to name:
**the author checks that the gate fires on the thing they were thinking about, and does
not enumerate the things the gate is not pointed at.** Two further guards came out of the
same review, both of which pass today and would not have been noticed failing later:

- `declared_type_names_are_unique` — the type index is keyed by bare name and keeps the
  first declaration, so a future same-named type in another module would silently make the
  digest hash the wrong declaration.
- `no_workspace_type_shadows_an_external_type_name` — an `EXTERNAL_TYPES` entry suppresses
  that bare name *everywhere*, so declaring a workspace type called `Vector` or `Box` would
  quietly drop it out of the digest.

### Holes found by the guards while the guards were being written

Which is the argument for writing them:

- **`pub type RoomIndex = usize;`** is wire-bearing but is neither an enum nor a struct.
  `every_referenced_type_resolves` refused to let an unresolved type name pass silently.
  Type aliases are now indexed.
- **`EnchantControllerConstraint`'s `#[derive(...)]` is rustfmt-wrapped across three
  lines.** The original line-based attribute walk saw `)]`, decided it was not an
  attribute, and dropped the container's entire serde config out of the digest —
  silently. Attribute extraction is now bracket-matched, and
  `every_closure_type_shows_its_serialize_derive` is the denominator guard that found it.

---

## For M10 implementers

- Wrap every `ClientMessage` / `ServerMessage` in `Envelope` — or, better, make those
  types the envelope payload and never serialize a bare `Command`/`GameEvent`.
  `an_untagged_message_is_a_malformed_envelope` guarantees a bare one will not be mistaken
  for version 0.
- On `VersionMismatch` at connection time: refuse the connection and surface
  `err.to_string()` to the user. It names both versions and says what to do.
- The bug-report artifact in the M10 roadmap ("full event log + state diff + failing
  command as a loadable JSON reproduction case") is a `ReplayLog` plus events. Use
  `encode_replay_log` / `decode_replay_log` so the hash-schema check comes for free.
- Do not add a "compatibility shim" that translates v(N-1) messages. If you need one,
  that is a design conversation about whether lockstep still holds — not a patch.
