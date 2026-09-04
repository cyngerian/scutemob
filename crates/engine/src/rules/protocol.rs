//! Wire protocol versioning for `Command` and `GameEvent` streams.
//!
//! `Command` is the only way into the engine and `GameEvent` is the only way out
//! (invariants #3 and #4), so the two enums *are* the wire protocol for M10's
//! centralized server, and they are what a replay log is made of. This module
//! puts a version tag on those serialized streams and defines what happens when
//! it does not match.
//!
//! # Policy: strict lockstep
//!
//! A message declares `protocol_version`. A receiver accepts it **iff** the
//! declared version equals [`PROTOCOL_VERSION`] exactly. Anything else is
//! rejected with [`ProtocolError::VersionMismatch`]. There is no negotiation, no
//! forward compatibility, and no best-effort decoding of an unknown version.
//!
//! The reason is invariant #9. Rewind, replay, and pause all rest on a complete
//! and accurate state history from turn 1. A client that silently drops an event
//! variant it does not understand, or that fills a missing field with a default,
//! holds a history that cannot be correctly rewound — and it holds it *without
//! knowing*. Refusing the connection is recoverable; a corrupted history is not.
//!
//! # The version number is machine-checked, not remembered
//!
//! A hand-bumped constant next to a growing enum is precisely the kind of
//! process guarantee the SR track exists to convert into a machine guarantee: it
//! is correct exactly as long as every future author remembers it. So
//! [`PROTOCOL_SCHEMA_FINGERPRINT`] pins a digest of the **transitive type
//! closure** of the three wire frames — `Command`, `GameEvent`, [`ReplayLog`] —
//! computed from workspace source by `tests/protocol_schema.rs`. Change the
//! shape of anything on the wire and that test fails, names the drift, and tells
//! you to bump [`PROTOCOL_VERSION`].
//!
//! The closure is 90 types, not 3. `GameEvent::CreatureDied` carries
//! `Option<Characteristics>`, which reaches `AbilityInstance` → `Effect` →
//! `TargetFilter` → the whole card DSL. **Adding an `Effect` variant is a wire
//! change**, so most primitive batches (PB-*) will bump this version. That is
//! not gate noise; it is what strict lockstep means.
//!
//! It bottoms out cleanly: `GameState` is *not* in the closure. Whole-state sync
//! is a different question, guarded by `HASH_SCHEMA_VERSION`
//! (`state::hash`), and a replay log carries both — see [`ReplayLog`].
//!
//! Full rationale, bump procedure, and known holes:
//! `docs/mtg-engine-protocol-versioning.md`.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::rules::command::Command;
use crate::state::hash::HASH_SCHEMA_VERSION;

