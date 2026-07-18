# W-MISS engine findings (EF-W-MISS-*)

**Task**: `scutemob-97` — W-MISS (campaign plan §3). **Date**: 2026-07-17.
Surfaced while triaging the 194 missing-file cards (`w-miss-roster-2026-07-17.md`).
None fixed inline (W-MISS is an authoring wave; these are engine gaps/bugs for a future
PB/SR). Most restate already-tracked gaps; **EF-W-MISS-1 is a NEW latent bug in a shipped
`Complete` def** and is the one worth a coordinator decision.

## EF-W-MISS-1 (HIGH — latent legal-but-wrong in a Complete def): `swan_song.rs` token recipient — ✅ CLOSED (PB-EF2, scutemob-102)
> **CLOSED 2026-07-18.** `TokenSpec` gained `recipient: PlayerTarget` (default `Controller`,
> all 201 existing `CreateToken`/`CreateTokenAndAttachSource` sites unchanged) and
> `PlayerTarget` gained `ControllerOfCounteredSpell` (captured into new
> `EffectContext::countered_spell_controller` by `Effect::CounterSpell` before the
> `cant_be_countered` check, per the An Offer ruling 2022-04-29) + `ControllerOfTriggeringObject`.
> The `CreateToken` executor now loops over `resolve_player_target_list(state,
> &spec.recipient, ctx)`, applying token-creation replacements (Doubling Season, etc.)
> per-recipient. `swan_song.rs` flipped back to `Complete`; new card
> `an_offer_you_cant_refuse.rs` shipped `Complete`. Regression:
> `crates/engine/tests/primitives/pb_ef2_create_token_recipient.rs` (8 tests). HASH 44→45,
> PROTOCOL 6→7.

