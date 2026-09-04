# PB-DX52 — execution notes

**Task**: `scutemob-229` · v4 queue rank 14 · seeds `OOS-DX25b-1` (headline) + `OOS-DX25b-5` (rider).
**Branch**: `feat/pb-dx52-bolt-bends-printed-or-ability-half-is-unreachable-an`
**Merge base**: `cecf0ba0`.

---

## §0 Stage 0 — measured, BEFORE any production line changed

### §0.1 Pre-edit baseline (executed)

`cargo test --workspace --no-fail-fast` to a file:
**5,117 passed / 0 failed / 5 ignored**, **64** test-result lines.
**Reproduces PB-DX36's published close pin exactly** (5,117 / 0 / 5, 64 targets) — no
discrepancy to report, which is the first time in five batches the inherited pin
reproduces without a correction (`OOS-DX51-5` was the last one that did not).

*Procedural note, recorded because it cost twenty minutes and will recur*: a wait loop
written as `for i in ...; do pgrep -f "cargo test --workspace" || break; done` **never
exits** — `pgrep -f` matches the loop's own `bash -c` command line, which contains the
needle verbatim. It reported STILL RUNNING for 20 minutes after the run had finished.
Filed as `OOS-DX52-1`.

### §0.2 Version / count pins at HEAD (read off source, not remembered)

| Pin | Value at HEAD | Cite |
|---|---|---|
| `PROTOCOL_VERSION` | **42** | `crates/engine/src/rules/protocol.rs:490` |
| `PROTOCOL_SCHEMA_FINGERPRINT` | `9d75f591…02aa47` | `protocol.rs:507-508` |
| `HASH_SCHEMA_VERSION` | **83** | `crates/engine/src/state/hash.rs:1046` |
| PROTOCOL closure type count | **98** | gate output, PB-DX36 |
| HASH closure type count | **132** | gate output, PB-DX36 |
| Coverage | 1,139 / 1,803 = 63.2% | `docs/authoring-status.md` |

### §0.3 THE DESIGN DECISION — `Target::StackObject`, not `state.objects` registration

The brief offers two options and calls the seed's own prescription a claim. Both were
costed by executed census before a line was written.

#### Option (b) — register ability stack entries in `state.objects` (CR 109.1's literal reading): REJECTED

Measured blast radius (241 full-map walk sites; 207 production; 9 with no zone filter at
all; 4 whose zone filter is *negative* or *includes `Stack`*):

1. **HARD BLOCKER — `crates/simulator/src/invariants.rs:79`** (`check_zone_integrity`)
   emits an `InvariantViolation` for every `state.objects` entry that is not in some
   zone's `object_ids()`. So an ability entry cannot merely be inserted; it must also be
   pushed into `zones[ZoneId::Stack]`.
2. Doing that moves **`public_state_hash`** — `hash.rs:8867-8879` is **zone-driven**, so
   the Stack zone's `Vector<ObjectId>` gaining an element changes the digest for *every
   game with an ability on the stack*, and the full `GameObject` is then hashed at
   `:8875`.
3. **`loop_detection.rs:132`** (`compute_mandatory_state_hash`) is map-driven and
   unconditional (`Hand | Library => continue, _ => hash`), and it *already* hashes
   `state.stack_objects` separately at `:145` — so an ability entry would be
   **double-counted** in the mandatory-loop fingerprint.
4. **A TARGETING CORRECTNESS REGRESSION, not a cost**: `queries.rs:428` and
   `retarget.rs:267` both offer `Target::Object(id)` for any object whose zone is
   `Battlefield | Stack | Graveyard(_)`, and `casting.rs`'s `TargetSpell` arm validates
   by `obj.zone != ZoneId::Stack` **alone, with no spell/ability discriminator**. An
   ability entry claiming `ZoneId::Stack` therefore becomes a legal target for
   *"counter target spell"* — CR 115.4-wrong, and a new defect this batch would have
   shipped while closing an old one.
5. **`sba.rs:451`** removes any `is_token` object not on the battlefield from
   `state.objects` — a landmine one field-default away.
6. `GameObject` (`card-types/src/state/game_object.rs:1069`) has **two non-`Option`
   fields that must be fabricated**: `characteristics: Characteristics` (requires a
   name, rules text, four `OrdSet`s, three `Vec`s, two `Vector`s — and **no `CardType`
   fits**, there is no `CardType::Ability`) and `zone: ZoneId`. There is no "kind"
   discriminator on `GameObject` at all: nothing in that map can say *"I am not a card."*
7. The view model already builds its stack view from `state.stack_objects()`
   (`view-model/src/lib.rs:513`), so registration produces a **duplicate
   representation**, not a missing one made present.

**The CR argument for (b) does not survive contact with the model.** CR 109.1's "object"
is a rules concept; `state.objects` is this engine's **card**-object map, and CR 113
abilities are modelled by `state.stack_objects`. Registering them in both makes the model
*doubly* represented, not *more* faithful.

#### Option (a) — `Target::StackObject(ObjectId)` naming the stack ENTRY's own id: TAKEN