/// The wire protocol version spoken by this build.
///
/// Bump this whenever the serialized shape of `Command`, `GameEvent`, or any
/// type reachable from them changes. `tests/protocol_schema.rs` fails until you
/// do, and tells you which types moved.
///
/// # History
/// - 1: SR-8 (2026-07-10) — initial versioned envelope. Baseline shape is the
///   90-type closure recorded in [`PROTOCOL_SCHEMA_FINGERPRINT`].
/// - 2: SR-10 (2026-07-10) — `Command::CastSpell`'s ~16-field payload boxed into
///   a new [`crate::rules::command::CastSpellData`] struct (clippy::large_enum_variant;
///   shrinks every `Command` value and replay-log entry). The serialized **bytes**
///   are unchanged — a boxed newtype variant wrapping a struct is serde-identical to
///   the former struct variant — but the shape digest moved because the closure grew
///   by one type (90 → 91) and the variant's declared form changed. Bumped per this
///   gate's policy that any non-variant-reorder digest move bumps the version.
/// - 3: SR-34 (2026-07-17) — `ManaAbility` (reachable from `Command`/`GameEvent` via
///   `Characteristics.mana_abilities: Vec<ManaAbility>`, a [`CLOSURE_MUST_CONTAIN`]
///   entry) gains `mana_cost: Option<ManaCost>` and `life_cost: u32`, its activation
///   cost's mana and life components (CR 605.1a — a mana ability is classified by
///   what it does, not what it costs; `handle_tap_for_mana` now pays these). The
///   closure stays 91 types (no new type joins it, `ManaCost` was already in the
///   closure), but `ManaAbility`'s declared shape changed, so the digest moves.
/// - 4: SR-36 (2026-07-17) — SF-8/SF-9: `ManaAbility` gains
///   `scaled_amount: Option<Box<EffectAmount>>` (a dynamic mana amount, CR 605.1a) and
///   `ActivationCost` (reachable via `Characteristics.activated_abilities: Vec<ActivatedAbility>`
///   → `ActivatedAbility.cost: ActivationCost`) gains `life_cost: u32` (CR 118.3/119.4
///   — a non-mana activated ability's life-payment component). `EffectAmount` was
///   already in the closure (via `Effect`), so the closure's type count is unchanged;
///   both structs' declared shapes moved, so the digest moves.
/// - 5: SR-37 (2026-07-17) — SF-10: `ManaAbility` gains
///   `activation_condition: Option<Condition>` (an "activate only if ..." restriction,
///   CR 605.1a + CR 602.5b — Tainted Field's coloured arms). `Condition` was already in
///   the closure (reachable via `Effect::Conditional`), so the closure's type count is
///   unchanged; `ManaAbility`'s declared shape moved, so the digest moves.
/// - 6: PB-EF1 (2026-07-18) — `ActivationCost` (reachable via
///   `Characteristics.activated_abilities` → `ActivatedAbility.cost`) gains
///   `sacrifice_exclude_self: bool` (CR 109.1 — the "Sacrifice ANOTHER [permanent]"
///   restriction on an activated ability's sacrifice cost; `SacrificeFilter` carries no
///   ObjectId so the bit rides on `ActivationCost`). The closure's type count is
///   unchanged; `ActivationCost`'s declared shape moved, so the digest moves.
/// - 7: PB-EF2 (2026-07-18) — `TokenSpec` (reachable via `Effect::CreateToken`/
///   `Effect::CreateTokenAndAttachSource`) gains `recipient: PlayerTarget` (CR 111.1 /
///   CR 608.2h — which player creates the token(s); "its controller creates …" cards
///   like Swan Song). `PlayerTarget` (already in the closure) gains two variants,
///   `ControllerOfCounteredSpell` and `ControllerOfTriggeringObject`. The closure's
///   type count is unchanged; both types' declared shapes moved, so the digest moves.
/// - 8: PB-EF3 (2026-07-18) — `EffectTarget` (reachable via `Effect::DealDamage.target`
///   and other `Effect` variants) gains `AttackTarget`; `PlayerTarget` (already in the
///   closure) gains `DefendingPlayer` (CR 508.4 — the defending player / attack target
///   of an attacking creature, EF-W-MISS-4/EF-W-MISS-10). The closure's type count is
///   unchanged; both types' declared shapes moved, so the digest moves.
/// - 9: PB-EF4 (2026-07-18) — `Effect::DealDamage` (reachable via `Effect` and thus in
///   the closure already) gains `source: Option<EffectTarget>` (CR 119.3 / 702.15a — an
///   optional damage-source override, e.g. "the entering creature deals it", resolved
///   to a single ObjectId at execution time; `EffectTarget` was already in the closure).
///   The closure's type count is unchanged; `Effect`'s declared shape moved, so the
///   digest moves. (`EffectFilter` also gained a `TriggeringCreature` variant in this
///   PB, but `EffectFilter` is off the wire closure — it lives inside `GameState`'s
///   `continuous_effects`, not `Command`/`GameEvent` — so that half is a HASH_SCHEMA_VERSION
///   bump only, not a PROTOCOL_VERSION one.)
/// - 10: PB-EF5 (2026-07-18) — `Effect` (already in the closure) gains a new unit
///   variant `TransformSelf` (CR 701.27a/f, 712.18 — flip the resolving ability's own
///   source DFC in place; used by an on-card triggered/activated/conditional effect,
///   distinct from the existing `Command::Transform`). The closure's type count is
///   unchanged; `Effect`'s declared shape moved, so the digest moves.
/// - 11: PB-EF6 (2026-07-18) — `TargetRequirement` (reachable via
///   `AbilityDefinition.targets` / `Effect`) gains a new unit variant `TargetOpponent`
///   (CR 102.3/102.4/115.1/601.2c/603.3d — "target opponent", an opponent-restricted
///   player target; EF-W-PB2-2). The closure's type count is unchanged;
///   `TargetRequirement`'s declared shape moved, so the digest moves.
/// - 12: PB-EF7 (2026-07-18) — `Command::ActivateAbility` (a wire frame) gains
///   `modes_chosen: Vec<usize>`, and `AbilityDefinition::Activated` (reachable via
///   `Characteristics.activated_abilities` → `ActivatedAbility` → the DSL closure)
///   gains `modes: Option<ModeSelection>` (CR 700.2a/601.2b — modal activated
///   abilities; EF-W-PB2-4). `ModeSelection` was already in the closure (via
///   `AbilityDefinition::Spell`/`Triggered`). The closure's type count is unchanged;
///   both `Command` and `AbilityDefinition`'s declared shapes moved, so the digest moves.
/// - 13: PB-EF8 (2026-07-18) — `Cost` (reachable via `AbilityDefinition::Activated.cost`)
///   gains a new unit variant `ExileSelfFromHand` (CR 118 + CR 400.7 + CR 605.1a — a
///   from-hand mana ability's exile-self activation cost, e.g. Simian/Elvish Spirit
///   Guide), and `ActivationZone` (reachable via `AbilityDefinition::Activated.activation_zone`)
///   gains a new unit variant `Hand` (CR 602.2 — decorative marker; the mana-lowering
///   path keys off `Cost::ExileSelfFromHand` alone, not this field). The closure's type
///   count is unchanged; both `Cost` and `ActivationZone`'s declared shapes moved, so the
///   digest moves.
/// - 14: PB-EF9 (2026-07-18) — `EffectDuration` (reachable via `Effect::GainControl` /
///   `Effect::ApplyContinuousEffect(ContinuousEffectDef)` → the card DSL closure) gains
///   a new variant `WhileYouControlSource(PlayerId)` (CR 611.2b/c — "for as long as you
///   control [source]", a continuous-effect duration for gain-control effects; Olivia
///   Voldaren, Dragonlord Silumgar). The closure's type count is unchanged;
///   `EffectDuration`'s declared shape moved, so the digest moves.
/// - 15: PB-EF10 (2026-07-18) — `AdditionalCost::Sacrifice` (reachable via
///   `CastSpell.additional_costs` → the wire closure) changes its `lki_powers: Vec<i32>`
///   field to `lki: Vec<SacrificedCreatureLki>` (CR 608.2b/608.2h/608.2i — the sacrificed
///   creature's LKI now carries power/toughness/mana value atomically, not just power;
///   EF-W-MISS-7). `TargetFilter` (reachable via `Effect`/`AbilityDefinition` →
///   the closure) gains `max_cmc_amount: Option<Box<EffectAmount>>` (CR 202.3/608.2h —
///   a runtime-computed search cap). The closure's type count is unchanged; both types'
///   declared shapes moved, so the digest moves.
/// - 16: PB-EF11 COMMIT 1 (2026-07-18) — `WheelDraw` (reachable via
///   `Effect::WheelHand` → the wire closure) gains a new unit variant
///   `GreatestDiscarded` (CR 121.1 — a wheel-draw count equal to the greatest number
///   of cards any affected player disposed of this way; unblocks Windfall). The
///   closure's type count is unchanged; `WheelDraw`'s declared shape moved, so the
///   digest moves.
/// - 17: PB-EF11 COMMIT 2 (2026-07-18) — `TargetRequirement` (reachable via
///   `AbilityDefinition.targets` / `Effect` → the wire closure) gains a new unit
///   variant `TargetSpellWithSingleTarget` (CR 115.7a/115.7b — a spell-ONLY
///   single-target restriction, stricter than `TargetSpellOrAbilityWithSingleTarget`;
///   unblocks Misdirection). The closure's type count is unchanged;
///   `TargetRequirement`'s declared shape moved, so the digest moves.
/// - 18: PB-EF12 (2026-07-18) — `Command::TapForMana` (a wire frame) gains
///   `chosen_color: Option<ManaColor>` (CR 605.3b/106.1b — a mana ability resolves
///   immediately and never uses the stack, so the colour choice for an `any_color`
///   ability's production is made on the activation command itself, not deferred;
///   closes EF-W-PB2-3, the last item on the EF queue). `ManaColor` was already in
///   the closure. The closure's type count is unchanged; `Command`'s declared shape
///   moved, so the digest moves.
/// - 19: PB-OS4 (2026-07-19, SHIP NARROWED) — `Effect` (already in the closure)
///   gains one new unit variant, `ExileSourceAndReturnTransformed` (CR 400.7 /
///   712.18 — a permanent that leaves and returns to the battlefield already
///   transformed is a NEW object, unlike `TransformSelf`'s in-place flip;
///   OOS-EF5-3; used by Fable of the Mirror Breaker's Saga chapter III). The
///   closure's type count is unchanged; `Effect`'s declared shape moved, so the
///   digest moves.
/// - 20: PB-OS5 (2026-07-19) — `EffectAmount` (already in the closure) gains
///   `OtherAttackersSharingCreatureType { relative_to: EffectTarget }` (CR
///   205.3m/508.1 — count of other attacking creatures sharing a creature type
///   with the triggering creature; OOS-EF4-1, Shared Animosity). Closure type
///   count unchanged; `EffectAmount`'s declared shape moved, so the digest moves.
/// - 21: PB-OS6 (2026-07-19) — four closure-shape moves in one batch (DFC
///   flip-condition sub-batch, OOS-EF5-4 a/b/g): `Condition` (already in the
///   closure via `Effect::Conditional`) gains two new unit/tuple variants,
///   `TopCardIsInstantOrSorcery` (CR 400.2/614.1c — delver_of_secrets upkeep
///   flip) and `YouAttackedWithNOrMore(u32)` (CR 508.1/508.4 — legions_landing
///   attack-count gate); `Effect` (already in the closure) gains a new variant
///   `RemoveFromCombat { target: EffectTarget }` (CR 506.4 — thaumatic_compass /
///   Spires of Orazca); `GameEvent` (a wire frame) gains a new variant
///   `RemovedFromCombat { object_id: ObjectId }`. The closure's type count is
///   unchanged (no new type joins it); all three types' declared shapes moved,
///   so the digest moves. (`PlayerState.attackers_declared_this_turn`, the
///   fourth new field in this batch, is inside `GameState`, not the wire
///   closure — HASH_SCHEMA_VERSION bump only, see `state::hash`.)
/// - 22: PB-OS7 (2026-07-19, OOS-EF3-1) — `EffectFilter` (reachable via
///   `Effect::ApplyContinuousEffect(ContinuousEffectDef)` → the card DSL
///   closure — the SAME `ContinuousEffectDef` struct whose sibling field
///   `duration: EffectDuration` already put `EffectDuration` in the closure
///   at v14/PB-EF9) gains a new unit variant `CreaturesControlledByDefendingPlayer`
///   (CR 508.4/611.2a — DSL placeholder substituted into
///   `CreaturesControlledBy(ctx.defending_player)` at execution time; Silumgar,
///   the Drifting Death). **Correction to the PB-EF4 (v9) note above**: that note
///   claimed `EffectFilter` was "off the wire closure" — true at PB-EF4 time, but
///   PB-EF9 (v14) put `EffectFilter`'s sibling field `EffectDuration` in the same
///   `ContinuousEffectDef` struct into the closure, which transitively pulled
///   `EffectFilter` in too (the whole struct is reachable once one field is
///   scanned, per how `ContinuousEffectDef` is parsed). The PB-OS7 plan assumed
///   the stale v9 claim still held and predicted no bump; the machine gate
///   disagreed — this closure's type count is unchanged (no new type joins;
///   `ContinuousEffectDef`/`EffectFilter` were already reachable), but
///   `EffectFilter`'s declared shape moved, so the digest moves.
/// - 23: PB-OS8 (2026-07-19, OOS-EF10-1 + min_cmc_amount rider): `Effect` gains a new
///   variant `LookAtTopThenPlace` (CR 120/601.2/118.12/202.3/400.7 — look at the top N
///   cards, optionally pay an interposed cost, place at most one matching card,
///   rest to bottom; Birthing Ritual, Growing Rites of Itlimoc) and `TargetFilter`
///   gains a new field `min_cmc_amount` (runtime lower-bound mana-value cap, mirror
///   of the existing `max_cmc_amount`). Both `Effect` and `TargetFilter` are already
///   in the closure — type COUNT unchanged, declared shape moves, digest moves.
/// - 24: PB-OS9 (2026-07-19, OOS-EF3b-1): `Condition` gains a new unit variant
///   `YouControlYourCommander` (CR 903.3d — Lieutenant ability word: "if/as long as
///   you control your commander"; `skyhunter_strike_force`, `loyal_apprentice`,
///   `siege_gang_lieutenant`). `Condition` is already in the closure (reachable via
///   `Effect::Conditional`) — type COUNT unchanged, declared shape moves, digest moves.
/// - 25: PB-OS10 (2026-07-19, OOS-XS-1 + OOS-EF7-1): `TargetRequirement` gains a new
///   variant `TargetPermanentDistinctFrom(usize)` (CR 601.2c "another target permanent"
///   inter-target distinctness; `hidden_strings`). `TargetRequirement` is already in
///   the closure (reachable via `AbilityDefinition::Spell.targets` etc.) — type COUNT
///   unchanged, declared shape moves, digest moves. (`TriggerEvent`/`TriggerCondition`
///   also gained a paired variant for OOS-EF7-1's any-recipient equipped-creature
///   combat-damage trigger, `umezawas_jitte`, but neither is in the wire closure —
///   that half of this batch is a HASH-only change, see `state::hash`.) **Correction
///   (PB-DX1, 2026-08-01, §7.4): the "neither is in the wire closure" claim is FALSE
///   for `TriggerEvent`.** `TriggeredAbilityDef.trigger_on: TriggerEvent`, and
///   `TriggeredAbilityDef` is reachable via `Characteristics.triggered_abilities:
///   Vec<TriggeredAbilityDef>` — `Characteristics` is a [`CLOSURE_MUST_CONTAIN`]
///   entry and was already in the closure at v25 time, independent of anything
///   PB-DX1 touched. `TriggerEvent` was therefore ALREADY in the wire closure when
///   this note was written; a probe against the live scanner (2026-08-01) confirms
///   `TriggerEvent` present / `TriggerCondition` absent. That OS10 sub-change should
///   have moved this digest and evidently did not get credited for it (the v25 bump
///   from `TargetRequirement` alone may have masked it, or the note was simply never
///   checked against the scanner). `TriggerCondition` remains correctly excluded —
///   it lives on the card-def `AbilityDefinition::Triggered`, not on the runtime
///   `TriggeredAbilityDef`, and is not reachable from `Command`/`GameEvent`.
/// - 26: PB-OS11 (2026-07-19, final PB-OS batch — OOS-LKI-3 reframed): `ManaAbility`
///   (reachable via `Characteristics.mana_abilities: Vec<ManaAbility>`, a
///   [`CLOSURE_MUST_CONTAIN`] entry) gains `remove_counter: Option<(CounterType, u32)>`
///   (CR 605.1a / CR 602.2c — a self-referential remove-counter mana-ability
///   activation cost; Workhorse "Remove a +1/+1 counter: Add {C}", plus backfill
///   Gemstone Array / Druids' Repository). `CounterType` was already in the closure
///   (reachable via `Effect::AddCounter`/`RemoveCounter`), so the closure's type
///   count is unchanged; `ManaAbility`'s declared shape moved, so the digest moves —
///   exactly the SR-34/36/37 precedent (`ManaAbility` field additions bump the
///   protocol digest even though the closure's type count does not grow). This
///   batch's other half — `TriggerCondition::WheneverYouAttack` unit -> struct with
///   `filter: Option<TargetFilter>` (CR 508.1/508.1m — Anim Pakal, General Kreat,
///   Hermes) — does NOT move this digest: `TriggerCondition` is not in the wire
///   closure (correct); **`TriggerEvent` IS** (see the `- 25:` correction above),
///   but this sub-change touched only `TriggerCondition` (the card-def type), not
///   `TriggerEvent` (the runtime type it lowers to), so the "HASH-only" conclusion
///   for THIS specific sub-change still holds even though its stated reason was
///   half wrong.
/// - 27: PB-RS2 (2026-07-20, OOS-RS-2): `Command::ActivateAbility` and
///   `Command::TapForMana` (both wire frames) each gain two fields —
///   `hybrid_choices: Vec<HybridManaPayment>` and
///   `phyrexian_life_payments: Vec<bool>` (CR 107.4e/107.4f via CR 602.2b/605.1a
///   — an activated ability's activation cost, and a mana ability's activation
///   cost, are the analogs of a spell's mana cost and must be able to express a
///   hybrid/Phyrexian payment choice the same way `CastSpellData` already does;
///   fixes the free-pip bug on all 7 shipped filter lands and any activated
///   ability with a hybrid/Phyrexian pip). `HybridManaPayment` was already in the
///   closure (via `CastSpellData::hybrid_choices`). The closure's type count is
///   unchanged; `Command`'s declared shape moved (twice, in the same commit), so
///   the digest moves.
/// - 28: PB-DP7 (2026-07-26, DP-3 — the cleanup discard becomes a player choice,
///   the engine's first pending decision that genuinely blocks progress, CR
///   514.1): two new wire-frame types append -- `Command::DiscardToHandSize {
///   player, cards: Vec<ObjectId> }` and `GameEvent::CleanupDiscardChoiceRequired
///   { player, count: u32, hand: Vec<ObjectId> }`. Both field types (`PlayerId`,
///   `Vec<ObjectId>`, `u32`) are already in the closure, so the closure's type
///   count is unchanged; `Command`'s and `GameEvent`'s declared shapes moved, so
///   the digest moves.
/// - 29: PB-DP8 (2026-07-26, DP-6 — triggered-ability targets become a player
///   choice, CR 603.3d/601.2c): two new wire-frame variants append --
///   `Command::ChooseTriggerTargets { player, choice_id: u64, targets:
///   Vec<Vec<Target>> }` and `GameEvent::TriggerTargetChoiceRequired { player,
///   choice_id: u64, source_object_id, ability_index: usize, slots:
///   Vec<TriggerTargetOption> }`. Unlike v28, the closure's **type count changes**:
///   `TriggerTargetOption` is new, and through it `SpellTarget` (which `command.rs`
///   never referenced -- it carried bare `Target`) enters the closure for the first
///   time. Both declared shapes moved, so the digest moves.
/// - 30: PB-DP8 fix cycle (2026-07-26, review Findings 2+6, CR 601.2c): the wire
///   type `TriggerTargetOption` gains `max: u32`, the slot's declared width. The
///   shipped v29 shape dropped `TargetRequirement::UpToN`'s `count` and enforced a
///   hard `<= 1`, so Elder Deep-Fiend ("tap up to **four** target permanents") and
///   Cloud of Faeries ("untap up to **two** lands") -- both `Complete` -- could
///   still announce at most one target, and an under-filled slot shifted every
///   later slot's `EffectTarget::DeclaredTarget { index }` down by one. The
///   closure's type count is unchanged (94); `TriggerTargetOption`'s declared shape
///   moved, so the digest moves.
/// - 31: PB-DP9 (2026-07-27, DP-7/DP-8/DP-9 — library search, scry and surveil
///   become player choices, CR 608.2d / 701.23a / 701.22a / 701.25a): two new
///   wire-frame variants append -- `Command::AnswerEffectChoice { player,
///   choice_id: u64, answer: EffectChoiceAnswer }` and
///   `GameEvent::EffectChoiceRequired { player, choice_id: u64,
///   source_object_id, question: EffectChoiceQuestion }`. **One** command for
///   all three effects, because CR 608.2d is one rule and 701.22a/701.23a/701.25a
///   are three instances of it with identical timing, actor and validity
///   condition. The closure's **type count changes**: `EffectChoiceQuestion` and
///   `EffectChoiceAnswer` are both new and both reachable from both frames. Both
///   declared shapes moved, so the digest moves. (`GameEvent::private_to()` and
///   `reveals_hidden_info()` also land in this commit; they are METHODS, not
///   declared shapes, and do not touch the digest.)
/// - 32: PB-DX1 (2026-08-01, OOS-DP6-1 — CR 603.4, the intervening-if dropped in the
///   runtime lowering): `InterveningIf` (reachable via
///   `Characteristics.triggered_abilities: Vec<TriggeredAbilityDef>` ->
///   `TriggeredAbilityDef.intervening_if: Option<InterveningIf>`; `Characteristics`
///   is a [`CLOSURE_MUST_CONTAIN`] entry) gains a new variant
///   `CardDef(Box<Condition>)`, carrying a card-definition intervening-if through
///   `build_face_ability_vectors`' lowering (previously hardcoded `None` at all 34
///   push sites — Aurelia, the Warleader granted herself unbounded extra combats on
///   a `Complete`, deck-legal def). **This is a `HASH`-only-predicted change that
///   turned out to also be `PROTOCOL`**: the audit row and dispatch brief both
///   predicted HASH only; `InterveningIf` was NOT previously known to be reachable
///   from the wire closure. `Condition` was already in the closure (reachable via
///   `Effect::Conditional`), so the closure's type COUNT is unchanged (96);
///   `InterveningIf`'s declared shape moved, so the digest moves. **See also the
///   `- 25:`/`- 26:` corrections above**: while verifying this prediction, a probe
///   against the live scanner found `TriggerEvent` (unlike `TriggerCondition`) was
///   ALREADY in the wire closure at v25 time, independent of this batch — those two
///   History notes are corrected in place, not by a new row (they are doc comments,
///   not `PROTOCOL_HISTORY` entries; the append-only rule covers the table, not the
///   prose).
/// - 33: PB-DX6 (2026-08-02, OOS-RS2-1 + OOS-DP4-1 — the last two unflattened
///   mana-cost payment sites): two `Command` variants change declared shape.
///   `Command::TurnFaceUp` gains hybrid/Phyrexian payment fields so
///   `TurnFaceUpMethod::ManaCost` can pay a hybrid (`{G/W}`) or Phyrexian
///   (`{G/P}`) pip instead of treating it as free (CR 107.4e/107.4f via CR
///   701.40b — turning a creature face up is not casting a spell, but the mana
///   cost is still paid under the same pip rules); `Command::DeclareAttackers`
///   gains the same fields so the CR 508.1h "costs to attack" tax can be paid
///   when it is denominated in hybrid or Phyrexian mana. Both reuse
///   `HybridManaPayment` and `bool`, already reachable via
///   `CastSpellData`/`ActivateAbility`, so the closure's **type count is
///   unchanged** — exact precedent: `- 27: PB-RS2`, the same two fields landing
///   on two other variants. Both declared shapes moved, so the digest moves.
/// - 34: ENG-1 (2026-08-02, effect-driven discard becomes a real player choice,
///   CR 701.9b): `EffectChoiceQuestion` and `EffectChoiceAnswer` — both already in
///   the closure since v31 — each gain a fourth variant, `Discard { hand:
///   Vec<ObjectId>, count: u32 }` / `Discard { chosen: Vec<ObjectId> }`. Both
///   field types (`Vec<ObjectId>`, `u32`) are already in the closure, so the
///   closure's type count is unchanged (96); `EffectChoiceQuestion`'s and
///   `EffectChoiceAnswer`'s declared shapes moved, so the digest moves.
/// - 35: ENG-2 (2026-08-02, OOS-G7-1 — an announcement-time target event, CR
///   601.2c/602.2b/603.3d): `GameEvent` (a wire frame) gains a new variant,
///   `TargetsAnnounced` (discriminant 132), reachable fields already in the
///   closure. The closure's type count is unchanged (96); `GameEvent`'s
///   declared shape moved, so the digest moves.
/// - 36: PB-DX27 rider (2026-08-13, `OOS-ADJ-7`): `LayerModification` (reachable
///   via `Effect::ApplyContinuousEffect(ContinuousEffectDef)` -> the card DSL
///   closure -- the SAME `ContinuousEffectDef` struct whose sibling fields
///   `duration: EffectDuration` and `filter: EffectFilter` already put those two
///   types in the closure at v14/v22; "the whole struct is reachable once one
///   field is scanned", per the v22 correction above) gains a new variant,
///   `SetLandTypes(OrdSet<SubType>)` (arm tag `32u8`). **This is a `HASH`-only-
///   predicted change that turned out to also be `PROTOCOL`** -- the same
///   mistake the v32 note names: `LayerModification` was assumed to be off the
///   wire closure because it is a runtime/layer-system type, not a card-DSL
///   type, but `ContinuousEffectDef.modification: LayerModification` is a
///   sibling field of `filter`/`duration` on the SAME struct the v22 correction
///   already established is reachable in full. The closure's type count is
///   unchanged (96); `LayerModification`'s declared shape moved, so the digest
///   moves.
/// - 37: PB-DX28 (2026-08-14, `OOS-DX4-6` + `OOS-DX4-1` — CR 115.10's untargeted
///   resolution-time choice, and CR 108.3 ownership as an axis distinct from CR
///   109.4 control). **Four declared shapes move and the closure's type count
///   moves too, 96 -> 98** — the first count change since v31, so it is worth
///   naming what is new rather than only what moved:
///   * `EffectTarget` (card-DSL closure, reachable via `Effect`) gains
///     `ChosenObject { zone: ChoiceZone, filter: Box<TargetFilter>, count: u32,
///     up_to: bool }` and `DamagedPlayer`. **`ChoiceZone` is a NEW type in the
///     closure** (+1).
///   * `TargetFilter` gains `owner: TargetOwner`. **`TargetOwner` is a NEW type
///     in the closure** (+1). `TargetFilter` itself has been reachable since
///     v14 (`EffectFilter`) — this is a field addition to an existing member,
///     the `- 27` shape, not a new root.
///   * `TriggerCondition::WheneverCreatureDies` gains `owner:
///     Option<TargetOwner>`; `Option<T>` adds no closure member.
///   * `EffectChoiceQuestion`/`EffectChoiceAnswer` (in the closure since v31)
///     each gain a fifth variant, `ChooseObject { candidates: Vec<ObjectId>,
///     count: u32, up_to: bool }` / `ChooseObject { chosen: Vec<ObjectId> }` —
///     exactly the `- 34` shape one variant later, and every field type is
///     already present.
///
///   The v32/v36 lesson recurs and is worth restating rather than assuming
///   learned: this batch's plan predicted the wire impact from the *engine*
///   types it was adding and would have missed `TargetFilter.owner`, which
///   rides into the closure on a struct made reachable many versions ago.
///   Both numbers above are the gates' own output, not a prediction.
/// - 38: PB-DX44 stage 2a (2026-08-15, `OOS-DX29-9` — CR 702.102a / CR 709.4,
///   casting only the right half of a split card): `AltCostKind` (reachable via
///   `CastSpellData.alt_cost: Option<AltCostKind>`, part of `Command::CastSpell`,
///   in the closure since its earliest versions) gains one new variant,
///   `SplitRightHalf`. This is a variant addition to an ALREADY-reachable
///   closure member, the same shape as `- 33`'s two field additions to
///   already-reachable `Command` variants: no new type joins the closure
///   (**98 -> 98, unchanged**), but `AltCostKind`'s declared shape moves, so the
///   digest moves. Predicted in writing before this line was added
///   (`memory/primitives/pb-DX44-execution-notes.md` §1) and confirmed by the
///   gate's own output, not transcribed from a memo (PB-DX8's rule).
/// - 39: PB-DX45 (2026-09-02, `OOS-DX24-9` ≡ `OOS-DX27-5` — CR 118.12, an
///   optional cost is the PLAYER's decision): `EffectChoiceQuestion` and
///   `EffectChoiceAnswer` each gain a **sixth** variant, `PayOptionalCost`
///   (`{ cost: Cost }` and `{ pay: bool }`). Both enums have been in the closure
///   since v31, reachable from `GameEvent::EffectChoiceRequired.question` and
///   `Command::AnswerEffectChoice.answer` respectively; `Cost` has been in it
///   for longer still, via `Effect::MayPayThenEffect { cost, .. }` — which is
///   precisely the effect this batch repairs. So this is a variant addition to
///   two ALREADY-reachable members carrying an ALREADY-reachable payload: the
///   closure's type count is **98 -> 98, unchanged** (the same shape as `- 38`
///   and `- 33`), and both declared shapes move, so the digest moves.
///   Predicted in writing before any code changed
///   (`memory/primitives/pb-DX45-execution-notes.md` §0.1, including the
///   unchanged type count) and taken from the failing gate's own output rather
///   than transcribed (PB-DX8's rule).
/// - 40: PB-DX50 (2026-09-03, `OOS-DX29-2` — CR 702.140c, the mutate over/under
///   choice is made **as the spell resolves**): TWO edits, each of which would
///   move this digest on its own.
///
///   (a) `EffectChoiceQuestion` and `EffectChoiceAnswer` each gain a **seventh**
///   variant, `MutateOnTop` (`{ host: ObjectId }` and `{ on_top: bool }`), both
///   enums reachable since v31 from `GameEvent::EffectChoiceRequired.question`
///   and `Command::AnswerEffectChoice.answer`.
///
///   (b) `AdditionalCost::Mutate` **LOSES** its `on_top: bool` field, reachable
///   from `Command::CastSpell`'s `CastSpellData.additional_costs`. That half is
///   the one a prediction from the new variants alone would have missed, and it
///   is why the plan named both.
///
///   `ObjectId` and `bool` are already closure members and the removal deletes no
///   type, so the closure's type count is **98 -> 98, unchanged** (the same shape
///   as `- 39`, `- 38` and `- 33`). Predicted in writing before any code changed
///   (`memory/primitives/pb-plan-DX50.md` §0.3, including the unchanged type
///   count) and taken from the failing gate's own output rather than transcribed
///   (PB-DX8's rule).
/// - 41: PB-DX20b (2026-09-03, `OOS-DX20-10` + `OOS-DX20-5` — CR 702.5a, an
///   Enchant line that names more than one card type): `EnchantFilter` gains
///   `has_card_types: Vec<CardType>`, the OR over card **types**, beside the
///   existing single `has_card_type` and the OR over **sub**types.
///
///   Reachable from the closure root `KeywordAbility` (a declared protocol root,
///   `tests/core/protocol_schema.rs`) via
///   `KeywordAbility::Enchant(EnchantTarget)` → `EnchantTarget::Filtered(EnchantFilter)`,
///   so a new field on that struct is a wire-shape change even though nothing
///   about a `Command` or `GameEvent` moved.
///
///   `CardType` is already a closure member — the existing
///   `has_card_type: Option<CardType>` puts it there — so the closure's type count
///   is **98 -> 98, unchanged**. Predicted in writing before any code changed
///   (`memory/primitives/pb-DX20b-execution-notes.md` §0.2, including the unchanged
///   type count) and taken from the failing gate's own output rather than
///   transcribed (PB-DX8's rule).
/// - 42: **PB-DX36** (`scutemob-228`, 2026-09-04, `OOS-CARDS2-6`): two wire types move,
///   in one bump. (a) `TriggerEvent` retires `EnchantedCreatureDealsDamageToPlayer` and
///   appends seven unit variants — the combat/noncombat × player/opponent cross product
///   for the Aura family (`EnchantedCreatureDeals{Combat,Any}DamageTo{Player,Opponent}`)
///   plus the three-valued self family (`SelfDealsDamage`,
///   `SelfDealsDamageToPlayer`, `SelfDealsDamageToOpponent`). (b) `EffectAmount` gains
///   `DamageDealt` — CR 608.2h/113.7a's *"that much"* for a damage trigger, the
///   noncombat-capable sibling of `CombatDamageDealt`.
///
///   `TriggerEvent` is in this closure through
///   `Characteristics.triggered_abilities: Vec<TriggeredAbilityDef>` (the correction
///   recorded under `- 25:` above, re-verified by probe here); `EffectAmount` is in it
///   through `Effect`. **Both were probed at stage 0 by extending
///   `CLOSURE_MUST_NOT_CONTAIN` and executing the closure walk, not inferred**
///   (`memory/primitives/pb-DX36-execution-notes.md` §0.2).
///
///   **The half that does NOT move this digest is the interesting half.** The card-DSL
///   side of the same change — `TriggerCondition::WhenDealsDamage`, the new
///   `recipient: DamageRecipient` field on
///   `WhenEnchantedCreatureDealsDamageToPlayer`, and the `DamageRecipient` enum itself —
///   is entirely off-wire: `TriggerCondition` lives on the card-def
///   `AbilityDefinition::Triggered`, not on the runtime `TriggeredAbilityDef`. Probed,
///   not assumed. `DamageRecipient` was deliberately kept off `TriggerEvent` for exactly
///   this reason, which is why the closure's **type count is 98 -> 98, unchanged** —
///   predicted in writing before any code changed and taken from the failing gate's own
///   output (PB-DX8's rule: publish the figure, do not transcribe it).
/// - 43: **PB-DX52** (`scutemob-229`, 2026-09-04, `OOS-DX25b-1` + `OOS-DX25b-5`): the
///   stack-entry target id space. (a) `Target` gains a third variant,
///   `StackObject(ObjectId)`, naming an ABILITY on the stack by its own
///   `StackObject::id` — the id space in which Bolt Bend's printed *"or ability"* half
///   was unreachable, because an activated/triggered ability's stack entry owns no card
///   and is therefore never in `state.objects`. (b) `TargetRequirement` gains
///   `TargetSpellOrAbility` (CR 115.4 / CR 115.7d, ANY target count) for Deflecting
///   Swat's printed line, which carries no *"with a single target"* clause and had no
///   expressible form.
///
///   `Target` is in this closure through `Command::CastSpell.targets: Vec<Target>` (and
///   `Command`'s three other target-carrying variants), and through `SpellTarget` on
///   `GameEvent::TargetsAnnounced` / `GameEvent::TargetsChanged`.
///
///   **Closure type count is 98 -> 98, unchanged** — predicted in writing before any
///   production line changed (`memory/primitives/pb-DX52-execution-notes.md` §0.4,
///   commit `8f919967`) and taken from the failing gate's own output, because this batch
///   adds VARIANTS to two existing closure types rather than a type.
///
///   **What deliberately did NOT move**: `ResolvedTarget` (`effects/mod.rs`) was not
///   given a matching variant. It is an engine-internal enum, off-wire either way, and
///   widening it would have created ~55 `if let ResolvedTarget::Object(..)` sites with no
///   `else` that the compiler cannot flag — in exchange for nothing, since a stack-entry
///   id resolves through the same `stack_registry::stack_index_for_announced_target` a
///   card id does.
pub const PROTOCOL_VERSION: u32 = 43;

