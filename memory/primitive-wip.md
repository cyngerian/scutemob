# Primitive WIP: PB-EF8 — `Cost::ExileSelfFromHand` (activation from hand) (EF-W-PB2-8)

batch: PB-EF8
title: Add `Cost::ExileSelfFromHand` (+ `ActivationZone::Hand`) so "Exile this card from your hand: Add {mana}" mana abilities activate from HAND, exile the card as the cost (CR 400.7 new object), produce mana stacklessly (CR 605.1a — no target, could add mana → it IS a mana ability), and do NOT reset `players_passed` (CR 605.5). Mirrors `Cost::DiscardSelf` (Channel, CR 702.34) but exiles instead of discards and the effect is mana rather than stack-using.
task: scutemob-109
branch: feat/pb-ef8-costexileselffromhand-activation-from-hand-ef-w-pb2-8
started: 2026-07-18
phase: done  # 2026-07-18 — plan→impl→review→fix complete. Review: 0 HIGH / 0 MED / 1 LOW (elvish oracle_text "card"→"creature", fixed). 2 flips (simian_spirit_guide, elvish_spirit_guide → Complete). HASH 50→51, PROTOCOL 12→13. EF-W-PB2-8 closed. All gates green.

## Implement phase — DONE (2026-07-18, scutemob-109)

- [x] Step 1a — `Cost::ExileSelfFromHand` added to `crates/card-types/src/cards/card_definition.rs` (Cost enum, after `Exert`)
- [x] Step 1b — `ActivationZone::Hand` added (same file, after `Graveyard`)
- [x] Step 2a — `ManaAbility.exile_self_from_hand: bool` added (`crates/card-types/src/state/game_object.rs`)
- [x] Step 2b — `ActivationCost.exile_self_from_hand: bool` added (same file); also backfilled 3
      pre-existing non-`..Default::default()` `ActivationCost` struct literals in
      `card_definition.rs` (food/clue/blood token specs) that the plan did not enumerate
- [x] Step 3a/3b/3c/3d — `replay_harness.rs`: `ManaAbilityCost.exile_self_from_hand`,
      accepting match arm in `mana_ability_cost_components`, no-tap guard relaxed
      (`!acc.requires_tap && !acc.exile_self_from_hand`), `mana_ability_lowering` carries the
      flag onto `ManaAbility`, `flatten_cost_into` arm added
- [x] Step 4 — `effects/mod.rs`: `Cost::ExileSelfFromHand` added to both `can_pay_optional_cost`
      and `pay_optional_cost` exhaustive match unreachable-arm groups
- [x] Step 5a/5b/5c/5d/5e — `rules/mana.rs` `handle_tap_for_mana`: ability-fetch moved above
      zone/controller check; conditional from-hand-vs-battlefield legality branch; exile-cost
      payment step (7c) with `GameEvent::ObjectExiled`; tap-gated steps correctly skipped;
      priority/players_passed untouched (verified by test)
- [x] Step 6 — hashing: `Cost::ExileSelfFromHand => 13u8`, `ActivationZone::Hand => 1u8`,
      `ManaAbility.exile_self_from_hand`, `ActivationCost.exile_self_from_hand` all hashed
- [x] Step 7 — version bumps, machine-forced values read from failing gates:
      HASH_SCHEMA_VERSION 50→**51**, PROTOCOL_VERSION 12→**13**; decl/stream/protocol
      fingerprints + FROZEN_HISTORY_PREFIX_DIGEST (both hash and protocol) updated; all
      ~34 `HASH_SCHEMA_VERSION, 50` test sentinels bulk-updated to 51; `pb_ef7_modal_activated.rs`'s
      `PROTOCOL_VERSION` sentinel updated 12→13
- [x] Card defs — `simian_spirit_guide.rs` (inert→Complete), `elvish_spirit_guide.rs`
      (known_wrong→Complete); both use `Cost::ExileSelfFromHand` + `activation_zone: Some(Hand)`,
      completeness marker dropped (defaults Complete), no TODOs remain
- [x] Tests — `crates/engine/tests/primitives/pb_ef8_exile_self_from_hand.rs` (7 tests),
      registered via `mod pb_ef8_exile_self_from_hand;` in `tests/primitives/main.rs`; both
      decoys proven non-vacuous by temporarily deleting each check and confirming failure,
      then restoring (diffed byte-identical to pre-experiment)
- [x] Gates: `cargo build --workspace` clean; `cargo test --all` all green (0 failures);
      `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` clean (after
      `cargo fmt`); `tools/check-defs-fmt.sh` clean (1792 defs)

## Source findings
- memory/primitives/ef-batch-plan-2026-07-17.md — PB-EF8 section (line ~448); §1 table (EF-W-PB2-8, line 191)
- memory/card-authoring/w-pb2-roster-2026-07-17.md — EF-W-PB2-8 (line 85, 103, 134)

