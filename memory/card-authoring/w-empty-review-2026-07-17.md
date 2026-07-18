# W-EMPTY Card Review — scutemob-96 (2026-07-17)

**Reviewer**: card-batch-reviewer (Opus)
**Cards**: 3 (Turn // Burn, Sea Gate Restoration // Sea Gate Reborn, Disciple of Freyalise // Garden of Freyalise)
**Findings**: 0 HIGH, 0 MEDIUM, 2 LOW
**Legal-but-wrong game state**: none found. All Complete defs verified faithful.

---

## Card 1: Turn // Burn (`turn.rs`) — marked Complete

- **Oracle match**: PARTIAL (see F1)
- **Types match**: YES — `Instant` (front). Split-card second half carried by `AbilityDefinition::Fuse`, mirroring the Complete `wear_tear.rs`.
- **Mana cost match**: YES — front `{2}{U}` = `generic 2, blue 1`; Burn Fuse half `{1}{R}` = `generic 1, red 1`. Matches Scryfall `{2}{U} // {1}{R}`.
- **DSL correctness**: YES
- **Findings**:
  - **F1 (LOW)**: `oracle_text` omits the Burn half's text. It contains only the Turn half
    ("Until end of turn, target creature loses all abilities and becomes a red Weird with
    base power and toughness 0/1.") plus the Fuse reminder. The Complete split-card sibling
    `wear_tear.rs` concatenates BOTH halves ("Wear — ...\nTear — ...\nFuse (...)"). Turn // Burn
    should likewise include "Burn deals 2 damage to any target." Reference-only field (behavior
    is driven by the two `AbilityDefinition`s, not by `oracle_text`), so no game-state impact —
    hence LOW — but a Complete card should carry its full printed text.

### Turn layer verification (the core correctness claim)
All four layer mods verified correct and faithful to CR + the 2013-04-15 rulings:
- **Layer 4 (`TypeChange`)** — `SetCreatureTypes({Weird})`: replaces creature subtypes with exactly Weird. Correct.
- **Layer 5 (`ColorChange`)** — `SetColors({Red})`: becomes mono-red. Correct.
- **Layer 6 (`Ability`)** — `RemoveAllAbilities`: "loses all abilities." Correct.
- **Layer 7b (`PtSet`)** — `SetPowerToughness {0,1}`: "base power and toughness 0/1." Correct.
  Applied via `Effect::ApplyContinuousEffect` (resolution-time, UntilEndOfTurn), which is the
  correct route — the `SetPowerToughness` AUTHORING NOTE that warns against `AbilityDefinition::Static`
  does NOT apply here (this is a one-shot duration effect, not a CDA/static on a permanent).
- **No `SetCardTypes`** — verified CORRECT against the 2013-04-15 ruling: "Turn will cause the
  creature lose all other colors and creature types, but it will retain any other card types
  (such as artifact)." Remains a creature (no card-type change); only subtypes/colors/abilities/PT change.
- All four use `EffectFilter::DeclaredTarget { index: 0 }` + `EffectDuration::UntilEndOfTurn`. Sequence
  order is documentation-only (engine sorts by layer). Correct.
- **Burn half**: `DealDamage` to `DeclaredTarget { index: 1 }`, amount `Fixed(2)`, `TargetAny`.
  **No lifegain** — verified against oracle ("Burn deals 2 damage to any target."). Index-1 target
  (right half follows left, CR 702.102d) matches the Complete `wear_tear.rs` Tear-at-index-1 pattern exactly.
- **Fuse marker** `AbilityDefinition::Keyword(KeywordAbility::Fuse)` present. Correct.
- Variant names all exist (`EffectLayer::{TypeChange,ColorChange,Ability,PtSet}`,
  `LayerModification::{SetCreatureTypes,SetColors,RemoveAllAbilities,SetPowerToughness}`) — no compile risk.

**Verdict: Complete is justified.** One LOW oracle-text omission.

---

## Card 2: Sea Gate Restoration // Sea Gate, Reborn (`sea_gate_restoration.rs`) — marked Complete

- **Oracle match**: YES (front exact; back plausibly-current, see note)
- **Types match**: YES — front `Sorcery`; back `Land`. Matches `Sorcery // Land`.
- **Mana cost match**: YES — `{4}{U}{U}{U}` = `generic 4, blue 3`.
- **DSL correctness**: YES
- **Findings**: none (LOW-info note below).

### Front verification
- `DrawCards { count: Sum(HandSize{Controller}, Fixed(1)) }`. **CR 608.2h**: `resolve_amount` is
  evaluated once before the draw loop, so the count is locked at resolution (hand of N → draw N+1);
  this correctly avoids the self-referential "draw then recount" trap. Correct.
- `SetNoMaximumHandSize { Controller }` — "no maximum hand size for the rest of the game" (CR 402.2).
  Real, non-stub effect (AC9 lineage). Correct.
- No targets (`targets: vec![]`). Correct — the spell affects only its controller.

### Back verification (Sea Gate, Reborn)
- `EntersTappedUnlessPayLife(3)` self-replacement, `WouldEnterBattlefield { Any }`, `is_self: true`.
  Matches oracle "As this land enters, you may pay 3 life. If you don't, it enters tapped." Correct
  (PB-2/PB-3 supported; not a stub).
- `{T}: Add {U}` — `mana_pool(0, 1, 0, 0, 0, 0)`. **Blue arg position confirmed correct**: arg order
  is (white, blue, black, red, green, colorless); blue at index 2, cross-checked against the green
  lands `mana_pool(0,0,0,0,1,0)`. Correct.
- **Note (LOW/info, not a defect)**: back-face `oracle_text` uses "As this land enters ..." rather than
  spelling the face name. This matches current (post-2023) Scryfall self-reference templating, so it is
  plausibly exact; I could not byte-verify the per-face text (MCP `lookup_card` returns only combined
  type/color for split/MDFC cards, and `card_faces` in `cards.sqlite` is not queryable with the
  read-only toolset available here). No behavioral impact regardless.

**Verdict: Complete is justified.** No findings.

---

## Card 3: Disciple of Freyalise // Garden of Freyalise (`disciple_of_freyalise.rs`) — marked `partial`

- **Oracle match**: YES (front exact; back plausibly-current per same MDFC templating note)
- **Types match**: YES — front `Creature — Elf Druid` (`creature_types(["Elf","Druid"])`); back `Land`.
- **Mana cost match**: YES — `{3}{G}{G}{G}` = `generic 3, green 3`.
- **P/T**: `Some(3)/Some(3)` — correct (fixed 3/3, not a `*/*` CDA, so `Some` is right).
- **DSL correctness**: YES (back face); front deliberately unimplemented.
- **Findings**: none. Disposition and note both verified sound.

### (a) Is `partial` the correct call? — YES, confirmed.
The front-face ETB ("you may sacrifice **another** creature; gain X life, draw X, X = its power")
genuinely cannot be shipped Complete today. Verified against engine source:
- `TargetFilter.exclude_self` exists (`card_definition.rs:4134`) but is **silently ignored** by the
  sacrifice-cost path.
- `matches_filter(chars: &Characteristics, filter: &TargetFilter)` (`effects/mod.rs:7941`) takes **no
  ObjectId** — structurally cannot compare a candidate against the ability's source.
- `eligible_sacrifice_targets(state, pid, filter)` (`effects/mod.rs:7346`) takes **no source ObjectId**
  and its body (read in full, 7346–7400) never references `exclude_self`, even though the candidate
  `id` is in scope — the missing piece is the *source* to exclude, not the candidate.
- Consequence: an implementation via `Cost::Sacrifice(TargetFilter{exclude_self:true})` (mandatory OR
  `pay_optional_cost`/`MayPayThenEffect`) or via `Effect::SacrificePermanents` (the Korvold route)
  would let the controller sacrifice **Disciple itself** to pay its own trigger — contradicting
  "another creature." That is wrong game state (CR 109.1 / 603.2 "another"), not a missing clause →
  **W5 policy correctly forbids shipping it.**
- The engine *does* have the pattern `!f.exclude_self || obj.id != ctx.source` — but only in
  `resolve_amount` counting arms (`effects/mod.rs:7032`, `7066`: AttackingCreatureCount /
  TappedCreatureCount), which carry `ctx.source`. It is **not** wired into the sacrifice selection
  path, confirming the gap rather than contradicting it.
- The OTHER potential blocker (X = sacrificed creature's power) is NOT a blocker:
  `EffectAmount::PowerOfSacrificedCreature` (PB-P) exists and is fed from
  `EffectContext.sacrificed_creature_powers`. So `exclude_self` is the sole, real blocker — the note
  does not over- or under-claim.

There is **no clean filter workaround** (Disciple shares subtypes/colors with plausible sacrifice
fodder; you cannot express "not this object" without `exclude_self`). Shipping Complete is not possible today.

### (b) Back face (Garden of Freyalise) fully/correctly implemented? — YES.
- `EntersTappedUnlessPayLife(3)` self-replacement (`WouldEnterBattlefield { Any }`, `is_self: true`) —
  matches "As this land enters, you may pay 3 life. If you don't, it enters tapped." Correct.
- `{T}: Add {G}` — `mana_pool(0,0,0,0,1,0)` (green at index 5). Correct.
- Identical, correct structure to the Complete `revitalizing_repast.rs` back face (modulo the
  pay-life-vs-plain-tapped replacement, which matches this card's oracle).

### (c) Is the `partial` note truthful? — YES.
Every mechanical claim in the note was verified against source: the `card_definition.rs:4134` field,
`effects/mod.rs:7346` and `:7941` signatures/bodies, and the cross-references to
`korvold_fae_cursed_king.rs` (confirmed: its note reads "`Effect::SacrificePermanents` exists but
`TargetFilter::exclude_self` is NOT honored") and `commissar_severina_raine.rs`. Line numbers cited in
the note are exact. The note correctly frames the omission as wrong-game-state avoidance, not a lazy TODO.

**Verdict: `partial` is the correct, honest disposition. Back face is Complete-quality; front is
genuinely unshippable today.**

---

## Gated-stub sweep (Complete defs only)
Confirmed **no** `Effect::Choose`, `Effect::MayPayOrElse`, `Effect::AddManaChoice`, or the
`AddManaAnyColor` family in any Complete def:
- `turn.rs` (Complete): `ApplyContinuousEffect`, `DealDamage` only.
- `sea_gate_restoration.rs` (Complete): `DrawCards`, `SetNoMaximumHandSize`, concrete `AddMana`
  (`mana_pool` blue), `EntersTappedUnlessPayLife`. All concrete.
- `disciple_of_freyalise.rs` is `partial`, not Complete; its back face uses only concrete `AddMana`
  + `EntersTappedUnlessPayLife`. `MayPayThenEffect` appears **only in the front-face TODO comment**,
  never in executable code, and is not used by any Complete card here.

## Summary
- **Clean / Complete-justified**: Turn // Burn (1 LOW oracle-text omission), Sea Gate Restoration.
- **Correctly `partial`**: Disciple of Freyalise — verdict upheld; note truthful; back face correct.
- **No HIGH, no MEDIUM, no legal-but-wrong game state.**
- **Only actionable item**: F1 (LOW) — add the Burn half's oracle text to `turn.rs`'s `oracle_text`.