/// Digest of the serialized shape of the wire-frame type closure
/// (`Command`, `GameEvent`, [`ReplayLog`] and everything they reach).
///
/// Recomputed from workspace source by `tests/protocol_schema.rs` and compared
/// against this constant. A mismatch means the wire format changed. Update this
/// value **and** bump [`PROTOCOL_VERSION`] in the same commit.
///
/// The one exception: widening the *definition* of the closure (adding a scan
/// root, a protocol root, or an `EXTERNAL_TYPES` entry) also moves the digest
/// without any wire change. Re-pin without bumping, and say so in the commit.
///
/// This is a shape digest, not a semantic one: renaming a field, adding a
/// variant, or adding `#[serde(skip)]` all move it, but redefining what an
/// existing `u32` *means* does not. Semantic changes still require a manual
/// [`PROTOCOL_VERSION`] bump.
pub const PROTOCOL_SCHEMA_FINGERPRINT: &str =
    "e872d2393bb6b30a9ad28aecbd63a3616671f1efcfc77c58474a294173fd30c3";

/// One `(version, fingerprint)` row of the append-only protocol-schema history.
///
/// The wire-protocol analogue of [`crate::state::hash::HashSchemaEpoch`] (SR-17).
/// The protocol has a single shape digest — [`PROTOCOL_SCHEMA_FINGERPRINT`] — with
/// no separate hash byte-stream, so one fingerprint per row (not two).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtocolEpoch {
    /// The [`PROTOCOL_VERSION`] this row pins.
    pub version: u32,
    /// [`PROTOCOL_SCHEMA_FINGERPRINT`] as of that version — blake3 of the
    /// normalized declaration text of the wire-frame type closure.
    pub fingerprint: &'static str,
}