## Corpus sweep (DONE 2026-07-18 by worker — grep of oracle_text "Exile this card from your hand" + notes)
Candidates with an exile-self-from-hand **mana** ability:
| Card | current | ability | verdict |
| --- | --- | --- | --- |
| Simian Spirit Guide | `inert` | "Exile this card from your hand: Add {R}." | **ELIGIBLE → Complete** |
| Elvish Spirit Guide | `known_wrong` | "Exile this card from your hand: Add {G}." (currently ships a FREE battlefield `Add {G}` — infinite mana) | **ELIGIBLE → Complete** |

False positives (out of scope, verified):
- `saw_it_coming` — "exile this card from your hand" is the **Foretell** keyword (CR 702.143), not a mana ability. Already Complete via `AbilityDefinition::Foretell`.
- `chrome_mox` — Imprint ETB exile-from-hand (different primitive; still blocked, unrelated).
- `gemstone_caverns` — "if you do, exile a card from your hand" is a Luck-counter ETB payload; its `{T}: Add {C}` is a battlefield ability. Unrelated.

**Discounted ship: 2 flips** (Simian + Elvish Spirit Guide). Matches plan's ~2–3.

## Architecture recon (worker — verify + extend during plan)

The two cards are **mana abilities** (no target, produce mana → CR 605.1a). They therefore must
lower through `mana_ability_lowering` and activate through **`handle_tap_for_mana`** (mana.rs),
NOT `handle_activate_ability`. That is the ONLY stackless path (CR 605.3b / 605.5). Getting this
wrong (routing through the stack-using activated path) would hand opponents a priority window a
mana ability must never grant — exactly the SR-33 defect.

