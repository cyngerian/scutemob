# W-EMPTY engine findings (task scutemob-96)

Findings surfaced while authoring the W-EMPTY authorable set. Filed, not fixed
inline (W6 / campaign policy). This wave authored only 3 cards, so the surface is
small; the roster (`w-empty-roster-2026-07-17.md`) catalogs the 57 blocked cards'
gaps, most already tracked.

## EF-W-EMPTY-1 (MEDIUM) — `exclude_self` is unenforced in the sacrifice-**cost** path — ✅ CLOSED (PB-EF1, scutemob-99)

> **CLOSED 2026-07-18** via option (a): the source `ObjectId` is threaded into
> `eligible_sacrifice_targets` (fixes the `SacrificePermanents` effect path and the
> `MayPayThenEffect` optional-cost path at once), and the activated-ability cost path
> carries `ActivationCost.sacrifice_exclude_self` (lowered from `Cost::Sacrifice`'s
> `TargetFilter.exclude_self`) enforced in `handle_activate_ability`. `korvold` flipped
> Complete; `disciple_of_freyalise` stayed `partial` on a *distinct* surviving blocker
> (EF-EF1-A: `PowerOfSacrificedCreature` not captured in the optional-cost path — since
> **CLOSED 2026-07-19 by PB-OS2 (`scutemob-128`)**, `disciple_of_freyalise` now `Complete`).
> Regressions: `sacrifice_permanents_effect_excludes_source`,
> `optional_cost_sacrifice_excludes_source`, `izoni_*`, `korvold_etb_*`.


**Where**: `crates/engine/src/effects/mod.rs` — `eligible_sacrifice_targets`
(≈:7346) filters candidates through `matches_filter(chars: &Characteristics,
filter: &TargetFilter)` (≈:7941). `matches_filter` receives only `Characteristics`
and no `ObjectId`, so it cannot compare a candidate against the ability's
`ctx.source`. `TargetFilter::exclude_self` (a real field,
`card_definition.rs`) is therefore silently ignored for **every** sacrifice
performed through a cost — both a mandatory `Cost::Sacrifice(filter)` and the
`pay_optional_cost` path used by `Effect::MayPayThenEffect` / `MayPayOrElse`.

**Consequence**: any "sacrifice **another** creature/permanent" **cost** authored
with `exclude_self: true` would let the controller sacrifice the ability's own
source to pay for itself — wrong game state (CR 601/118 + the card's "another"
clause). This is why `disciple_of_freyalise.rs`'s front-face ETB is left `partial`
this wave (back face is Complete).

**Corroboration** (pre-existing, same root cause):
- `korvold_fae_cursed_king.rs` — `partial`, note already says
  `TargetFilter::exclude_self` is NOT honored by `Effect::SacrificePermanents`
  (the **effect** path — the sibling of this cost-path gap).
- `commissar_severina_raine.rs` — TODO "Sacrifice another creature — Cost::SacrificeOther not in DSL".
- `yawgmoth_thran_physician.rs` — TODO "Sacrifice another creature — Cost::SacrificeOtherCreature not in DSL".

So the gap spans **both** the effect path (`SacrificePermanents`) and the cost
path (`Cost::Sacrifice` via mandatory or optional payment); neither honors
`exclude_self`.

**Fix shape** (for a future PB, not this wave): either (a) thread the source
`ObjectId` into `eligible_sacrifice_targets` / `matches_filter` and honor
`exclude_self` there (fixes both paths at once, since both call
`eligible_sacrifice_targets`), or (b) a dedicated `Cost::SacrificeOther`
variant that excludes the source by construction (cost path only). (a) is
preferred — it closes korvold too.

**Verification**: confirmed by source read (2026-07-17), not inferred from the
note — `TargetFilter` struct has `exclude_self: bool`; `eligible_sacrifice_targets`
calls `matches_filter(&chars, tf)` with no ObjectId; korvold independently
documents the effect-path half.

## Non-findings (checked, no issue)

- **Fuse standalone target index** (`turn.rs`): the Burn half uses
  `DeclaredTarget { index: 1 }`, matching the Complete `wear_tear.rs` pattern
  exactly — the engine's Fuse dispatch handles standalone-vs-fused indexing. Not a
  finding.
- **`EffectAmount::Sum(HandSize, Fixed(1))`** (`sea_gate_restoration.rs`): resolved
  once by `Effect::DrawCards` before the draw loop (CR 608.2h) — no self-referential
  recount. Compiles and matches existing `Sum`/`HandSize` usage. Not a finding.