/// SR-27: append-only ledger backing [`PROTOCOL_VERSION`], mirroring
/// [`crate::state::hash::HASH_SCHEMA_HISTORY`] (SR-17).
///
/// # Why this exists on top of the fingerprint
///
/// [`PROTOCOL_SCHEMA_FINGERPRINT`] makes the version *shape-derived*: you cannot
/// change the wire without `tests/core/protocol_schema.rs` reddening and naming the
/// drift. But it does not stop the *other* half of the cheat — re-pin the
/// fingerprint to the new value and skip the [`PROTOCOL_VERSION`] bump. The
/// recompute gate goes green again and a wire change ships under the old version,
/// so two builds with incompatible shapes both claim the same version and
/// mis-decode each other *silently* — precisely the failure strict lockstep exists
/// to prevent. The `protocol_version_sentinel` forces you to *notice* a bump, never
/// to *make* one.
///
/// This table closes that. It is **append-only**: the tail row is the live schema
/// (validated by recomputation in `protocol_schema.rs`), and every row behind it is
/// shipped-and-superseded and frozen. The test pins the baseline row against its own
/// FROZEN constants and pins a digest of the whole frozen prefix, so re-pinning any
/// row in place — including the current one, while it is still the baseline — fails.
///
/// # Append-only bump procedure
///
/// To change the wire protocol, in one commit:
///   1. bump [`PROTOCOL_VERSION`] and add its `- N:` History line above;
///   2. **append** a new row here whose `fingerprint` is the recomputed digest
///      (read it from the `protocol_schema.rs` failure text) and set
///      [`PROTOCOL_SCHEMA_FINGERPRINT`] to the same value;
///   3. update the `protocol_version_sentinel` and the FROZEN prefix digest in
///      `protocol_schema.rs`.
///
/// Never edit an existing row.
///
/// The baseline is version 2 (the version at SR-27 time). Versions 1..=1 predate
/// this ledger and are not reconstructed — exactly as SR-17 started
/// `HASH_SCHEMA_HISTORY` at the then-current version rather than back-filling.
pub const PROTOCOL_HISTORY: &[ProtocolEpoch] = &[
    ProtocolEpoch {
        version: 2,
        // SR-27 (2026-07-16): baseline. Pins whatever PROTOCOL_VERSION 2 already was
        // (the 91-type closure after SR-10 boxed CastSpell). Same value as
        // PROTOCOL_SCHEMA_FINGERPRINT; the two are kept in lockstep by
        // `history_tail_matches_the_fingerprint_const`.
        fingerprint: "ba7907d9f51a65acba39ccf020a14bd6234f637731c934490a7cbf749e5f97b6",
    },
    ProtocolEpoch {
        version: 3,
        // SR-34 (2026-07-17): ManaAbility gained mana_cost/life_cost (see the `- 3:`
        // History line above).
        fingerprint: "c23d09a7956b239cc1a4edfe629b268b37a2918138def227c9ba373d805ea0f6",
    },
    ProtocolEpoch {
        version: 4,
        // SR-36 (2026-07-17): ManaAbility gained scaled_amount; ActivationCost gained
        // life_cost (see the `- 4:` History line above).
        fingerprint: "45dd82a14adf0b7e2247f7d22fad32c017adf9a25cc4129c92c489513c4ae4d4",
    },
    ProtocolEpoch {
        version: 5,
        // SR-37 (2026-07-17): ManaAbility gained activation_condition (see the `- 5:`
        // History line above).
        fingerprint: "e8d28a23ccc2a1ba7c7b2643b33bb32b0374e0651b3eb6b60ec15f4817e3a85a",
    },
    ProtocolEpoch {
        version: 6,
        // PB-EF1 (2026-07-18): ActivationCost gained sacrifice_exclude_self (see the
        // `- 6:` History line above).
        fingerprint: "df270ca1b58b7fa17bfa2ca56afb564de4f8de22cc15770da511b3a6c7c7a4dc",
    },
    ProtocolEpoch {
        version: 7,
        // PB-EF2 (2026-07-18): TokenSpec gained recipient; PlayerTarget gained
        // ControllerOfCounteredSpell/ControllerOfTriggeringObject (see the `- 7:`
        // History line above).
        fingerprint: "c5931e6163641a6a3f5501a3fc080867a05508047e4c766f2fec415d2b47ef8f",
    },
    ProtocolEpoch {
        version: 8,
        // PB-EF3 (2026-07-18): EffectTarget gained AttackTarget; PlayerTarget gained
        // DefendingPlayer (see the `- 8:` History line above).
        fingerprint: "f5a61a19da2e912416c7bf6ee58acb7cacb0966681868a6810bc8af6d2285ee8",
    },
    ProtocolEpoch {
        version: 9,
        // PB-EF4 (2026-07-18): Effect::DealDamage gained source: Option<EffectTarget>
        // (see the `- 9:` History line above).
        fingerprint: "9bf63ef25ae621acf53155feaa21f01131d35fc7ad6db34b04e35900cb825ac5",
    },
    ProtocolEpoch {
        version: 10,
        // PB-EF5 (2026-07-18): Effect gained TransformSelf (see the `- 10:` History
        // line above).
        fingerprint: "ec3ccb9e5c1cbdc834c86d6fbbc5d8ee6914e1fe1ef44eeee26d078bbea3d618",
    },
    ProtocolEpoch {
        version: 11,
        // PB-EF6 (2026-07-18): TargetRequirement gained TargetOpponent (see the `- 11:`
        // History line above).
        fingerprint: "07e514663c1b64b1831d2aaf0ee95c3e6bf62a3a1ff0b15dd3ca4316a022e739",
    },
    ProtocolEpoch {
        version: 12,
        // PB-EF7 (2026-07-18): Command::ActivateAbility gained modes_chosen;
        // AbilityDefinition::Activated gained modes (see the `- 12:` History line above).
        fingerprint: "05eaa04bf425a625415c58b3f44e6e75489c90deba14a80f7f99c91369a60cde",
    },
    ProtocolEpoch {
        version: 13,
        // PB-EF8 (2026-07-18): Cost gained ExileSelfFromHand; ActivationZone gained
        // Hand (see the `- 13:` History line above).
        fingerprint: "379fb0c4f791138a405b8b47f7efe629c9a870e026db99629da3b709ec83bafa",
    },
    ProtocolEpoch {
        version: 14,
        // PB-EF9 (2026-07-18): EffectDuration gained WhileYouControlSource (see the
        // `- 14:` History line above).
        fingerprint: "b94f90e1c6d7f4193385489f6f6d541dbb764534eab09593584f99361ea828d7",
    },
    ProtocolEpoch {
        version: 15,
        // PB-EF10 (2026-07-18): AdditionalCost::Sacrifice reshaped lki_powers -> lki;
        // TargetFilter gained max_cmc_amount (see the `- 15:` History line above).
        fingerprint: "814403943d8b2a3185bb73f5b8d2658f7f39f92f00c93d9feed08f7ecb785d1d",
    },
    ProtocolEpoch {
        version: 16,
        // PB-EF11 COMMIT 1 (2026-07-18): WheelDraw gained GreatestDiscarded (see the
        // `- 16:` History line above).
        fingerprint: "6748164f0b5b0e79d5ab8e729bac142851a7c9bb1b2c320b0e7d57a8f0cf82aa",
    },
    ProtocolEpoch {
        version: 17,
        // PB-EF11 COMMIT 2 (2026-07-18): TargetRequirement gained
        // TargetSpellWithSingleTarget (see the `- 17:` History line above).
        fingerprint: "a836605e96a0976d268ed2c37a76244b829b11a6dddd2e348a82a7b79e39976c",
    },
    ProtocolEpoch {
        version: 18,
        // PB-EF12 (2026-07-18): Command::TapForMana gained chosen_color (see the
        // `- 18:` History line above).
        fingerprint: "841e4b4130b2e2bfef5b190dc6dc57f18a2ee42a5484a652c2df690358cb115e",
    },
    ProtocolEpoch {
        version: 19,
        // PB-OS4 (2026-07-19, SHIP NARROWED): Effect gained
        // ExileSourceAndReturnTransformed (see the `- 19:` History line above).
        fingerprint: "14d2b0d4380ac53be126fd26e5541bfc834c49942cca9598921858caf442aa7c",
    },
    ProtocolEpoch {
        version: 20,
        // PB-OS5 (2026-07-19): EffectAmount gained
        // OtherAttackersSharingCreatureType (see the `- 20:` History line above).
        fingerprint: "5243cffc75ff5357ce485988f43e4df781590d48605d0875e1230a3cd6f421b6",
    },
    ProtocolEpoch {
        version: 21,
        // PB-OS6 (2026-07-19): Condition gained TopCardIsInstantOrSorcery /
        // YouAttackedWithNOrMore; Effect gained RemoveFromCombat; GameEvent gained
        // RemovedFromCombat (see the `- 21:` History line above).
        fingerprint: "c617138c61188620e1276c9113efe11a2590682c926ee16381db93f1953dd2d6",
    },
    ProtocolEpoch {
        version: 22,
        // PB-OS7 (2026-07-19, OOS-EF3-1): EffectFilter gained
        // CreaturesControlledByDefendingPlayer (see the `- 22:` History line above).
        fingerprint: "cb8af22f82c4966d1e3fc971dc28ab60bbce2058468e4cc3e1798ee307e78508",
    },
    ProtocolEpoch {
        version: 23,
        // PB-OS8 (2026-07-19, OOS-EF10-1 + min_cmc_amount rider): Effect gained
        // LookAtTopThenPlace; TargetFilter gained min_cmc_amount (see the `- 23:`
        // History line above).
        fingerprint: "553f2ff2e54c7de707209b79db7f8bca0fc0c37405871a0c1b31c431e6dedb32",
    },
    ProtocolEpoch {
        version: 24,
        // PB-OS9 (2026-07-19, OOS-EF3b-1): Condition gained YouControlYourCommander
        // (see the `- 24:` History line above).
        fingerprint: "0e538f6b09a62e9c2c2ecf667fc61d0af7c41bce875b602f509bb0cc91aaffb0",
    },
    ProtocolEpoch {
        version: 25,
        // PB-OS10 (2026-07-19, OOS-XS-1 + OOS-EF7-1): TargetRequirement gained
        // TargetPermanentDistinctFrom (see the `- 25:` History line above).
        fingerprint: "a3f9bb05a3c8e784468ac6b0946e50bb1ae43bf9d75789ef24581cd42e04fd62",
    },
    ProtocolEpoch {
        version: 26,
        // PB-OS11 (2026-07-19, final PB-OS batch — OOS-LKI-3 reframed): ManaAbility
        // gained remove_counter (see the `- 26:` History line above).
        fingerprint: "315a211a729431c5688f89d1d3517453cb2d5ffd9c3833c68cf8622387a01559",
    },
    ProtocolEpoch {
        version: 27,
        // PB-RS2 (2026-07-20, OOS-RS-2): Command::ActivateAbility and
        // Command::TapForMana each gained hybrid_choices/phyrexian_life_payments
        // (see the `- 27:` History line above).
        fingerprint: "f035e7973cc3b33a6048fe7b38b7de71f4be8d8411c719af85d6deba1c30fe3e",
    },
    ProtocolEpoch {
        version: 28,
        // PB-DP7 (2026-07-26, DP-3): Command::DiscardToHandSize and
        // GameEvent::CleanupDiscardChoiceRequired appended (see the `- 28:`
        // History line above).
        fingerprint: "bf5f5dded64029f15272c4151edd847c340793ff7ebe7d4ee32ef51be81114b4",
    },
    ProtocolEpoch {
        version: 29,
        // PB-DP8 (2026-07-26, DP-6): Command::ChooseTriggerTargets and
        // GameEvent::TriggerTargetChoiceRequired appended, and TriggerTargetOption
        // + SpellTarget enter the closure (see the `- 29:` History line above).
        fingerprint: "afdb3aebb512568b22879d5f1df6e4659378edb40a2d02e16f11f475a7bd7d48",
    },
    ProtocolEpoch {
        version: 30,
        // PB-DP8 fix cycle (2026-07-26, review Findings 2+6): TriggerTargetOption
        // gained `max` (see the `- 30:` History line above).
        fingerprint: "70faee7c16cd09f491ce60fcaad972edd42107e441b0058fd205801955e7ea79",
    },
    ProtocolEpoch {
        version: 31,
        // PB-DP9 (2026-07-27, DP-7/8/9): Command::AnswerEffectChoice and
        // GameEvent::EffectChoiceRequired appended, and EffectChoiceQuestion +
        // EffectChoiceAnswer enter the closure (see the `- 31:` History line above).
        fingerprint: "5c389360ca13beee2ff7de28a482ce99448e560d375723d4b3dbcd2380693b79",
    },
    ProtocolEpoch {
        version: 32,
        // PB-DX1 (2026-08-01, OOS-DP6-1): InterveningIf gains CardDef(Box<Condition>)
        // (see the `- 32:` History line above). Closure type count unchanged (96).
        fingerprint: "52e9b37c9612f839f7318a484f4947993295a22e2f4522fe7c19c10db663ac73",
    },
    ProtocolEpoch {
        version: 33,
        // PB-DX6 (2026-08-02, OOS-RS2-1 + OOS-DP4-1): Command::TurnFaceUp and
        // Command::DeclareAttackers both gain hybrid_choices/phyrexian_life_payments
        // (see the `- 33:` History line above). Closure type count unchanged (96).
        fingerprint: "a153b6655890ccb3335d83678d7145b27358716334ef0971b898a3a54b4997f6",
    },
    ProtocolEpoch {
        version: 34,
        // ENG-1 (2026-08-02, effect-driven discard becomes a real player choice):
        // EffectChoiceQuestion and EffectChoiceAnswer both gain a fourth Discard
        // variant (see the `- 34:` History line above). Closure type count
        // unchanged (96).
        fingerprint: "2cda8c055ffd09cf507c6d7ca366a9f24915e79268b823cc0507492a89f5e932",
    },
    ProtocolEpoch {
        version: 35,
        // ENG-2 (2026-08-02, OOS-G7-1): GameEvent gains TargetsAnnounced (see the
        // `- 35:` History line above). Closure type count unchanged (96).
        fingerprint: "7a5fc4b0c7f2e116a6674051ffa7b3455416e45cceac7e54f06d2f44698b386b",
    },
    ProtocolEpoch {
        version: 36,
        // PB-DX27 rider (2026-08-13, `OOS-ADJ-7`): LayerModification gains
        // SetLandTypes, reachable via ContinuousEffectDef (see the `- 36:`
        // History line above). Closure type count unchanged (96).
        fingerprint: "686d14e4e028f7d1148958ae58fcc17a9f359ed46c4835a864199895077f5f04",
    },
    ProtocolEpoch {
        version: 37,
        // PB-DX28 (2026-08-14, `OOS-DX4-6` + `OOS-DX4-1`): EffectTarget gains
        // ChosenObject + DamagedPlayer, TargetFilter gains `owner`,
        // WheneverCreatureDies gains `owner`, and EffectChoiceQuestion/Answer
        // each gain a fifth variant (see the `- 37:` History line above).
        // Closure type count 96 -> 98: ChoiceZone and TargetOwner are both NEW
        // members. First count change since v31.
        fingerprint: "03c5a4ac138556dd27c63a00088624287070a6107d382220b16c67b0df3d00a3",
    },
    ProtocolEpoch {
        version: 38,
        // PB-DX44 stage 2a (2026-08-15, `OOS-DX29-9`): AltCostKind gains
        // SplitRightHalf (see the `- 38:` History line above). Closure type
        // count unchanged (98).
        fingerprint: "50e69006e68918bfffde8882e0bf21e9e18a6b8afbefaf8981975b691e205a27",
    },
    ProtocolEpoch {
        version: 39,
        // PB-DX45 (2026-09-02, `OOS-DX24-9` ≡ `OOS-DX27-5`):
        // EffectChoiceQuestion/Answer each gain a sixth variant,
        // PayOptionalCost (see the `- 39:` History line above). Closure type
        // count unchanged (98).
        fingerprint: "4e3b00203568d19fa1c7a680078c86e58e2cfb2083311f07bbaf78b0c3578aab",
    },
    ProtocolEpoch {
        version: 40,
        // PB-DX50 (2026-09-03, `OOS-DX29-2`): EffectChoiceQuestion/Answer each
        // gain a seventh variant, MutateOnTop, AND `AdditionalCost::Mutate` loses
        // its `on_top` field (see the `- 40:` History line above). Closure type
        // count unchanged (98).
        fingerprint: "fbfe9b6c9696d5146dcf3f3ed9b3733c70d333a12e8d42450da45836d089ceed",
    },
    ProtocolEpoch {
        version: 41,
        // PB-DX20b (2026-09-03, `OOS-DX20-10` + `OOS-DX20-5`): EnchantFilter gains
        // has_card_types: Vec<CardType>, reachable from the KeywordAbility closure
        // root (see the `- 41:` History line above). Closure type count unchanged
        // (98).
        fingerprint: "96b7b687b5ddaade2147be0a4103cf84b3c3039f94f7259d1f32044c6d504c7b",
    },
    ProtocolEpoch {
        version: 42,
        // PB-DX36 (2026-09-04, `OOS-CARDS2-6`): TriggerEvent retires one variant and
        // appends seven (the damage-recipient cross product), and EffectAmount gains
        // DamageDealt (see the `- 42:` History line above). Both types are in the
        // closure; the card-DSL half of the same change (TriggerCondition,
        // DamageRecipient) is not, and was probed rather than assumed. Closure type
        // count unchanged (98).
        fingerprint: "9d75f591b263a7a69c78a722ffcc2bd6291bc81cf161db8c9b1bb1dad002aa47",
    },
    ProtocolEpoch {
        version: 43,
        // PB-DX52 (2026-09-04, `OOS-DX25b-1` + `OOS-DX25b-5`): `Target` gains
        // `StackObject(ObjectId)` -- an ability's stack entry, the id space Bolt Bend's
        // printed "or ability" half needed -- and `TargetRequirement` gains
        // `TargetSpellOrAbility` (CR 115.4/115.7d, any target count) for Deflecting
        // Swat. Both reachable from `Command`/`GameEvent` (see the `- 43:` History line
        // above). Closure type count unchanged (98).
        fingerprint: "e872d2393bb6b30a9ad28aecbd63a3616671f1efcfc77c58474a294173fd30c3",
    },
];