### Existing mirror to copy: `ActivationZone::Graveyard` (PB-35, Reassembling Skeleton)
- `ActivationZone` enum — `card_definition.rs:4156` — currently only `Graveyard`. **Add `Hand`.**
- The graveyard zone-legality branch is in `handle_activate_ability` (`abilities.rs:189-202`) —
  but that is the STACK path (Reassembling Skeleton's effect is not mana). Our cards are mana, so
  the analogous check must live in `handle_tap_for_mana`.

### Cost enum & runtime cost structs
- `Cost` enum — `card_definition.rs:1217`. Has `SacrificeSelf`, `ExileSelf` (battlefield exile,
  CR 118.12), `DiscardSelf` (Channel, hand-discard). **Add `Cost::ExileSelfFromHand`** — self-exile
  from HAND as an activation cost (the mana-ability analog of `DiscardSelf`).
- Runtime `ActivationCost` struct (stack-path cost) — `game_object.rs:~288-340`. Has `sacrifice_self`,
  `discard_self`, `exile_self`, `exert`, `life_cost`. **Add `exile_self_from_hand: bool`** (mirrors
  `discard_self`) so the flatten function `crates/engine/src/testing/replay_harness.rs:~3989`
  (`Cost::DiscardSelf => ac.discard_self = true`) stays exhaustive over `Cost`. (Our cards use the
  mana path, not this struct, but the `Cost` match must handle the new variant.)
- Runtime `ManaAbility` struct — `game_object.rs:172`. Has `requires_tap`, `sacrifice_self`,
  `mana_cost`, `life_cost`, `scaled_amount`, `activation_condition`. **Add
  `exile_self_from_hand: bool`** — the flag `handle_tap_for_mana` reads to switch to the
  hand-zone-exile payment path.

### Mana-ability lowering (`crates/engine/src/testing/replay_harness.rs`)
- `mana_ability_cost_components` (line 3711) — walks the `Cost`. It currently **rejects**
  `Cost::ExileSelf | ExileFromHand | DiscardSelf | ...` (line 3746-3748) because "TapForMana has no
  payload for them." **For `ExileSelfFromHand` this is FALSE** — the source IS `self`, no extra
  ObjectId payload is needed (identical to `SacrificeSelf`, which IS accepted). **Add a
  `Cost::ExileSelfFromHand => acc.exile_self_from_hand = true` arm.**
- **KEY HAZARD — the no-tap guard (line 3760):** `if !acc.requires_tap { return None; }`. A card in
  HAND cannot tap (CR 302 — only permanents on the battlefield tap). Simian/Elvish have **no `{T}`**
  in their cost. So this guard would reject them. **Relax it: a cost with `exile_self_from_hand`
  is lowerable without `requires_tap`** (the SR-34 "decline no-tap cost" rule guarded against free
  repeatable battlefield abilities — an exile-from-hand cost is inherently one-shot and self-consuming,
  so the seam that rule closed does not apply). Justify against CR 605.1a in the plan.
- `mana_ability_lowering` (line 3786) copies the components onto the `ManaAbility`. **Add
  `ma.exile_self_from_hand = components.exile_self_from_hand;`**.
- `try_as_tap_mana_ability` (line 3816) handles `Effect::AddMana` → `mana_pool_to_ability`. `Add {R}` /
  `Add {G}` are single-color → already handled. No change needed there (but verify the `requires_tap`
  it sets is overwritten by the components — it is, at line 3798).
- `enrich_spec_from_def` mana-ability loop (~2117-2155) — calls `mana_ability_lowering`; propagates
  the returned `ManaAbility` into `mana_abilities` and excludes it from `activated_abilities` (same
  predicate). Verify the exclusion still fires for our cards.

### `handle_tap_for_mana` (`crates/engine/src/rules/mana.rs:37`) — the core change
Current flow: step 3 rejects `obj.zone != Battlefield`; step 4 rejects `obj.controller != player`;
step 6 taps if `requires_tap`; step 7 pays `sacrifice_self` (moves source to graveyard, CR 400.7).
**New branch when `ability.exile_self_from_hand`:**
1. **Zone legality (replaces steps 3-4):** source must be in `ZoneId::Hand(owner)` and `obj.owner == player`
   (a hand card's `controller` is the owner; use `owner`, mirroring the Channel/graveyard checks in
   `handle_activate_ability` which check `obj.owner`). **Reject if on battlefield** — this is decoy A.
   (Decoy B — a battlefield-only mana ability with source in hand — is already rejected by the
   existing step-3 battlefield check, which stays the default for non-`exile_self_from_hand` abilities.)
2. **Skip tap** (step 6 is gated on `requires_tap`, which is false — already correct). Also the
   summoning-sickness / already-tapped checks live inside that gate, so they are correctly skipped.
3. **Pay the exile cost** (analogous to step 7 `sacrifice_self`): move source to `ZoneId::Exile`
   via `state.move_object_to_zone(source, ZoneId::Exile)` BEFORE producing mana. CR 400.7: the
   card becomes a new object in exile; `source` is a dead ObjectId afterward. Emit an appropriate
   event (check for an existing `GameEvent` exile/zone-change variant — mirror what `Cost::ExileSelf`
   emits in `handle_activate_ability`).
4. **Produce mana** (steps 8+) — unchanged; `requires_tap` is false so the mana-production
   replacement (step 7b, gated on `requires_tap`) and the WhenTappedForMana trigger (step 10,
   gated on `requires_tap`) are correctly **skipped** (CR 106.12 "tap for mana" — exiling from hand
   is not tapping, so Nyxbloom/Mana Reflection do NOT multiply it, and Forbidden-Orchard-style
   "whenever tapped for mana" triggers do NOT fire). Confirm this reading vs oracle/CR in the plan.
5. **Priority retained, `players_passed` unchanged** (step 11 — already the invariant of this fn).

### Hashing
- `Cost::ExileSelfFromHand` — add a hash arm wherever `Cost` variants are hashed (grep `Cost::ExileSelf`
  / `Cost::DiscardSelf` in `crates/engine/src/state/hash.rs`).
- `ActivationZone::Hand` — hash arm next to `ActivationZone::Graveyard => 0u8` at `hash.rs:5352`
  (use `1u8`).
- `ManaAbility.exile_self_from_hand` and `ActivationCost.exile_self_from_hand` — add to their
  `HashInto` impls.

### Wire bumps (verify — read digests from FAILING gate output, never hand-guess)
- **HASH_SCHEMA_VERSION**: the runtime `ManaAbility` / `ActivationCost` live inside `GameState`, so a
  new bool field reaches the hash closure → HASH bump likely machine-forced. Read current from
  `state/hash.rs`.
- **PROTOCOL_VERSION / PROTOCOL_SCHEMA_FINGERPRINT**: `Cost` and `ActivationZone` are in the card DSL,
  which is inside the SR-8 wire type closure (Characteristics → Effect → DSL). Adding a `Cost`
  variant + `ActivationZone` variant is a wire-shape change → PROTOCOL bump likely machine-forced.
  Read current from `rules/protocol.rs`. **Command::TapForMana is UNCHANGED** (no new payload — the
  source-in-hand is identified by ObjectId exactly as a battlefield source is), so the wire *frame*
  shape is unchanged; only the transitive DSL closure changes.

## COORDINATOR SCOPING DECISIONS (constraints for planner/runner)
1. **Scope = the 2 eligible flips** (Simian + Elvish Spirit Guide) + the recorded sweep. No other
   cards. Do not widen to Chrome Mox / Gemstone Caverns / Foretell.
2. **Path = mana-ability lowering → `handle_tap_for_mana` (stackless).** This is non-negotiable
   (acceptance criterion 1, CR 605.1a/605.3b/605.5). Do NOT route these through the stack-using
   `handle_activate_ability`.
3. **Single source of truth for "this is a hand-exile cost" = `Cost::ExileSelfFromHand`** (mirrors how
   `Cost::DiscardSelf` alone drives Channel's hand activation — no redundant `activation_zone` needed
   to *drive* behavior). **Still add `ActivationZone::Hand`** to the enum (task asks for the concept;
   keeps the enum symmetric and future-proofs a non-mana hand ability) and have the two defs carry
   `activation_zone: Some(ActivationZone::Hand)` to document intent — but the engine's mana-lowering
   and legality key off the **cost**, so there is no state to keep in sync. Planner: confirm this is
   the cleanest design or propose better with CR justification.
4. **Decoys must fail on exactly the zone check, both directions** (criterion 1): (A) same card on
   battlefield cannot use the from-hand ability; (B) card in hand cannot activate a battlefield-only
   mana ability. Plus CR 400.7 object-identity assertion across the exile.
5. PROTOCOL/HASH bumps only if a gate machine-forces them; read the new digest from the failing gate,
   justify each in the commit + here.