Measured blast radius: **≈22 files to compile and pass gates** (13 production + 7
test/`cfg(test)` forced by exhaustive matches, + `protocol.rs` and
`tests/core/protocol_schema.rs`), plus **13 wildcard/`if let`-without-`else` sites across
8 files that the compiler will NOT flag** — enumerated and each dispositioned in §0.5 —
plus the untyped `tools/replay-viewer/frontend/` JS.

**The one shape choice that collapses most of the work**: `Target::StackObject(id)`
carries the **stack-entry** id, and
`state::stack_registry::stack_index_for_announced_target`'s **first clause is already
`so.id == announced`**. So every existing consumer — `Effect::ChangeTargets`,
`Effect::CounterSpell`, `Effect::CopySpellOnStack`, `casting.rs`'s two single-target arms
— resolves a stack-entry id through the shared arithmetic *with no new plumbing at all*.
`resolve_effect_target_list_indexed` (`effects/mod.rs:8337-8348`) likewise **already**
accepts a stack-entry id (its `exists_on_stack` clause, written for CR 702.21a Ward), so
`Target::StackObject(id)` maps to `ResolvedTarget::Object(id)` at that single site.

**`ResolvedTarget` is deliberately NOT given a third variant.** That enum has ~55
`if let ResolvedTarget::Object(..)` sites with no `else` in `effects/mod.rs`; growing it
would create 55 silent-swallow sites to buy nothing, because the id it would carry is the
same id and the one function that consumes it already resolves both spaces. Stated here
so a later batch does not "finish the job".

### §0.4 WIRE PREDICTION — written before any production line changed

**Predicted, per half:**

* **PROTOCOL 42 → 43, ONE bump.** `Target` is in the wire closure: `Command::CastSpell.targets: Vec<Target>` (`rules/command.rs:105`), `:405`, `:749`, `:779`, and `SpellTarget` via `GameEvent::TargetsChanged` (`rules/events.rs:1424/1426`) and `GameEvent::TargetsAnnounced` (`:1571`). Adding a variant is a *shape* change, so `PROTOCOL_SCHEMA_FINGERPRINT` moves.
* **HASH 83 → 84, ONE bump.** `impl HashInto for Target` (`state/hash.rs:4864-4877`) is reached from `StackObject::hash_into` (`:4889`) via `SpellTarget`, which is inside the `GameState` serde closure the DECLARATION fingerprint is taken over. The new `2u8` stream tag is *append-only*, so no existing hash value changes — but the **declaration** digest moves, which is what the gate pins.
* **Closure type counts predicted UNCHANGED at 98 (PROTOCOL) / 132 (HASH)**, and the reason is stated rather than asserted: this batch adds a **variant to an existing type**, not a new type. No `TargetRequirement` variant is added either.
* **Both are ONE bump for the whole PB** — the card-def half (`deflecting_swat`) mints no type, and the offer/validate/re-check halves add only functions.
* **A two-step observation is expected** (v40/v82/v83's pattern): with the variant in the tree and hashed but BEFORE the version bump, `declaration_fingerprint_is_pinned` should be RED while `stream_fingerprint_is_pinned` may be GREEN, because `canonical_fixture()` carries no stack object with a `StackObject` target. To be observed, not assumed.

**Not owed, stated rather than left implied**: **no `GameEvent::PermanentTargeted` is
owed for a stack-entry target.** CR 702.21a Ward is *permanent*-only ("Whenever this
permanent becomes the target of a spell or ability an opponent controls"), and an ability
on the stack is not a permanent. `rules/events.rs::permanent_targeted_events`'s
`_ => None` arm therefore gives the CR-correct answer for the new variant by accident —
so the arm is made **explicit** rather than left to the wildcard, since a wildcard that
happens to be right is not a decision anyone made.

### §0.5 The 13 sites the compiler will NOT flag — disposition table

| # | Site | Verdict |
|---|---|---|
| 1 | `resolution.rs:4294-4311` Modular death-trigger `_ => None` | correct-by-accident → made explicit |
| 2 | `events.rs:1670-1681` `permanent_targeted_events` `_ => None` | **CR-correct** (Ward is permanent-only) → made explicit with the CR cite |
| 3 | `resolution.rs:1912-1918` Aura attach `else { None }` | correct (an Aura cannot enchant a stack entry) → explicit |
| 4 | `resolution.rs:4504-4510` Scavenge `else { None }` | correct → explicit |
| 5 | `casting.rs:3907` Aura battlefield check | correct → explicit |
| 6 | `casting.rs:6506` CR 601.2c inter-target distinctness | **stated residual**: no corpus requirement pairs two stack-object slots; recorded, not silently skipped |
| 7 | `abilities.rs:564` AttachEquipment | correct → explicit |
| 8 | `abilities.rs:634` Fortify | correct → explicit |
| 9 | `view-model/src/redact.rs:246` | **must be handled** — a redaction hole is Invariant 7 |
| 10 | `testing/replay_harness.rs:947-950` | script channel — handled |
| 11 | `targeting.rs:62` `is_unchosen_slot` | correct (the SENTINEL placeholder is always `Target::Object`) |
| 12 | `resolution.rs:7713` equality compare | correct |
| 13 | `pb_dp8_trigger_target_choice.rs:311` (test) | correct |