/// Why a versioned message could not be decoded.
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    /// The message declared a protocol version this build does not speak.
    ///
    /// This is the strict-lockstep rejection. The payload is **not** inspected;
    /// under a different version its shape is not knowable.
    #[error(
        "protocol version mismatch: this build speaks v{expected}, message declares v{found}. \
         Strict lockstep — client and server must run the same protocol version."
    )]
    VersionMismatch { expected: u32, found: u32 },

    /// The bytes are not a versioned envelope at all — most often an untagged
    /// message from before versioning existed, or a truncated stream.
    #[error("malformed envelope (no readable `protocol_version` field): {0}")]
    MalformedEnvelope(String),

    /// The version matched but the payload did not decode. This is a genuine
    /// bug (a peer at our own version sent something we cannot read), not a
    /// compatibility problem.
    #[error("payload failed to decode at protocol v{version}: {source}")]
    Payload {
        version: u32,
        #[source]
        source: serde_json::Error,
    },

    /// A replay log was recorded against a different state-hash schema, so its
    /// commands may decode cleanly yet replay to a state whose hash cannot be
    /// compared against the recorded one.
    #[error(
        "replay log state-hash schema mismatch: this build uses HASH_SCHEMA_VERSION {expected}, \
         log records {found}"
    )]
    HashSchemaMismatch { expected: u8, found: u8 },

    /// The value could not be serialized.
    #[error("failed to encode payload: {0}")]
    Encode(#[from] serde_json::Error),
}