`Effect::CreateToken` creates the token for `ctx.controller` (the caster). Swan Song
("Counter target enchantment, instant, or sorcery spell. **Its controller** creates a 2/2 blue
Bird…") ships **Complete** but hands the Bird to the Swan Song caster, not to the countered
spell's controller. Proven by shape, not run — flag for verification. Same defect blocks
authoring **An Offer You Can't Refuse** ("its controller creates two Treasures"). Root gap: no
player-scoped recipient on `CreateToken` (recipient = the controller of a referenced object).
Fix shape: add a `recipient: PlayerTarget` (default Controller) to `Effect::CreateToken`, and a
`PlayerTarget::ControllerOfCounteredSpell` / `…OfTriggeringObject`. Marker note on swan_song
should be demoted from Complete until fixed, OR fixed in an SR. **Coordinator call.**

## EF-W-MISS-2 (MEDIUM): `Effect::UntapAll` ignores `TargetFilter.exclude_self` — ✅ CLOSED (PB-EF1, scutemob-99)
> **CLOSED 2026-07-18.** The `UntapAll` executor now applies `(!filter.exclude_self ||
> **id != ctx.source)`. New card `copperhorn_scout.rs` shipped Complete. Regression:
> `copperhorn_untaps_others_but_not_itself`.

"Untap each **other** creature you control" (Copperhorn Scout) is inexpressible — `UntapAll`
untaps every filter match including the source. Affects any "each other" untap. Fix: honour
`exclude_self` in the `UntapAll` executor.

## EF-W-MISS-3 (MEDIUM): granted keyword-triggers are silent no-ops
`LayerModification::AddKeyword` inserts into `keywords` but the derived triggered ability
(Melee, Battle Cry, Annihilator) is synthesized only from **printed** keywords in `builder.rs`.
So an anthem granting Melee/Battle Cry to *other* creatures registers the keyword but the
trigger never fires. Blocks Adriana, Skyhunter Strike Force (Lieutenant). Static keywords
(flying/haste) grant fine; only trigger-bearing keywords are affected. Fix: synthesize the
keyword-derived triggered ability when a keyword is added by a continuous effect.

## EF-W-MISS-4 (MEDIUM): no "defending player" target for attack triggers (Hellrider gap) — ✅ CLOSED by PB-EF3 (`scutemob-103`, 2026-07-18)
> Added `EffectTarget::AttackTarget` (player or planeswalker the attacker is attacking) and
> `PlayerTarget::DefendingPlayer` (defending player only, CR 508.4). Defending player captured
> per-attacker at `AttackersDeclared` into `PendingTrigger.defending_player_id`, threaded to
> `StackObject`/`EffectContext`; substituting EachOpponent/Controller correctly avoided. Shipped
> `hellrider.rs` (flip) + `raid_bombardment.rs` (new). Silumgar's continuous-effect variant filed
> as OOS-EF3-1 (needs a locked `EffectFilter::CreaturesControlledBy`); Brutal Hordechief / Norn's
> Decree / Karazikar / Cunning Rhetoric stay blocked on other, distinct primitives.

No `PlayerTarget`/`EffectTarget` resolves to the specific player (or planeswalker) the
triggering attacker is attacking. Substituting `EachOpponent`/`Controller` is wrong in
4-player Commander. Keeps `hellrider.rs` partial; blocks Brutal Hordechief, Raid Bombardment,
Norn's Decree, Karazikar, Silumgar (defending-player creature filter), Cunning Rhetoric.

## EF-W-MISS-5 (MEDIUM): `EffectFilter::TriggeringCreature` does not exist
Continuous "it gets +N/+N EOT" / "it gains <keyword> EOT" on the just-attacked (or
just-triggered) creature cannot be expressed via `ApplyContinuousEffect` — no filter selects
the triggering object. Keeps `ogre_battledriver.rs` partial; blocks Atarka, Fervent Charge,
Goblin Piledriver, Muxus.

## EF-W-MISS-6 (LOW — large but known cohort): no card-invokable self-transform effect
The Effect enum has only `Meld`; there is no `Effect::Transform`/`TransformSelf`. A card cannot
cause itself (or another named permanent) to transform from a triggered/activated/conditional
ability — `KeywordAbility::Transform`'s behaviour is carried only by the external
`Command::Transform`. Blocks the **entire body-only bucket** (11 DFCs). Also needed: `CardType::Battle`
(Invasion of Ikoria) and the "Super Nova" keyword (Sephiroth). Documented in
`thaumatic_compass.rs`, `delver_of_secrets.rs`. A high-yield future PB.

## EF-W-MISS-7 (LOW): sacrifice-driven `EffectAmount` / `max_cmc` gaps
- No `EffectAmount::ToughnessOfSacrificedCreature` (only `PowerOfSacrificedCreature`) — Momentous Fall.
- No runtime-computed `max_cmc` (`N + sacrificed creature's MV`) on `SearchLibrary`; `max_cmc`
  is a fixed `Option<u32>` — Birthing Ritual, Eldritch Evolution.
- No `Condition` reporting whether a resolution-time `SacrificePermanents` fired ("if you do")
  — Victimize.

## EF-W-MISS-8 (LOW): `WheelDraw` lacks a "greatest number discarded" variant
`WheelDraw` has only `ThatMany` (own hand size) and `Fixed(n)`. Windfall draws "equal to the
greatest number of cards any player discarded this way" — not representable. Blocks Windfall
(Wheel of Fortune / Tolarian Winds / Fateful Showdown, which use `ThatMany`/`Fixed`, are fine).

## EF-W-MISS-9 (LOW): no spells-only single-target restriction
Misdirection needs "target spell **with a single target**". The only single-target
`TargetRequirement` (`TargetSpellOrAbilityWithSingleTarget`) also permits abilities, and
`TargetFilter` has no target-count field to narrow `TargetSpellWithFilter`. So Misdirection
cannot be authored Complete without over-permissive cast legality. Fix: a spell-only
single-target `TargetRequirement`, or a target-count predicate on the spell filter.

## EF-W-MISS-10 (HIGH): targeted `WheneverCreatureYouControlAttacks` drops its target — ✅ CLOSED by PB-EF3 (`scutemob-103`, 2026-07-18)
> `enrich_spec_from_def` now forwards each card-def `AbilityDefinition::Triggered { targets }`
> into the runtime `TriggeredAbilityDef.targets` (all 30 enrich blocks; was hardcoded `vec![]`),
> and the auto-target fallback is kind-guarded: `Normal` treats the runtime `triggered_abilities`
> vec as authoritative, `CardDefETB` keeps the `def.abilities` raw-index. Fixed 4 pre-existing
> sites mis-tagged `Normal` while raw-indexing `def.abilities` (incl. the Throat Slitter path).
> Shipped `ojutai_soul_of_winter.rs` (the card removed unshipped in W-MISS). Note: "Dragonlord
> Ojutai" was a mis-listed candidate (combat-damage trigger, no target — not this finding).

The *targeted* variant of the attack trigger has never worked. `enrich_spec_from_def`
converts `WheneverCreatureYouControlAttacks` to a runtime `TriggeredAbilityDef` with
**hardcoded `targets: vec![]`** (`crates/engine/src/testing/replay_harness.rs:3011`),
discarding the DSL's `targets` (`TargetPermanentWithFilter`, etc.). The registry fallback
(`abilities.rs:6699-6713`) then indexes `def.abilities` by `trigger.ability_index`, but
that index is into the runtime `triggered_abilities` vec, not `def.abilities` — so it
matches the wrong ability and returns no targets. Net: the trigger goes on the stack with
no target and any `TapPermanent`/`PreventNextUntap`/etc. resolves against an empty list —
wrong game state, not merely omitted text. Every shipped user of this trigger passes
`targets: vec![]` (kolaghan, dromoka, utvara, kazuul), so the path was never exercised.
**Blocked: Ojutai, Soul of Winter** ("tap target creature or artifact an opponent
controls…") — authored, reviewed, then **removed** this wave (not shipped wrong). Fix
(a PB/SR, not an authoring wave): forward the DSL `targets` into the runtime trigger def in
the enrich block, and fix the fallback to match the Triggered ability rather than raw-index
`def.abilities`.

## Note (not a finding): report name-normalization
`authoring-report.py` lists **Steelshaper's Gift** and **Dwynen's Elite** as missing though
`steelshaper_s_gift.rs` / `dwynen_s_elite.rs` exist with correct names — an apostrophe
plan-matching quirk. Cosmetic (2 cards under-counted as authored); out of W-MISS scope but
worth a one-line fix in the report tool later.
</content>
