# Primitive Batch Plan: PB-OS3 — WhenTappedForMana trigger target dispatch (OOS-EF6-1)

**Generated**: 2026-07-19
**Primitive**: Behaviour-only fix. Reclassify the `PendingTrigger` kind used by the mana-trigger
dispatch path so that a `TriggerCondition::WhenTappedForMana` triggered ability that declares a
DSL target actually resolves that target (instead of silently dropping it). No new DSL/wire type.
**CR Rules**: 605.1a/605.1b (mana-ability criteria), 605.5a (targeted/non-mana-trigger is NOT a
mana ability → goes on the stack), 603.3d (auto-target selection / fizzle if no legal target),
106.12a (WhenTappedForMana trigger semantics — already handled by SR-28).
**Cards affected**: 1 flip (`forbidden_orchard`: `known_wrong` → `Complete`) + 6 no-regression
roster cards verified untouched.
**Dependencies**: PB-EF2 (`TokenSpec.recipient`, shipped), PB-EF6 (`TargetRequirement::TargetOpponent`
+ auto-picker, shipped), PB-EF12 (`AddManaAnyColor` real-colour choice, shipped), PB-EF3
(EF-W-MISS-10 — the analogous kind/index-space fix on the *attack* path, shipped). All present.
**Deferred items from prior PBs**: none for this seed.

---

## Chosen fix: Option B (reclassify the queued trigger kind). No wire bump.

**Decision: Option B. Verified sound end-to-end. Recommended and endorsed.**

The single engine change is one identifier in `rules/mana.rs`:
`PendingTriggerKind::Normal` → `PendingTriggerKind::CardDefETB` for the stack-push branch of a
targeted / non-mana WhenTappedForMana trigger.

### Why Option B is correct (traced against source, not asserted)

Root cause (verified): `fire_mana_triggered_abilities` (`rules/mana.rs:675-716`) iterates
**`def.abilities`** and, for a targeted WhenTappedForMana ability, queues
`PendingTrigger::blank(src, player, PendingTriggerKind::Normal)` with
`trigger.ability_index = ability_idx` (the **raw `def.abilities` index**, `mana.rs:711-714`).
But `flush_pending_triggers` (`rules/abilities.rs`) resolves a `Normal`-kind trigger's targets from
the **runtime** `obj.characteristics.triggered_abilities.get(trigger.ability_index)` vec
(`abilities.rs:6980-7004`, the PB-EF3-A2 "runtime vec is authoritative, do not fall through" path),
and `enrich_spec_from_def` has **no `WhenTappedForMana` block**, so that runtime vec has no entry.
Net: declared `targets` are unreachable; `TargetOpponent` resolves to nothing (proven empirically:
wiring `recipient` produced 0 tokens).

`CardDefETB` is the existing sibling kind whose lookups use the **raw `def.abilities` index** — the
exact index the mana path already holds:

- **`has_ability_targets`** (`abilities.rs:6876-6900`): the `CardDefETB` arm reads
  `def.abilities.get(trigger.ability_index)` → sees `targets = [TargetOpponent]` → `true`.
- **Target resolution** (`abilities.rs:6979-7025`): the `CardDefETB` branch again uses
  `def.abilities.get(trigger.ability_index)` (`:7006-7020`) → returns `[TargetOpponent]`.
- **Auto-picker** (`abilities.rs:7076-7091`): `TargetRequirement::TargetOpponent` picks the first
  active opponent in `turn_order` that isn't the controller; contributes `None` (trigger removed,
  CR 603.3d) if the source has NO opponent — never falls back to controller (no illegal self-target).
- **Stack-object build** (`abilities.rs:8107-8125`): `Normal` and `CardDefETB` build the *identical*
  `StackObjectKind::TriggeredAbility { source_object, ability_index, .. }`. The only differences are
  `is_carddef_etb: false → true` and `embedded_effect: <clone> → None`. The mana path's
  `PendingTrigger::blank` already carries `embedded_effect = None`, so nothing is lost.