/// A payload plus the protocol version that describes its shape.
///
/// `protocol_version` is serialized first and read on its own (see [`decode`]),
/// so a version mismatch is reported as [`ProtocolError::VersionMismatch`]
/// rather than as an opaque serde error about a field the reader has never
/// heard of.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope<T> {
    /// The protocol version the sender was built against.
    pub protocol_version: u32,
    /// The `Command`, `GameEvent`, `ReplayLog`, or batch thereof.
    pub payload: T,
}

impl<T> Envelope<T> {
    /// Wrap a payload at this build's [`PROTOCOL_VERSION`].
    pub fn new(payload: T) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            payload,
        }
    }

    /// Unwrap, discarding the (already-validated) version tag.
    pub fn into_payload(self) -> T {
        self.payload
    }
}

/// Reads only the version tag, ignoring the payload entirely.
///
/// This is the whole reason decoding is staged: the payload's *shape* is only
/// knowable once the version is known to match, so it must not be parsed first.
#[derive(Deserialize)]
struct VersionProbe {
    protocol_version: u32,
}

/// Serialize a payload inside a versioned envelope.
///
/// Works for `Command`, `GameEvent`, `Vec<Command>`, [`ReplayLog`] — anything
/// `Serialize`.
pub fn encode<T: Serialize>(payload: &T) -> Result<String, ProtocolError> {
    Ok(serde_json::to_string(&Envelope::new(payload))?)
}

