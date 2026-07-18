# W-PB2 engine findings (scutemob-95) — 2026-07-17

Findings surfaced while authoring the W-PB2 wave. Per the task guardrail ("don't
implement engine changes in this wave — mark and file"), these are **filed, not fixed**.
Each names a card that stays non-`Complete` because of it.

## EF-W-PB2-1 — `EffectAmount::PermanentCount` ignores `exclude_self` (MEDIUM) — ✅ CLOSED (PB-EF1, scutemob-99)

> **CLOSED 2026-07-18.** The `PermanentCount` resolver now applies `(!filter.exclude_self
> || obj.id != ctx.source)` (mirroring the sibling `AttackingCreatureCount` /
> `TappedCreatureCount` resolvers). `eomer_king_of_rohan.rs` flipped `known_wrong` →
> Complete. Regression: `permanent_count_excludes_the_entering_source`.


`effects/mod.rs:6749` — the `PermanentCount` resolver closure filters on zone / phased-in /
controller / `matches_filter` / chosen-subtype / counter-type, but **never applies
`filter.exclude_self`**. Its sibling amount resolvers do: `AttackingCreatureCount`
(`effects/mod.rs:7032`) and `TappedCreatureCount` (`:7066`) both guard `obj.id != ctx.source`.

**Instance**: `eomer_king_of_rohan.rs`. Oracle: "Éomer enters with a +1/+1 counter on it for
each **other** Human you control." The def is correct — `EntersWithCounters` with
`count = PermanentCount{ has_subtype: Human, controller: You, exclude_self: true }` — but the
self-ETB replacement resolves the count with `ctx.source = Éomer` **after** Éomer is already on
the battlefield (moved at `resolution.rs:576`, replacement runs at `:1646`). Éomer is a Human
you control, so it counts itself: a 2/2 with no other Humans enters as a 3/3.

**Éomer is the ONLY def using `PermanentCount + exclude_self: true`** (grep-verified over the
corpus), so the fix is zero-risk to existing cards and changes no existing test's hash.

**Fix (one line)**: add `&& (!filter.exclude_self || obj.id != ctx.source)` to the
`PermanentCount` closure, mirroring `:7032`/`:7066`. Then flip `eomer_king_of_rohan.rs` to
`Complete` and add a regression test asserting the entered counter count with 0 and with N
other Humans. Demoted to `known_wrong` here.

## EF-W-PB2-2 — no opponent-restricted player `TargetRequirement` (MEDIUM) — ✅ CLOSED (PB-EF6, scutemob-107, 2026-07-18)

> **CLOSED 2026-07-18.** Added `TargetRequirement::TargetOpponent` (unit variant, hash
> discriminant 18). Validation threads the source's controller (`caster`) into
> `validate_player_satisfies_requirement`, which now rejects a self-target (`Ok` iff `id !=
> caster`; no teams model exists — opponent = any non-controller, CR 102.2/102.3/601.2c). Both
> trigger auto-target pickers (outer + UpToN-inner) pick the first active opponent with **no
> self-fallback** — the trigger is removed from the stack if the source has no opponent (CR
> 603.3d). Wire bump **PROTOCOL 10→11, HASH 48→49** (machine-forced). **3 clean flips → Complete**
> (shaman_of_the_pack, raiders_wake, and vengeful_bloodwitch — the last found by roster recall,
> not in the brief) + a latent legal-but-wrong self-target on the shipped-Complete `fell_specter`
> corrected. blood_tribute / blessed_alliance(idx3) / forbidden_orchard / ajani_sleeper_agent
> target-fixed but stay non-Complete on their REAL surviving blockers (HalfLife / Escalate /
> AddManaAnyColor+WhenTappedForMana-dispatch / no-op loyalty). Review: 0 HIGH, 0 MEDIUM, 3 LOW
> (all fixed). New seed filed: **OOS-EF6-1** (WhenTappedForMana triggers queue as
> `PendingTriggerKind::Normal` with a raw def-index the auto-picker can't read — forbidden_orchard's
> token target/recipient is dead until fixed). Plan/review: `memory/primitives/pb-plan-EF6.md` /
> `pb-review-EF6.md`.

`TargetRequirement` has `TargetPlayer` (any player) but **no `TargetOpponent`** variant, so
"target opponent …" oracle text cannot be authored without permitting an illegal self-target
(KI-1). Confirmed by author against the DSL and 3 sibling defs already stuck on this exact gap:
`raiders_wake.rs`, `forbidden_orchard.rs`, `ajani_sleeper_agent.rs`.

**Instance**: `shaman_of_the_pack.rs`. The ETB *amount* is now expressible
(`PermanentCount{ has_subtype: Elf, controller: You }`), but "target opponent loses life" is not.
Stays `partial`. Also unblocks the 3 sibling defs above.

**Fix**: add `TargetRequirement::TargetOpponent` (+ its validation arm restricting candidates to
opponents of the source's controller, CR 115.x). A PB-sized task, out of scope for a
card-authoring wave.

## EF-W-PB2-3 — granted `any_color` ManaAbility stubs to `Colorless` (MEDIUM)

`rules/mana.rs` `handle_tap_for_mana` (L337–365) — a `ManaAbility { any_color: true }` (whether
intrinsic or granted via `LayerModification::AddManaAbility`) unconditionally adds
`ManaColor::Colorless`, with the comment *"Simplified: colorless until interactive color choice is
implemented, consistent with `Effect::AddManaAnyColor`"*. This is the **same defect class** as the
gated `Effect::AddManaAnyColor` family (SR-37), but on the granted-mana-ability path rather than
the Effect path — and it is **not** caught by `effect_choose_gate` (which walks the Effect serde
tree, not `ManaAbility` structs).

**Instance**: `elven_chorus.rs` — clause "Creatures you control have '{T}: Add one mana of any
color'". Wiring `AddManaAbility{ any_color: true }` would make every creature tap for **colorless**,
not any color — wrong game state. Stays `partial`.

**Correction to the marker-sweep worklist**: it claimed `enduring_vitality.rs` implements this
clause and is `Complete`, valid precedent. **False** — `enduring_vitality.rs` is currently
`partial` (grep-verified), so it never proved the grant against this stub. Any future "add any
color via granted ability" card is blocked here too.

**Fix**: implement interactive/deterministic color choice for `any_color` mana abilities (the same
work `Effect::AddManaAnyColor` needs), then the `tainted_field` one-ability-per-color pattern or a
real choice channel. PB-sized, out of scope for an authoring wave.

## EF-W-PB2-4 — no modal-activated-ability primitive (MEDIUM) — ✅ CLOSED (PB-EF7, scutemob-108, 2026-07-18)

> **CLOSED 2026-07-18.** Added `modes: Option<ModeSelection>` to
> `AbilityDefinition::Activated` (+ the runtime `ActivatedAbility` struct), reusing
> `ModeSelection.mode_targets` (PB-AC4) for per-mode targets, and
> `Command::ActivateAbility.modes_chosen` for the mode announcement. Approach (a): the
> chosen mode's effect is baked into the stack object's `embedded_effect` **at activation
> time** (not resolution) — required because both eligible cards pay `Cost::SacrificeSelf`,
> so the source `ObjectId` is dead by resolution (CR 400.7);
> `resolution.rs`'s `ActivatedAbility` arm is UNCHANGED. `goblin_cratermaker.rs` and
> `cankerbloom.rs` both flipped `known_wrong` → `Complete` (the former also picked up its
> `exclude_colors` colorless-filter fix — a pure def change, `exclude_colors` was already
> honored). `umezawas_jitte.rs` stays `known_wrong`: its modal-activated ability could now
> be modeled correctly, but a second, distinct blocker survives — see **OOS-EF7-1** below.
> `HASH_SCHEMA_VERSION` 49→50, `PROTOCOL_VERSION` 11→12. Tests:
> `crates/engine/tests/primitives/pb_ef7_modal_activated.rs` (11 tests, incl. forward/reverse
> mode-choice decoys, invalid-mode-index/nonmodal rejection, an LKI-persistence test, and
> the `exclude_colors` / CR 700.2c empty-target-slice decoys).

`AbilityDefinition::Activated` (`card_definition.rs:285`) has **no `modes` field** — only
`Triggered` and `Spell` carry `modes: Option<ModeSelection>`. So a "Choose one —" on an
**activated** ability can only be modeled with the gated `Effect::Choose` stub (always executes
`choices.first()`), which is barred from Complete.

**Instance**: `goblin_cratermaker.rs` — "{1}, Sacrifice: Choose one — deal 2 damage to target
creature; or destroy target colorless nonland permanent." Stays `known_wrong` (silently always
resolves mode 0 while still demanding an unused mode-1 target). The secondary `exclude_colors`
filter defect on mode 2 is moot until the modal primitive exists.

**Fix**: add `modes: Option<ModeSelection>` (+ `mode_targets`) to `AbilityDefinition::Activated`
and wire announce/validate/resolution, mirroring the `Spell`/`Triggered` modal path. PB-sized.

### OOS-EF7-1 — Umezawa's Jitte still blocked after PB-EF7 (its trigger, not its modal ability)

The modal-activated primitive now exists and would correctly resolve Jitte's "Remove a
charge counter: Choose one —" ability (equipped creature +2/+2 / target creature -1/-1 /
you gain 2 life) if rewritten onto `AbilityDefinition::Activated::modes`. That is **not**
the surviving blocker: Jitte's charge-counter trigger
(`AbilityDefinition::Triggered { trigger_condition:
WhenEquippedCreatureDealsCombatDamageToPlayer, .. }`) only fires on combat damage dealt **to
a player**, but the oracle says "Whenever equipped creature deals combat damage" (any
recipient — e.g. a blocking creature). Needs a new `TriggerCondition` variant (something like
`WhenEquippedCreatureDealsCombatDamage`, distinct from the existing `…ToPlayer` variant) before
this card can be `Complete`. Out of scope for PB-EF7 (scoped to the 2 eligible flips only,
per the coordinator's scoping decision). `umezawas_jitte.rs`'s marker note was rewritten to
cite this as the real surviving blocker.

## EF-W-PB2-5 — no "while you control source" `EffectDuration` (MEDIUM) — ✅ CLOSED (PB-EF9, scutemob-110, 2026-07-18)
> `EffectDuration::WhileYouControlSource(PlayerId)` added (CR 611.2b/c). "You" fixed to
> `ctx.controller` at creation (PlayerId(0) placeholder, mirroring UntilYourNextTurn). Termination
> is a one-shot permanent removal (`expire_while_you_control_source_effects`, run per-iteration in
> `check_and_apply_sbas`) so it NEVER resumes; control reverts via `recompute_object_controller`
> (reapplies remaining SetController effects in timestamp order). Phased-out source stays controlled
> (CR 702.26e — `is_phased_in()` deliberately not checked). **Discovery: no control-reversion existed
> in the engine at all** — WhileSourceOnBattlefield/UntilEndOfTurn gain-control never reverted either;
> this PB builds it. olivia_voldaren + dragonlord_silumgar flipped → Complete; roil_elemental stays
> partial (optional "you may" wrapper inexpressible); kellogg stays partial (sacrifice-N cost).
> OOS-EF9-1 filed for the latent WhileSourceOnBattlefield/UntilEndOfTurn never-reverts gap.
> PROTOCOL 13→14, HASH 51→52.


`continuous_effect.rs` L44–64 — `EffectDuration` has `WhileSourceOnBattlefield` but no variant for
"for as long as you control [source]". The two differ under gain-control.

**Instance**: `olivia_voldaren.rs` — the `{3}{B}{B}` gain-control ability says "for as long as you
control Olivia Voldaren"; modeled with `WhileSourceOnBattlefield`, so a borrowed creature would not
return if an opponent gains control of Olivia while she remains on the battlefield. Demoted from
Complete to `partial` (the `{1}{R}` half is correct).

**Fix**: add `EffectDuration::WhileYouControlSource` (or similar) + its continuous-effect
expiry check. PB-sized.

## EF-W-PB2-6 — no `EffectFilter::TriggeringCreature` (MEDIUM) — ✅ CLOSED (PB-EF4, scutemob-105, 2026-07-18)
> `EffectFilter::TriggeringCreature` added (resolves to `SingleObject(ctx.triggering_creature_id)`
> at `ApplyContinuousEffect`). dragon_tempest / ogre_battledriver flipped to Complete;
> atarka_world_render / fervent_charge / dreadhorde_invasion authored/flipped Complete using it.
> shared_animosity stayed `inert` (surviving count-`EffectAmount` gap → OOS-EF4-1). PROTOCOL 8→9,
> HASH 46→47.


`continuous_effect.rs:67` — `EffectFilter` (the filter on `ContinuousEffectDef`) has
`Source`/`DeclaredTarget`/`CreaturesYouControl`/… but **no `TriggeringCreature`**. Only
`EffectTarget` has `TriggeringCreature` — usable for point effects (`AddCounter`, `DealDamage`) but
NOT for continuous-effect filters. So "when a creature enters, **it** gains \<keyword\> until end of
turn" (grant a continuous effect to the entering creature) is inexpressible.

**Instances**: `dragon_tempest.rs` (flying-creature ETB → "it gains haste"), and the already-marked
`ogre_battledriver.rs` / `shared_animosity.rs` carry the identical documented gap.

**Fix**: add `EffectFilter::TriggeringCreature` + its resolution (read the trigger's
`triggering_creature_id` from ctx). PB-sized.

## EF-W-PB2-7 — `Effect::DealDamage` has no source-override (MEDIUM) — ✅ CLOSED (PB-EF4, scutemob-105, 2026-07-18)
> `Effect::DealDamage.source: Option<EffectTarget>` added (`#[serde(default)]`; `None` = existing
> `ctx.source`; `Some(t)` threads a single `damage_source_id` through all 12 attribution reads).
> dragon_tempest (Dragon half), scourge_of_valkas, warstorm_surge flipped Complete with
> `source: Some(EffectTarget::TriggeringCreature)`. terror_of_the_peaks deliberately kept
> `source: None`. PROTOCOL 8→9, HASH 46→47.


`card_definition.rs:1330` — `DealDamage { target, amount }` always sources from `ctx.source`. So
"when another permanent enters, **it** deals damage" (the entering permanent as the damage source,
not the ability's source) is inexpressible.

**Instances**: `dragon_tempest.rs` (Dragon ETB → "it deals X damage"; Dragon Tempest is never itself
a Dragon, so it misattributes on 100% of firings — left `inert`), and `scourge_of_valkas.rs` (the
"or another Dragon enters" half; the self-ETB half IS authored because there `ctx.source` = Scourge
= "it" — left `partial`).

**Fix**: add an optional `source: Option<EffectTarget>` to `DealDamage` (defaulting to `ctx.source`),
resolvable to `TriggeringCreature`. PB-sized. Wire change (Effect shape) → PROTOCOL bump.

## EF-W-PB2-8 — no "exile this card from hand" activation cost (MEDIUM)

`card_definition.rs:1247` — the only exile-from-hand `Cost` is `Cost::ExileFromHand { color }`,
which is the **Force of Will-style pitch cost** (exile a card *of a chosen color* from hand to
help pay for a *different* spell, recorded as `AdditionalCost::ExileFromHand { card }`,
casting.rs:4168). There is **no** cost meaning "exile THIS card from your hand" as a
self-activation cost (the nearest, `Cost::DiscardSelf`, discards rather than exiles).

**Instance**: `simian_spirit_guide.rs` — "Exile Simian Spirit Guide from your hand: Add {R}."
Cannot be authored Complete. **Note**: the def previously shipped `Cost::Mana(ManaCost::default())`
→ a **free, untapped, repeatable "Add {R}" from the battlefield** (invented infinite red mana) — a
live wrong-state bug now removed (`abilities: vec![]`, `partial` marker names the gap).

**Fix**: add `Cost::ExileSelfFromHand` (+ `activation_zone: Hand` support) mirroring
`Cost::DiscardSelf`. PB-sized.

## LOW / accepted (non-blocking, cards ship Complete)

- **avenger_of_zendikar** — landfall "you may put a +1/+1 counter" modeled as mandatory (the
  effect is always beneficial; matches the khalni/roster convention for non-interactive "may").
- **access_denied** — `ManaValueOf` sums the countered spell's printed cost and omits a chosen
  `X`, so countering an `{X}` spell undercounts the Thopter tokens. Narrow, pre-existing
  `ManaValueOf` limitation (the blessed primitive for this card per the marker sweep); accepted
  as Complete for the non-X case.