- **Resolution** (`rules/resolution.rs:1978-2003`): the `is_carddef_etb == true` branch resolves the
  effect via `def.abilities.get(ability_index)` (the registry raw index) → the `CreateToken` effect
  (CR 605.5a). The current `Normal` path already reaches the same effect via the registry *fallback*
  at `:1951-1977` (the runtime vec has no entry, so it falls through), so effect execution is
  unchanged — only target resolution is fixed.
- **No spurious doubling**: `compute_trigger_doubling` → `doubler_applies_to_trigger`
  (`abilities.rs:8274-8340`) keys strictly on `trigger.triggering_event`, never on
  `PendingTriggerKind`. `PendingTrigger::blank` leaves `triggering_event = None`, so no
  Panharmonicon/Yarok/CreatureDeath doubler matches. Switching the kind does not touch
  `triggering_event`. (Confirmed CardDefETB is already used across many non-ETB events —
  SelfAttacks etc. — so the kind is purely an index-space marker, not an ETB semantic.)
- **Immediate-mana branch untouched**: the `targets.is_empty() && is_mana_producing_effect(effect)`
  branch (`mana.rs:698-707`) is not modified. Only the `else` (stack-push) branch changes.

- **`CardDefETB` is an existing enum variant (hash arm 46 in the closure).** Adding no field, no
  variant, no DSL type → **NO PROTOCOL/HASH bump.**

### Why NOT Option A

Option A (add a `WhenTappedForMana` block to `enrich_spec_from_def` + iterate the runtime
`triggered_abilities` vec so `ability_index` becomes a runtime index) is strictly more invasive:
it drops `ManaSourceFilter` unless that is also carried onto the runtime `TriggeredAbilityDef`, and
there is **no `TriggerEvent::SelfTappedForMana`** variant — adding one lands inside the SR-8
fingerprint closure and would force a PROTOCOL bump. Option B avoids all of this. **Option A is the
fallback only if Option B were unsound; it is not — Option B is endorsed.**

---

## CR Rule Text

**605.1a** — An activated ability is a mana ability if: it doesn't require a target, it could add
mana when it resolves, and it's not a loyalty ability.
**605.1b** — A triggered ability is a mana ability if: it doesn't require a target, it triggers from
the activation/resolution of an activated mana ability or from mana being added, and it could add
mana when it resolves.
**605.5a** — An ability *with a target* is NOT a mana ability, even if it could add mana. It follows
the normal rules for triggered abilities (goes on the stack).
**603.3d** — When a triggered ability goes on the stack, if a required choice has no legal option,
the ability is simply removed from the stack (auto-target fizzle).

`Forbidden Orchard` oracle (MCP-confirmed): "{T}: Add one mana of any color. / Whenever you tap this
land for mana, **target opponent** creates a 1/1 colorless Spirit creature token." Its second
ability declares a target → by CR 605.5a it is correctly NOT a mana ability and belongs on the
stack (which the engine already does); only the target *dispatch* was broken.

---

## Engine Changes

### Change 1: Reclassify the mana-trigger stack-push kind

**File**: `crates/engine/src/rules/mana.rs`
**Site**: `fire_mana_triggered_abilities`, the `else` branch at **`mana.rs:708-715`** (the
"Normal triggered ability with targets or non-mana effect: push to stack" branch).
**Action**: change

```rust
let mut trigger =
    PendingTrigger::blank(trigger_source_id, player, PendingTriggerKind::Normal);
trigger.ability_index = ability_idx;
```

