# PB-DX48 — implementation plan (engine is DONE; this file specifies the TESTS)

Task `scutemob-219`. Read `memory/primitives/pb-DX48-execution-notes.md` §0 first
(the wire prediction and movement budget, committed before any code).

## §1 What already shipped (commit `72c0770c`) — do NOT re-implement

**Part A — one dispatcher.** `crates/engine/src/rules/events.rs` gains
`permanent_targeted_events(state, controller, stack_object_id) -> Vec<GameEvent>`,
and `push_target_announcement` now emits `TargetsAnnounced` **then** the
`PermanentTargeted` events, in that order. The three hand-rolled loops that used to
do this were DELETED:

* `rules/casting.rs::handle_cast_spell` (its `battlefield_targets` collection and loop),
* `rules/abilities.rs::handle_activate_ability` (same shape),
* `rules/abilities.rs::handle_activate_bloodrush` (an unconditional single push).

Predicate, unchanged from those three copies: a `Target::Object` whose
`zone_at_cast == Some(ZoneId::Battlefield)`.

**Part B — bounded fixpoint.** `rules/engine.rs::check_and_flush_triggers` is now a
loop over `MAX_BECOMES_TARGET_WAVES = 16` with an exactly-once scan cursor. Wave 0
is byte-for-byte the old pass. Waves 1+ scan only the `PermanentTargeted` events
appended since the previous wave. `run_delayed_trigger_cleanup` was lifted out
verbatim and still runs on wave 0 only.

**Two findings that must appear in the tests, because they are the batch's content:**

1. *Emission alone is not sufficient at the flush sites, and neither seed says so.*
   `check_and_flush_triggers` scanned the command's events and only then called
   `flush_pending_triggers`, so the events a flush itself produced were scanned by
   nothing. A Ward trigger caused by a **triggered** ability sat in
   `state.pending_triggers` until the next command — after priority had been
   granted, which CR 603.3b forbids.
2. *A hook inside `flush_sorted` was tried FIRST and defeated BY EXECUTION.* It
   dispatched the same `PermanentTargeted` that `check_and_flush_triggers` then
   re-scanned, and **Ward fired twice** (two `AbilityTriggered`, two ward stack
   objects). Exactly-once scanning is the mechanism that makes "once per targeting
   event" true, and a test must pin the count, not merely its non-zero-ness.

## §2 Census, RE-VERIFIED at HEAD by the inverse method (do not re-derive by hand — gate it)

12 `push_target_announcement` sites =
**3 emitters** (`casting.rs::handle_cast_spell`, `abilities.rs::handle_activate_ability`,
`abilities.rs::handle_activate_bloodrush`)
+ **5 with no Ward dispatch** — `handle_activate_forecast`, `flush_sorted` ×2 (the
modular arm and the main arm), `handle_scavenge_card`,
`engine.rs::handle_activate_loyalty_ability`
+ **4 structurally target-free today** (`targets: vec![]`, the `OOS-ENG2-3` sites) —
`copy.rs::resolve_cascade`, `copy.rs::resolve_discover`,
`resolution.rs::resolve_top_of_stack_inner` ×2 (cipher-copy, suspend).

**The memo's claim that the five-site census is exact and complete REPRODUCES.**

Populations (memo derivation, re-derive and PRINT):
* `KeywordAbility::Ward(_)` over `all_cards()`, deck-legal `Complete` → **3**:
  `adrix_and_nev_twincasters` (Ward 2), `miirym_sentinel_wyrm` (Ward 2, `#[default]`
  derive), `tyrranax_rex` (Ward 4). The other two Ward defs
  (`rith_liberated_primeval`, `vein_ripper`) are `partial`.
* `WhenBecomesTarget` / `WhenBecomesTargetByOpponent` → **6 defs, 0 deck-legal**
  (5 `partial` + 1 `inert`).
* The Disguise/Cloak `layers.rs` Ward(2) grant → **0** deck-legal members.

VERIFIED at HEAD: all three real Ward defs DO synthesize
`TriggerEvent::SelfBecomesTargetByOpponent` through
`enrich_spec_from_def` + `GameStateBuilder::build` (`state/builder.rs:405-450`), so a
real-def probe is possible and must be used.

## §3 Tests to write

### 3a. `crates/engine/tests/primitives/pb_dx48_ward_dispatch.rs` (new; add `mod` to `tests/primitives/main.rs`)

One probe per NEW site, each asserting the CR 702.21a dispatch by a **resolution
effect or a stack count**, never merely by the event:

* `t1` — a **triggered** ability (the headline: `flush_sorted`'s main arm) targeting an
  opponent's Ward permanent. Assert **exactly one** `PermanentTargeted` AND
  **exactly one** ward `AbilityTriggered` whose controller is the ward permanent's
  controller. **The count is the assertion** (finding 2 above). Then resolve and
  assert the targeting ability was COUNTERED (`GameEvent::SpellCountered` or the
  ward stack object's `CounterSpell` effect having removed the targeting entry) and
  the ward permanent took no damage.
* `t1b` — the same shape where the trigger's controller is the **ward permanent's own
  controller**: CR 702.21a says "an opponent controls", so **zero** ward triggers.
  This is the non-vacuity partner for `t1`.
* `t2` — `handle_activate_forecast` (site A3).
* `t3` — `handle_scavenge_card` (site A12).
* `t4` — `handle_activate_loyalty_ability` (site A13).
* `t5` — `flush_sorted`'s **modular** arm (site T6). Modular's auto-target scan picks
  the first artifact creature; give the ward creature the artifact type so it is the
  pick. If the arm cannot be reached with a ward artifact creature, say so IN THE
  TEST rather than dropping it.
* `t6` — a **real-def** probe covering all three deck-legal `Complete` Ward defs
  (`Adrix and Nev, Twincasters`, `Miirym, Sentinel Wyrm`, `Tyrranax Rex`), built via
  `enrich_spec_from_def` + `card_name_to_id`, each targeted by a triggered ability
  and each firing exactly once. Assert the ward COST differs (2/2/4) so the probe is
  reading the real def and not a stand-in.
* `t7` — **wave bound / no-cascade**: assert the ward trigger's own
  `SpellTarget.zone_at_cast` is `None`, which is the structural reason
  `MAX_BECOMES_TARGET_WAVES` truncates nothing reachable today. Assert it from the
  stack object, not from a comment.
* `t8` — **the three pre-existing emitter sites are unchanged**: a cast, an activated
  ability and a bloodrush each still emit exactly the same `PermanentTargeted`
  payload after Part A folded their loops into the helper. Bloodrush specifically:
  prove the deleted unconditional push and the new predicate agree.

### 3b. `crates/engine/tests/core/pb_dx48_announcement_site_roster.rs` (new)

Machine gates, all parsing SOURCE for the SITE axis and walking `all_cards()` for the
CARD axis (SR-36):

* `r1` — the inverse-method census as a gate: parse
  `crates/engine/src/rules/{abilities,casting,copy,engine,events,resolution}.rs`, collect every
  `push_target_announcement(` call site, and assert the count and the enclosing
  function names against a pinned list with a REASON per entry. **A new site must be
  classified here.** (`pb_eng2_targets_announced.rs::every_announcement_site_is_classified`
  already pins the wider stack-push census; this row is the narrower
  `push_target_announcement` axis and must state that it is a second axis, not a
  duplicate.)
* `r2` — **there is exactly ONE `GameEvent::PermanentTargeted` construction site in
  `crates/engine/src`**, and it is `rules::events::permanent_targeted_events`. This is
  the gate that makes "one mechanism" true rather than asserted. Ratchet at the
  measured value, with the ceiling equal to the measurement (PB-DX45's lesson: a
  ratchet's slack is its blind spot).
* `r3` — the Ward population, PRINTED: walk `all_cards()`, print every def declaring
  `KeywordAbility::Ward`, its cost, and its `Completeness`; assert the deck-legal
  `Complete` set is exactly the three named above.
* `r4` — the `WhenBecomesTarget` / `WhenBecomesTargetByOpponent` population, PRINTED;
  assert **0** deck-legal `Complete` members and print the 6 non-deck-legal ones with
  their markers, so a future promotion reddens this row.
* `r5` — an **inverse** axis: defs whose ORACLE TEXT prints "Ward" but which declare no
  `KeywordAbility::Ward` (the PB-DX26/DX43/DX45/DX47 lesson — a roster derived from one
  declaration construct measures that construct). Ratcheted, members printed.
* `t_census_report` — print every population above so the numbers are PUBLISHED, never
  transcribed (PB-DX8's rule).

### 3c. `crates/simulator/tests/pb_dx48_ward_channel.rs` (new)

Reachability per the UI-4 standard — existence is never sufficiency:

* `c1` — a real `LocalGame` with a human seat (`StubProvider` + `HeuristicBot`),
  a real Ward def on the opponent's battlefield, and a triggered ability targeting
  it. Assert by RESOLUTION EFFECT: the targeting ability is countered and the ward
  permanent is untouched. Follow `crates/simulator/tests/pb_dx45_optional_cost_channel.rs`
  for the drive helpers (`drive_until`, `AdvanceOutcome::AwaitingHuman`, `submit`).
* `c2` — the **bot path**: the same shape with both seats bot-driven, asserted the
  same way. `StubProvider` must need no change; assert that rather than assume it.
* `c3` — a **non-default** answer wherever the drive offers a choice (the trigger's
  own CR 603.3d target announcement gives one whenever ≥2 creatures are legal): pick
  the ward permanent explicitly, not the engine's default, so the probe
  discriminates "the human chose" from "the engine defaulted".

### 3d. The deviation pin — INVERT, do not delete

`crates/engine/tests/primitives/pb_eng2_targets_announced.rs:384-392` asserts that
`flush_sorted` emits **no** `PermanentTargeted`, with an in-test instruction to INVERT
when `OOS-ENG2-1` closes.

**A literal inversion of that assertion would be RED after a correct fix, and that is
itself a finding to state rather than paper over.** The fixture is Fell Specter,
whose trigger targets `TargetOpponent` — a **player**. `PermanentTargeted` carries an
`ObjectId` and cannot express a player target, so no correct engine emits one there.
**The pin could never have discriminated the fix in the direction it asked for**, which
is why it stayed green through this batch's whole engine change. Therefore:

* rewrite the assertion's CLAIM from "DEVIATION PIN (OOS-ENG2-1)" to a CR 702.21a
  **scope** pin: a player target raises no `PermanentTargeted`, which is CR-correct,
  not a deviation — and say in the comment that this is the inversion, that the
  boolean could not flip, and why;
* add the positive sibling in the SAME file: the same `flush_sorted` path with an
  **object** target on an opponent's permanent DOES emit exactly one. That sibling is
  the discriminator the original pin was asking a successor to produce.

Disclose this in the delta as a renamed/inverted test, not netted out.

## §4 Standards this batch is held to

* Every new gate/probe proven RED by an **executed** revert; record the matrix in
  `memory/primitives/pb-DX48-execution-notes.md`. An UNDISCRIMINATED row is disclosed
  in the test itself, not only in `memory/`.
* Non-vacuity floors on every roster row.
* `clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check` and
  `tools/check-defs-fmt.sh` clean against the FINAL tree.
* No assertion weakened anywhere.