/// Deserialize a versioned envelope, rejecting any version but our own.
///
/// Staged on purpose:
/// 1. read `protocol_version` alone;
/// 2. reject a mismatch **before** touching the payload;
/// 3. only then decode the payload.
///
/// Step 2 is what makes [`ProtocolError::VersionMismatch`] reachable. Decoding
/// straight into `Envelope<T>` would instead surface an old message as a serde
/// error about an unknown variant — true, but useless to a client deciding
/// whether to reconnect or to tell the user to upgrade.
pub fn decode<T: DeserializeOwned>(json: &str) -> Result<T, ProtocolError> {
    let probe: VersionProbe =
        serde_json::from_str(json).map_err(|e| ProtocolError::MalformedEnvelope(e.to_string()))?;

    if probe.protocol_version != PROTOCOL_VERSION {
        return Err(ProtocolError::VersionMismatch {
            expected: PROTOCOL_VERSION,
            found: probe.protocol_version,
        });
    }

    let envelope: Envelope<T> =
        serde_json::from_str(json).map_err(|source| ProtocolError::Payload {
            version: probe.protocol_version,
            source,
        })?;

    Ok(envelope.payload)
}

/// A recorded command stream: everything needed to replay a game from turn 1.
///
/// Carries **two** versions because a replay must answer two different
/// questions, and passing one does not imply passing the other:
///
/// - `protocol_version` (on the [`Envelope`]) — can this build *read* the
///   commands?
/// - `hash_schema_version` — can this build's state hashes be *compared*
///   against the ones this log was recorded alongside?
///
/// A log can decode perfectly and still replay to states whose hashes are
/// incomparable, which would silently break the desync detection that invariant
/// #9's history rests on. [`decode_replay_log`] checks both.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayLog {
    /// `state::hash::HASH_SCHEMA_VERSION` at record time.
    pub hash_schema_version: u8,
    /// The commands, in submission order.
    pub commands: Vec<Command>,
}

impl ReplayLog {
    /// Record a command stream against this build's state-hash schema.
    pub fn new(commands: Vec<Command>) -> Self {
        Self {
            hash_schema_version: HASH_SCHEMA_VERSION,
            commands,
        }
    }
}

/// Encode a replay log inside a versioned envelope.
pub fn encode_replay_log(log: &ReplayLog) -> Result<String, ProtocolError> {
    encode(log)
}

/// Decode a replay log, checking the protocol version *and* the state-hash schema.
///
/// The hash-schema check is deliberately separate from and after the protocol
/// check: a log whose commands we cannot read is a different failure from one we
/// can read but cannot verify.
pub fn decode_replay_log(json: &str) -> Result<ReplayLog, ProtocolError> {
    let log: ReplayLog = decode(json)?;
    if log.hash_schema_version != HASH_SCHEMA_VERSION {
        return Err(ProtocolError::HashSchemaMismatch {
            expected: HASH_SCHEMA_VERSION,
            found: log.hash_schema_version,
        });
    }
    Ok(log)
}