to use `PendingTriggerKind::CardDefETB` instead of `PendingTriggerKind::Normal`. `ability_idx` is
already the raw `def.abilities` index — leave it. Update the adjacent comment to explain that
CardDefETB is used so `flush_pending_triggers` resolves declared `targets` via the raw
`def.abilities` index (cite CR 605.5a and reference PB-EF3's EF-W-MISS-10 index-space rationale).
**CR**: 605.5a (targeted trigger on the stack), 603.3d (auto-target).
**Do NOT touch** the immediate-mana branch (`mana.rs:698-707`).

### Change 2: Exhaustive match updates

**None required.** `CardDefETB` is a pre-existing `PendingTriggerKind` variant already handled in
every match site (`has_ability_targets`, target resolution, stack-object build, resolution). No new
enum variant, field, or DSL type is introduced. `cargo build --workspace` confirms no new arms are
needed. (No `state/hash.rs`, replay-viewer, or TUI arm changes — those exhaust on
`StackObjectKind`/`KeywordAbility`, which are unchanged.)

**Wire verdict**: **No PROTOCOL/HASH bump expected.** `PROTOCOL_VERSION` stays 18,
`HASH_SCHEMA_VERSION` stays 55. If the runner finds a bump is somehow forced, **STOP and flag it as
a re-scope signal** — do not silently plan a bump.

---

## Card Definition Fixes

### forbidden_orchard.rs (`known_wrong` → `Complete`)

**Oracle**: "{T}: Add one mana of any color. / Whenever you tap this land for mana, target opponent
creates a 1/1 colorless Spirit creature token."
**Current state**: `completeness: Completeness::known_wrong(...)` with a long marker documenting the
WhenTappedForMana target-dispatch gap; the `CreateToken` effect deliberately omits `recipient`
(defaults to `PlayerTarget::Controller` — token goes to the OWN controller, inverting the drawback);
`targets: vec![TargetRequirement::TargetOpponent]` is already declared but was unreachable.
**Fix** (apply only after Change 1 lands and the compose test is green — verify both halves compose):
1. Add `recipient: PlayerTarget::DeclaredTarget { index: 0 },` to the WhenTappedForMana
   `TokenSpec` (field exists at `crates/card-types/src/cards/card_definition.rs:3868`). Index 0 is
   the auto-selected opponent that the fixed dispatch now places on the stack object.
2. Remove the "Approximation: Spirit token for controller" comment and the `TODO` block
   (`forbidden_orchard.rs:29-62`) describing the now-fixed dispatch gap.
3. Flip `completeness` from `known_wrong(...)` to `Completeness::Complete`. The other half
   (`AddManaAnyColor` → real chosen colour) was already fixed by PB-EF12; both halves now compose
   (chosen-colour mana produced AND opponent-targeted Spirit created). Cite CR 605.5a in a short
   header comment for the token half.
4. If — and only if — the runner finds the two halves do NOT compose (e.g. the token still misroutes
   under a real 4-player game), leave an honest `known_wrong`/`partial` marker recording the true
   remaining blocker and do NOT flip. (Do not expect this — the chain is verified.)

## New Card Definitions

None.

---

## Roster sweep (SR-34/36 — from `all_cards()`, not grep)

The runner must enumerate `all_cards()` and filter for defs carrying a
`TriggerCondition::WhenTappedForMana` ability. Indicative (grep, to be confirmed via `all_cards()`):
**7 defs** — `badgermole_cub`, `crypt_ghast`, `forbidden_orchard`, `leyline_of_abundance`,
`miraris_wake`, `wild_growth`, `zendikar_resurgent`.

Only **forbidden_orchard** declares a non-empty `targets` on its WhenTappedForMana ability. The
other **6** were verified (this plan) to use `targets: vec![]` with a mana-producing effect
(`Effect::AddMana` / `Effect::AddManaMatchingType`), both recognized by
`is_mana_producing_effect` (`mana.rs:781-795`). Therefore all 6 take the **immediate-mana branch**
(`mana.rs:698-707`), which Change 1 does **not** touch — they never reach the reclassified
stack-push branch, so they cannot regress.

**How the runner proves no regression on the 6 doublers**: (a) re-confirm from `all_cards()` that
each of the 6 has `targets.is_empty()` on its WhenTappedForMana ability and a mana-producing effect
(so it routes through the untouched immediate branch); (b) add/keep an executing no-regression test
(below) that fires one inline mana-doubler and asserts its added mana is unchanged. Report the full
7-card set in the close-out.

---

## Unit Tests

**File**: extend `crates/engine/tests/rules/mana_triggers.rs` (an existing member of the `rules`
test group — SR-9a: never add a top-level `tests/*.rs`; add to an existing group module). The file
already has `build_registry`, `make_spec`, `mana_pool`, `pass_all`, and `p(n)` helpers.

**Tests to write / update**:

- **`test_mana_trigger_forbidden_orchard` (UPDATE existing, `:809`)** — CR 605.5a. The kind
  assertion at `:856-860` must change from `PendingTriggerKind::Normal` to
  `PendingTriggerKind::CardDefETB` (Option B reclassifies it). `ability_index` stays 1 (raw
  `def.abilities` index). Keep the "trigger queued not immediately resolved" and "1 Spirit created"
  assertions. In this 2-player game the Spirit now goes to p2 (the sole opponent) — strengthen the
  final assertion to check the Spirit's controller/owner is **p2**, not p1. Add a CR 605.5a citation
  in the doc comment for the recipient behaviour.

- **`test_forbidden_orchard_token_goes_to_declared_opponent_4player` (NEW)** — CR 605.5a + 603.3d.
  4-player game (`p1..p4`), p1 controls Forbidden Orchard, p1 active. Tap for a **chosen colour**
  (`Command::TapForMana { chosen_color: Some(ManaColor::White), .. }`). Assert:
  1. p1's mana pool gained the chosen colour (the any-colour half composes — PB-EF12).
  2. Exactly one `PendingTrigger`, `kind == CardDefETB`, `ability_index == 1`.
  3. After `pass_all` twice, inspect the resolved stack (or capture the stack object before
     resolution) and assert its target is `Target::Player(p2)` — the **declared** opponent
     (first active opponent in turn order), proving target dispatch, not just the end effect.
  4. Exactly one Spirit token exists and its `controller`/`owner` is **p2**.
  5. **Decoy proof**: p3 and p4 (the decoy opponents) each control **zero** Spirits, and p1 (the
     controller) controls **zero** Spirits. This distinguishes the declared target from
     (a) the controller, (b) `EachOpponent` (which would give all three opponents a token), and
     (c) a random opponent. This is the mandatory decoy compose test (AC 5034/5035).

- **`test_mana_doubler_when_tapped_for_mana_no_regression` (NEW)** — CR 605.4a / 106.12a. Put an
  inline empty-target mana-doubler (e.g. `Wild Growth` enchanting a Forest, or `Mirari's Wake`) on
  the battlefield, tap the affected source for mana, and assert the doubled/added mana appears in the
  pool exactly as before (immediate branch, no stack, no PendingTrigger of the reclassified kind).
  Proves the 6 non-target roster cards are unaffected by the kind change. Use `mana_pool()` to read
  totals; assert no `PendingTrigger` was queued for the doubler (or that it resolved immediately).

- **No-target fizzle (CR 603.3d)** — judged NOT worth a dedicated engine test: in any real
  multiplayer game at least one opponent always exists, and the `TargetOpponent` auto-picker already
  contributes `None` (trigger removed) when no opponent is available — covered by existing PB-EF6
  auto-picker tests (`pb_ef6_target_opponent.rs`). Note this reasoning in the plan; do not add a
  synthetic single-player fizzle test.

**Sentinel (wire-verdict guard)**: the existing sentinels already assert the frozen versions
(`pb_ef7_modal_activated.rs::test_ef7_hash_and_protocol_versions` → `PROTOCOL_VERSION == 18`,
`HASH_SCHEMA_VERSION == 55`; `pb_ef12_any_color_choice.rs::test_ef12_protocol_version_sentinel`).
The runner must keep the full suite green — these passing is the machine confirmation of "no bump".
No new sentinel is required (this PB adds no version-relevant type).

**Non-vacuity**: each new assertion (especially the p2-recipient and decoy-zero checks) must be
proven fail-before / pass-after by temporarily reverting Change 1 (or the `recipient` wiring) and
re-running, per the project's non-vacuous-test discipline.

**Pattern**: follow the existing `test_mana_trigger_forbidden_orchard` structure for state build and
`pass_all` priority-passing; follow `pb_ef6_target_opponent.rs` for opponent auto-target assertions
and `pb_ef2_create_token_recipient.rs` for token-recipient/owner assertions.

---

## Verification Checklist

- [ ] `mana.rs` change compiles (`cargo check`)
- [ ] `forbidden_orchard.rs` TODO/approximation removed; `recipient` wired; `Complete` (only if both
      halves compose in the 4-player compose test)
- [ ] `cargo build --workspace` clean (confirms no missing match arms — CardDefETB is pre-existing)
- [ ] Unit tests pass (`cargo test --all`), including the updated existing test + decoy compose +
      no-regression
- [ ] New assertions proven non-vacuous (revert-and-rerun)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35 — the script is the only one that
      checks the card def)
- [ ] Sentinels green: `PROTOCOL_VERSION == 18`, `HASH_SCHEMA_VERSION == 55` (no wire bump)
- [ ] Roster of all 7 WhenTappedForMana defs re-verified from `all_cards()`; 6 doublers confirmed
      immediate-branch and unregressed
- [ ] No remaining TODO in `forbidden_orchard.rs`

## Close-out (per brief)

- Close **OOS-EF6-1** in `memory/primitives/oos-retriage-plan-2026-07-18.md` §3 (SHIPPED banner +
  queue-summary table strike) and `memory/primitives/ef-batch-plan-2026-07-17.md` §10 (CLOSED
  banner).
- Update the `forbidden_orchard.rs` header comment to reflect the resolved state.
- Update `memory/primitive-wip.md` PB-OS3 status / step checkboxes.

---

## Risks & Edge Cases

- **Existing-test kind assertion** (`mana_triggers.rs:856-860`) is the one guaranteed breakage from
  Change 1 (it pins `PendingTriggerKind::Normal`). It MUST be updated to `CardDefETB` in the same
  commit — this is expected and load-bearing, not a regression.
- **Golden scripts**: no approved golden script exercises Forbidden Orchard's token recipient (only
  the direct-Command test does). The runner should still grep the script corpus for "Forbidden
  Orchard" / "Spirit" mana-trigger assertions before collect; expected result: none. (The `tokens/`
  and `stack/` swan_song scripts touched by PB-EF2 are unrelated — different card.)
- **Effect-execution equivalence**: the current `Normal` path already reaches the `CreateToken`
  effect via the registry *fallback* at `resolution.rs:1951-1977` (runtime vec empty → fall
  through). Option B routes it via the explicit CardDefETB branch (`:1978-2003`), which uses the same
  `def.abilities.get(ability_index)` lookup. Effect resolution is therefore unchanged; only target
  binding is fixed. (This is why the pre-fix test saw 1 token — mis-targeted to the controller —
  rather than 0.)
- **Auto-picker determinism**: `TargetOpponent` picks the first non-controller active player in
  `turn_order`, so the "declared" opponent is deterministic (p2 in a p1-active game). The decoy test
  relies on this ordering; assert `Target::Player(p2)` explicitly rather than "some opponent".
- **`intervening_if` / modes**: forbidden_orchard's trigger has `intervening_if: None`, `modes:
  None`; the CardDefETB resolution path handles both (`resolution.rs:1987-2003`, `:2011-2039`) — no
  interaction.
- **Multiplayer control-change**: token `recipient` resolves at effect execution against the stack
  object's bound target (`Target::Player(p2)`), captured at flush time — correct per CR 608.2b even
  if board state shifts between flush and resolution.
