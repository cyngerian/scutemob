//! Stack object types: spells and abilities on the stack (CR 405).
//!
//! The stack is an ordered zone (LIFO). When a spell is cast or an ability
//! is activated/triggered, a StackObject is pushed onto the stack.
//! Resolution pops the top object off the stack (CR 608.1).
//!
//! For spells, the corresponding card has moved to `ZoneId::Stack` and appears
//! as a `GameObject` there. For abilities, no corresponding `GameObject` exists
//! in the Stack zone — the `StackObject` alone represents the ability on the stack.
use super::game_object::{ManaCost, ObjectId};
use super::player::{CardId, PlayerId};
use super::targeting::SpellTarget;
use super::types::{AdditionalCost, ChampionFilter, CumulativeUpkeepCost, KeywordAbility};
use serde::{Deserialize, Serialize};
/// Captured data for triggered abilities on the stack.
/// Replaces per-trigger StackObjectKind variants with a unified payload.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriggerData {
    /// No extra data needed (Melee, Exploit, Training, Hideaway-simple, etc.).
    Simple,
    /// Counter-removal upkeep triggers (Vanishing, Fading, Impending).
    CounterRemoval { permanent: ObjectId },
    /// Vanishing/Fading/Impending sacrifice (counter reached 0).
    CounterSacrifice { permanent: ObjectId },
    /// Echo/CumulativeUpkeep (cost-based upkeep).
    UpkeepCost {
        permanent: ObjectId,
        cost: UpkeepCostKind,
    },
    // --- Group 1: Combat triggers ---
    /// Flanking: blocker gets -1/-1.
    CombatFlanking { blocker: ObjectId },
    /// Rampage N: +N/+N for each blocker beyond the first.
    CombatRampage { n: u32 },
    /// Provoke: target creature must block if able.
    CombatProvoke { target: ObjectId },
    /// Poisonous N: target player gets N poison counters.
    CombatPoisonous { target_player: PlayerId, n: u32 },
    /// Enlist: source gets +X/+0 where X is enlisted creature's power.
    CombatEnlist { enlisted: ObjectId },
    // --- Group 2: ETB triggers ---
    /// Backup N: place counters on target, optionally grant abilities.
    ETBBackup {
        target: ObjectId,
        count: u32,
        abilities: Vec<KeywordAbility>,
    },
    /// Graft: move a +1/+1 counter to entering creature.
    ETBGraft { entering_creature: ObjectId },
    /// Champion ETB: exile another permanent or sacrifice self.
    ETBChampion { filter: ChampionFilter },
    /// Soulbond: pair with target creature.
    ETBSoulbond { pair_target: ObjectId },
    /// Ravenous draw: draw a card if X >= 5.
    ETBRavenousDraw { permanent: ObjectId, x_value: u32 },
    /// Squad: create N token copies.
    ETBSquad { count: u32 },
    /// Offspring: create 1/1 token copy; source_card_id for LKI.
    ETBOffspring { source_card_id: Option<CardId> },
    /// Gift: give gift to chosen opponent.
    ETBGift {
        source_card_id: Option<CardId>,
        gift_opponent: PlayerId,
    },
    /// Hideaway N: look at top N cards, exile one face down.
    ETBHideaway { count: u32 },
    /// PartnerWith: search library for partner card.
    ETBPartnerWith {
        partner_name: String,
        target_player: PlayerId,
    },
    // --- Group 3: Spell-copy triggers ---
    /// Storm/Replicate/Gravestorm: create N copies of original spell.
    SpellCopy {
        original_stack_id: ObjectId,
        copy_count: u32,
    },
    /// Cascade: exile cards until finding one with lower mana value.
    CascadeExile { spell_mana_value: u32 },
    /// Casualty: create one copy of the original spell.
    CasualtyCopy { original_stack_id: ObjectId },
    // --- Group 4: EOT/delayed zone-change triggers ---
    /// Delayed zone change (Dash return, Blitz sacrifice, Unearth exile, Evoke sacrifice).
    DelayedZoneChange,
    /// Encore sacrifice: delayed sacrifice with activator tracking.
    EncoreSacrifice { activator: PlayerId },
    // --- Group 5: Death/LTB triggers ---
    /// Modular: move N +1/+1 counters to target artifact creature.
    DeathModular { counter_count: u32 },
    /// Haunt exile: move haunt card from graveyard to exile targeting a creature.
    DeathHauntExile {
        haunt_card: ObjectId,
        haunt_card_id: Option<CardId>,
    },
    /// Haunted creature dies: fire haunt effect from exile.
    DeathHauntedCreatureDies {
        haunt_source: ObjectId,
        haunt_card_id: Option<CardId>,
    },
    /// Champion LTB: return exiled card to battlefield.
    LTBChampion { exiled_card: ObjectId },
    /// Recover: pay cost or exile card.
    DeathRecover {
        recover_card: ObjectId,
        recover_cost: ManaCost,
    },
    // --- Group 6: Remaining triggers ---
    /// Cipher: copy encoded spell on combat damage.
    CipherDamage {
        source_creature: ObjectId,
        encoded_card_id: CardId,
        encoded_object_id: ObjectId,
    },
    /// Ingest: exile top card of damaged player's library.
    IngestExile { target_player: PlayerId },
    /// Renown N: place N +1/+1 counters and become renowned.
    RenownDamage { n: u32 },
    /// Evolve: put +1/+1 counter if entering creature has greater P or T.
    ETBEvolve { entering_creature: ObjectId },
    /// Myriad: create token copies attacking each other opponent.
    MyriadAttack { defending_player: PlayerId },
    /// Madness: cast from exile using the madness cost.
    /// `exiled_card` is the ObjectId in exile (== source in practice);
    /// `cost` is the madness alternative mana cost.
    Madness {
        exiled_card: ObjectId,
        cost: ManaCost,
    },
    /// Miracle: cast from hand using the miracle cost.
    /// `revealed_card` is the ObjectId in hand (== source in practice);
    /// `cost` is the miracle alternative mana cost.
    Miracle {
        revealed_card: ObjectId,
        cost: ManaCost,
    },
    /// Suspend counter removal or cast trigger.
    /// `card` is the exiled suspended card (== source in practice).
    Suspend { card: ObjectId },
    /// CR 603.7: Delayed trigger action — fires when a delayed trigger's timing
    /// condition is met (e.g., "at next end step", "when source leaves").
    /// `action` is what to do; `target` is the object to act on.
    DelayedAction {
        action: super::stubs::DelayedTriggerAction,
        target: ObjectId,
    },
}
/// Cost payload for upkeep-cost triggers.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpkeepCostKind {
    Echo(ManaCost),
    CumulativeUpkeep(CumulativeUpkeepCost),
}
/// An object on the stack: a spell, activated ability, or triggered ability
/// (CR 405.1).
///
/// Stack objects are ordered LIFO — the last one pushed is the first to resolve.
/// Use `GameState.stack_objects` to access the stack (index 0 = bottom, last = top).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StackObject {
    /// Unique identifier for this stack item.
    pub id: ObjectId,
    /// The player who controls this spell or ability (CR 108.4).
    pub controller: PlayerId,
    /// What kind of object this is (spell, activated ability, or triggered ability).
    pub kind: StackObjectKind,
    /// Targets announced at cast time (CR 601.2c). Empty for non-targeting spells.
    /// Validated again at resolution for the fizzle rule (CR 608.2b).
    pub targets: Vec<SpellTarget>,
    /// CR 101.6: If true, this spell can't be countered by spells or abilities.
    /// Set from the card definition at cast time.
    #[serde(default)]
    pub cant_be_countered: bool,
    /// CR 707.10: If true, this is a copy of a spell — it has no physical card
    /// object to move when it resolves. Copies execute the spell's effect but do
    /// NOT move a source card to graveyard or battlefield at resolution.
    ///
    /// Copies are NOT cast (no "cast" triggers), are not affected by effects that
    /// care about casting, and do not go to graveyard when they finish resolving.
    #[serde(default)]
    pub is_copy: bool,
    /// CR 702.34a: If true, this spell was cast via flashback. When it leaves the
    /// stack (resolves, is countered, or fizzles), it is exiled instead of going
    /// to any other zone. Set at cast time in `handle_cast_spell`.
    ///
    /// Must always be false for copies (`is_copy: true`) — copies are not cast.
    #[serde(default)]
    pub cast_with_flashback: bool,
    /// CR 702.33d: Number of times the kicker cost was paid when this spell was cast.
    ///
    /// 0 = not kicked. 1 = kicked (standard kicker). N = multikicked N times
    /// (CR 702.33c). Set at cast time and propagated to copies on the stack
    /// (CR ruling: copies of kicked spells are also kicked).
    #[serde(default)]
    pub kicker_times_paid: u32,
    /// CR 702.74a: If true, this spell was cast by paying its evoke cost
    /// (an alternative cost). When the permanent enters the battlefield,
    /// the evoke sacrifice trigger fires.
    ///
    /// Must always be false for copies (`is_copy: true`) — copies are not cast.
    #[serde(default)]
    pub was_evoked: bool,
    /// CR 702.103b: If true, this spell was cast by paying its bestow cost.
    /// On the stack, this spell is an Aura enchantment (not a creature) with
    /// enchant creature. At resolution, if the target is illegal, it ceases
    /// to be bestowed and resolves as a creature (CR 702.103e / 608.3b).
    ///
    /// CR 702.103c: If the original spell was bestowed, copies are also bestowed
    /// Aura spells. Any rule that refers to a spell cast bestowed applies to the
    /// copy as well.
    #[serde(default)]
    pub was_bestowed: bool,
    /// CR 702.35a: If true, this spell was cast via madness from exile. The card
    /// was exiled during a discard, and the owner chose to cast it by paying the
    /// madness cost. Unlike flashback, madness does NOT change where the card goes
    /// on resolution -- it resolves normally (permanent to battlefield, instant/sorcery
    /// to graveyard).
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub cast_with_madness: bool,
    /// CR 702.94a: If true, this spell was cast via miracle from hand. The card
    /// was drawn as the first card this turn, revealed, and the owner chose to
    /// cast it by paying the miracle cost. Like madness, miracle does NOT change
    /// where the card goes on resolution -- it resolves normally.
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub cast_with_miracle: bool,
    /// CR 702.138b: If true, this spell was cast via escape from the graveyard.
    /// The spell's escape cost (mana + exiling other cards) was paid as an
    /// alternative cost. Unlike flashback, escape does NOT change where the
    /// spell goes on resolution -- it resolves normally.
    ///
    /// This flag is propagated to the permanent as `was_escaped` at resolution
    /// time (for "escapes with [counter]" and "escapes with [ability]" effects).
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub was_escaped: bool,
    /// CR 702.143a: If true, this spell was cast from exile by paying its foretell
    /// cost. The foretell cost is an alternative cost (CR 118.9). Unlike flashback,
    /// foretell does NOT change where the card goes on resolution -- it resolves
    /// normally (permanent to battlefield, instant/sorcery to graveyard).
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub cast_with_foretell: bool,
    /// CR 702.27a: If true, this spell was cast with its buyback cost paid as an
    /// additional cost. On resolution, the spell returns to its owner's hand
    /// instead of going to the graveyard.
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub was_buyback_paid: bool,
    /// CR 702.62a: If true, this spell was cast via the suspend cast trigger
    /// (the last time counter was removed). The spell was cast without paying
    /// its mana cost. If the spell is a creature, the permanent gains haste
    /// when it enters the battlefield.
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub was_suspended: bool,
    /// CR 702.96a: If true, this spell was cast by paying its overload cost
    /// (an alternative cost). At resolution, the spell's effect uses the
    /// "each" (all-matching) branch instead of the "target" (single-target)
    /// branch. The spell has no targets and cannot fizzle.
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub was_overloaded: bool,
    /// CR 702.133a: If true, this spell was cast via jump-start from the graveyard.
    /// When it leaves the stack (resolves, is countered, or fizzles), it is exiled
    /// instead of going to any other zone. Same departure behavior as flashback.
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub cast_with_jump_start: bool,
    /// CR 702.127a: If true, this spell was cast as the aftermath half of a split
    /// card from the graveyard. At resolution, the aftermath half's effect is used
    /// instead of the first half's Spell effect.
    ///
    /// The exile-on-stack-departure behavior is handled by `cast_with_flashback`
    /// being set to true (same mechanism as Flashback and Jump-start).
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub cast_with_aftermath: bool,
    /// CR 702.109a: If true, this spell was cast by paying its dash cost
    /// (an alternative cost). When the permanent enters the battlefield,
    /// it gains haste and a delayed trigger returns it at end step.
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub was_dashed: bool,
    /// CR 702.152a: If true, this spell was cast by paying its blitz cost
    /// (an alternative cost). When the permanent enters the battlefield,
    /// it gains haste, gains "When this dies, draw a card," and a delayed
    /// trigger sacrifices it at the beginning of the next end step.
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub was_blitzed: bool,
    /// CR 702.170d: If true, this spell was cast from exile as a plotted card
    /// (without paying its mana cost). Used for tracking that this was a plot-cast.
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub was_plotted: bool,
    /// CR 718.3b: If true, this spell was cast as a prototyped spell.
    ///
    /// The spell uses its alternative power, toughness, mana cost, and color
    /// (derived from the prototype mana cost) instead of the card's normal values.
    ///
    /// IMPORTANT: This is NOT mutually exclusive with other alt-cost flags (CR 118.9,
    /// ruling 2022-10-14). A spell can be both prototyped AND cast with flashback.
    ///
    /// CR 718.3c: If a prototyped spell is copied, the copy is also a prototyped
    /// spell (copies inherit was_prototyped via copy system).
    #[serde(default)]
    pub was_prototyped: bool,
    /// CR 702.176a: If true, this spell was cast by paying its impending cost
    /// (an alternative cost). When the permanent enters the battlefield, it
    /// enters with N time counters and is not a creature while it has time
    /// counters.
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub was_impended: bool,
    /// CR 702.166b: If true, this spell was cast with its bargain cost paid
    /// (sacrificed an artifact, enchantment, or token as an additional cost).
    /// Used by `Condition::WasBargained` to check at resolution time.
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    /// Note: Copies of a bargained spell are also bargained (CR 707.2), so
    /// this should be propagated to copies in the copy system.
    #[serde(default)]
    pub was_bargained: bool,
    /// CR 701.59c: If true, this spell was cast with its collect evidence cost paid
    /// (exiled cards from graveyard with total mana value >= N as an additional cost).
    /// Used by `Condition::EvidenceWasCollected` to check at resolution time.
    /// Enables linked "if evidence was collected" ability effects (CR 701.59c / 607).
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub evidence_collected: bool,
    /// CR 702.117a: If true, this spell was cast by paying its surge cost
    /// (an alternative cost). Used to enable "if surge cost was paid" conditional
    /// effects on permanents (e.g., Crush of Tentacles, Reckless Bushwhacker).
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub was_surged: bool,
    /// CR 702.153a: If true, this spell was cast with its casualty cost paid
    /// (sacrificed a creature with power >= N as an additional cost).
    /// Used to drive the CasualtyTrigger that creates a copy of this spell.
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    /// Note: Unlike Bargain, this flag does NOT propagate to the permanent
    /// because no permanent cares whether casualty was paid after resolution.
    #[serde(default)]
    pub was_casualty_paid: bool,
    /// CR 702.148a: If true, this spell was cast by paying its cleave cost
    /// (an alternative cost). Used by `Condition::WasCleaved` to branch between
    /// the restricted (normal) and broadened (cleaved) spell effects at resolution.
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub was_cleaved: bool,
    /// CR 715.3d: If true, this spell was cast as an Adventure (using the adventure face's
    /// characteristics). On successful resolution, the card is exiled instead of going
    /// to the graveyard. From exile, the controller may cast the creature half (but NOT
    /// as an Adventure again per CR 715.3d).
    ///
    /// If countered or fizzled, the card goes to graveyard normally (NOT exile) —
    /// exile only happens on successful resolution (CR 715.3d).
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub was_cast_as_adventure: bool,
    // was_entwined: REMOVED — read from AdditionalCost::Entwine in additional_costs
    // escalate_modes_paid: REMOVED — read from AdditionalCost::EscalateModes in additional_costs
    /// CR 702.47a: Effects from cards spliced onto this spell.
    ///
    /// Each entry is an `Effect` from a spliced card's `AbilityDefinition::Splice.effect`.
    /// At resolution, these effects are executed in order after the main spell's effect
    /// (CR 702.47b: "The effects of the main spell must happen first").
    ///
    /// Empty vec = no splice. These are stored on the StackObject so they are
    /// automatically discarded when the spell leaves the stack for any reason
    /// (CR 702.47e: "The spell loses any splice changes once it leaves the stack").
    #[serde(default)]
    pub spliced_effects: Vec<crate::cards::card_definition::Effect>,
    /// CR 702.47a: ObjectIds of cards spliced onto this spell (for validation and display).
    ///
    /// Used to enforce CR 702.47b: "You can't splice any one card onto the same spell
    /// more than once." Also used to verify each splice card is in the caster's hand
    /// at resolution (though by 702.47a the reveal is at cast time and the card stays
    /// in hand regardless of what happens to the spell).
    #[serde(default)]
    pub spliced_card_ids: Vec<ObjectId>,
    // devour_sacrifices: REMOVED — use AdditionalCost::Sacrifice in additional_costs
    /// CR 700.2a / 601.2b: Mode indices chosen at cast time for a modal spell.
    /// Empty for non-modal spells or when mode[0] is auto-selected (backward compatible).
    /// Propagated to copies per CR 700.2g / 707.10.
    #[serde(default)]
    pub modes_chosen: Vec<usize>,
    // was_fused: REMOVED — read from AdditionalCost::Fuse in additional_costs
    /// CR 107.3m: The value of X chosen when this spell was cast. 0 for non-X spells.
    /// Propagated from CastSpell.x_value at cast time and copied to GameObject.x_value
    /// at resolution so ETB replacement effects and triggers can read it.
    #[serde(default)]
    pub x_value: u32,
    // squad_count: REMOVED — read from AdditionalCost::Squad in additional_costs
    // offspring_paid: REMOVED — read from AdditionalCost::Offspring in additional_costs
    // gift_was_given: REMOVED — read from AdditionalCost::Gift in additional_costs
    // gift_opponent: REMOVED — read from AdditionalCost::Gift in additional_costs
    // mutate_target: REMOVED — read from AdditionalCost::Mutate in additional_costs
    // mutate_on_top: REMOVED — read from AdditionalCost::Mutate in additional_costs
    /// CR 712.11a / CR 702.146a: If true, this spell was cast "transformed" — that is,
    /// it was placed on the stack with its back face up.
    ///
    /// Currently set when Disturb alternative cost is used. At resolution, the permanent
    /// enters the battlefield with `is_transformed = true` and `was_cast_disturbed = true`.
    ///
    /// CR 712.8c: The spell's mana value uses the front face's mana cost even when cast
    /// transformed.
    ///
    /// Must always be false for copies unless explicitly copied as transformed.
    #[serde(default)]
    pub is_cast_transformed: bool,
    /// Consolidated additional costs (RC-1 type consolidation).
    /// Mirrors `CastSpell.additional_costs`. Populated during cast-to-stack transfer.
    #[serde(default)]
    pub additional_costs: Vec<AdditionalCost>,
    /// CR 510.3a: The player dealt combat damage in the triggering event.
    /// Set from PendingTrigger::damaged_player when a triggered ability is flushed to the stack.
    /// Read by PlayerTarget::DamagedPlayer at resolution time.
    #[serde(default)]
    pub damaged_player: Option<PlayerId>,
    /// CR 510.3a: The amount of combat damage dealt in the triggering event.
    /// Set from PendingTrigger::combat_damage_amount when a triggered ability is flushed.
    /// Read by EffectAmount::CombatDamageDealt at resolution time.
    #[serde(default)]
    pub combat_damage_amount: u32,
    /// CR 510.3a: The ObjectId of the creature that triggered a per-creature combat damage trigger.
    /// Set from PendingTrigger::entering_object_id for per-creature combat damage triggers.
    /// Read by EffectTarget::TriggeringCreature at resolution time.
    #[serde(default)]
    pub triggering_creature_id: Option<super::game_object::ObjectId>,
    /// CR 608.2b: Captured LKI powers of creatures sacrificed as a cost when this
    /// activated ability was put on the stack. Populated at activated-ability
    /// cost-payment time (abilities.rs sacrifice block) BEFORE `move_object_to_zone`.
    /// Read at resolution time and copied into `EffectContext.sacrificed_creature_powers`
    /// so `EffectAmount::PowerOfSacrificedCreature` resolves correctly.
    /// Empty for stack objects whose costs did not include creature sacrifice.
    #[serde(default)]
    pub sacrificed_creature_powers: Vec<i32>,
    /// PB-A: If true, this spell was cast from the top of the library via a permission
    /// that has an `on_cast_effect` (e.g. Thundermane Dragon grants haste).
    ///
    /// When this flag is set, `resolution.rs` directly inserts `KeywordAbility::Haste`
    /// into the permanent's keywords after it enters the battlefield. This avoids the
    /// CR 400.7 problem: a continuous effect targeting the *stack* ObjectId would never
    /// apply to the *battlefield* permanent (which has a different ObjectId).
    ///
    /// Must always be false for copies (`is_copy: true`) -- copies are not cast.
    #[serde(default)]
    pub cast_from_top_with_bonus: bool,
}
impl StackObject {
    /// Build a triggered-ability StackObject with all cast-specific fields set to
    /// their "not-a-spell" defaults (false/empty/zero). Use this for keyword triggers
    /// (Storm, Gravestorm, Cascade, Casualty, Replicate, etc.) to eliminate boilerplate.
    ///
    /// Required fields that must still be supplied by the caller:
    /// - `id`: unique ObjectId for this stack entry
    /// - `controller`: the player who controls the ability
    /// - `kind`: the specific StackObjectKind (KeywordTrigger, etc.)
    ///
    /// Optional fields left at caller's discretion:
    /// - `targets`: defaults to empty; set if the trigger has targets
    /// - `cant_be_countered`: defaults to false; set if the trigger can't be countered
    ///
    /// MR-TC-25: Eliminates ~400 lines of repeated boilerplate across Storm / Gravestorm /
    /// Cascade / Casualty / Replicate trigger constructors in casting.rs.
    pub fn trigger_default(
        id: super::game_object::ObjectId,
        controller: super::PlayerId,
        kind: StackObjectKind,
    ) -> Self {
        Self {
            id,
            controller,
            kind,
            targets: vec![],
            cant_be_countered: false,
            is_copy: false,
            cast_with_flashback: false,
            kicker_times_paid: 0,
            was_evoked: false,
            was_bestowed: false,
            cast_with_madness: false,
            cast_with_miracle: false,
            was_escaped: false,
            cast_with_foretell: false,
            was_buyback_paid: false,
            was_suspended: false,
            was_overloaded: false,
            cast_with_jump_start: false,
            cast_with_aftermath: false,
            was_dashed: false,
            was_blitzed: false,
            was_plotted: false,
            was_prototyped: false,
            was_impended: false,
            was_bargained: false,
            was_surged: false,
            was_casualty_paid: false,
            was_cleaved: false,
            was_cast_as_adventure: false,
            spliced_effects: vec![],
            spliced_card_ids: vec![],
            modes_chosen: vec![],
            x_value: 0,
            evidence_collected: false,
            is_cast_transformed: false,
            additional_costs: vec![],
            damaged_player: None,
            combat_damage_amount: 0,
            triggering_creature_id: None,
            sacrificed_creature_powers: vec![],
            cast_from_top_with_bonus: false,
        }
    }
}
/// The kind of object on the stack.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StackObjectKind {
    /// A spell being cast (CR 601).
    ///
    /// The `source_object` is the `ObjectId` of the card now in `ZoneId::Stack`.
    /// When the spell resolves, the card moves to the graveyard (or stays in play
    /// for permanents).
    Spell { source_object: ObjectId },
    /// An activated ability (CR 602).
    ///
    /// The `source_object` remains in whatever zone it is in — the source does
    /// NOT move to the stack when an activated ability is put on the stack.
    /// `ability_index` identifies which ability on the source object this is.
    ActivatedAbility {
        source_object: ObjectId,
        ability_index: usize,
        /// Effect captured at activation time (CR 602.2). Required when the source
        /// is sacrificed as a cost and will not exist at resolution time (CR 608.3b).
        /// Boxed to keep the enum variant size manageable (clippy::large_enum_variant).
        #[serde(default)]
        embedded_effect: Option<Box<crate::cards::card_definition::Effect>>,
    },
    /// CR 606: A loyalty ability on the stack (CR 606.2).
    ///
    /// The loyalty cost (add/remove counters) was paid at activation time.
    /// Resolution executes the effect. The source planeswalker must still be
    /// on the battlefield for the effect to resolve (standard ability rules).
    LoyaltyAbility {
        source_object: ObjectId,
        /// Index into the planeswalker's loyalty abilities (filtered from CardDefinition).
        ability_index: usize,
        /// The effect to execute on resolution. Captured at activation time so it
        /// resolves correctly even if the source leaves the battlefield.
        effect: Box<crate::cards::card_definition::Effect>,
    },
    /// A triggered ability (CR 603).
    ///
    /// The `source_object` may be in any zone (it triggered from wherever it
    /// was when the trigger condition was met). `ability_index` identifies
    /// which ability triggered.
    ///
    /// `is_carddef_etb` signals that `ability_index` is into the card registry's
    /// `CardDefinition::abilities` Vec, NOT into the runtime `triggered_abilities`.
    /// Set by `queue_carddef_etb_triggers` via `PendingTriggerKind::CardDefETB`.
    TriggeredAbility {
        source_object: ObjectId,
        ability_index: usize,
        /// When true, resolution always uses the card registry path for effect lookup.
        /// Prevents index-namespace collisions with runtime triggered_abilities.
        #[serde(default)]
        is_carddef_etb: bool,
    },
    /// CR 702.35a: Madness triggered ability on the stack.
    ///
    /// When a card with madness is discarded and exiled by the madness static
    /// ability, this trigger fires: "When this card is exiled this way, its
    /// owner may cast it by paying [cost] rather than paying its mana cost.
    /// If that player doesn't, they put this card into their graveyard."
    ///
    /// `exiled_card` is the ObjectId of the card in exile (new ID after zone move).
    /// `madness_cost` is captured at trigger time from the card definition.
    MadnessTrigger {
        source_object: ObjectId,
        exiled_card: ObjectId,
        madness_cost: crate::state::game_object::ManaCost,
        owner: PlayerId,
    },
    /// CR 702.94a: Miracle triggered ability on the stack.
    ///
    /// When a player reveals a card using its miracle ability (as the first draw of
    /// the turn), this trigger fires: "When you reveal this card this way, you may
    /// cast it by paying [cost] rather than its mana cost."
    ///
    /// `revealed_card` is the ObjectId of the card in hand (new ID after draw zone move).
    /// `miracle_cost` is captured at trigger time from the card definition.
    /// When this trigger resolves, the player may have already cast the card (from hand
    /// using `CastSpell` with `cast_with_miracle: true` while the trigger was on the stack).
    /// If the card is still in hand at resolution, it stays there (player declined).
    MiracleTrigger {
        source_object: ObjectId,
        revealed_card: ObjectId,
        miracle_cost: crate::state::game_object::ManaCost,
        owner: PlayerId,
    },
    /// CR 702.84a: Unearth activated ability on the stack.
    ///
    /// When this ability resolves: (1) move the source card from graveyard to
    /// battlefield, (2) grant haste, (3) set was_unearthed flag, (4) fire normal
    /// ETB triggers.
    ///
    /// If the source card is no longer in the graveyard at resolution time,
    /// the ability does nothing (card was exiled, shuffled, etc.) -- CR 400.7.
    UnearthAbility { source_object: ObjectId },
    /// CR 702.100a: Evolve trigger on the stack.
    ///
    /// When a creature with evolve sees another creature its controller controls
    // EvolveTrigger: migrated to KeywordTrigger
    /// CR 702.62a: Suspend upkeep counter-removal trigger.
    ///
    /// "At the beginning of your upkeep, if this card is suspended, remove a
    /// time counter from it." Fires at the start of the card owner's upkeep step.
    ///
    /// When this trigger resolves:
    /// 1. Check if the card is still in exile and still suspended (CR 603.4).
    /// 2. Remove one time counter.
    /// 3. If that was the last counter, queue a SuspendCastTrigger.
    ///
    /// If countered (e.g., Stifle), no counter is removed.
    SuspendCounterTrigger {
        source_object: ObjectId,
        suspended_card: ObjectId,
    },
    /// CR 702.62a: Suspend cast trigger (last counter removed).
    ///
    /// "When the last time counter is removed from this card, if it's exiled,
    /// you may play it without paying its mana cost if able." (CR 702.62a)
    ///
    /// When this trigger resolves:
    /// 1. Check if the card is still in exile (CR 603.4 intervening-if).
    /// 2. Cast the card without paying its mana cost.
    ///    - Timing restrictions are ignored.
    ///    - If the spell is a creature, clear summoning sickness at ETB.
    ///
    /// If countered (e.g., Stifle), the card stays in exile with 0 time counters
    /// (no longer suspended per CR 702.62b).
    SuspendCastTrigger {
        source_object: ObjectId,
        suspended_card: ObjectId,
        owner: crate::state::player::PlayerId,
    },
    // FlankingTrigger, RampageTrigger, ProvokeTrigger, RenownTrigger,
    // MeleeTrigger, PoisonousTrigger, EnlistTrigger: migrated to KeywordTrigger
    /// CR 702.49a: Ninjutsu activated ability on the stack.
    ///
    /// When this ability resolves: put the ninja card from hand (or command
    /// zone) onto the battlefield tapped and attacking the captured
    /// `attack_target`.
    ///
    /// `source_object` / `ninja_card` are the ObjectId of the card in
    /// hand/command zone (same value; `source_object` follows the convention
    /// used by other `StackObjectKind` variants).
    /// `attack_target` is the attack target inherited from the returned attacker.
    /// `from_command_zone` indicates commander ninjutsu (CR 702.49d).
    ///
    /// If the ninja card is no longer in hand/command zone at resolution
    /// time, the ability does nothing (CR 400.7 -- object left the expected zone).
    NinjutsuAbility {
        source_object: ObjectId,
        ninja_card: ObjectId,
        attack_target: crate::state::combat::AttackTarget,
        from_command_zone: bool,
    },
    /// CR 702.128a: Embalm activated ability on the stack.
    ///
    /// When this ability resolves: create a token that's a copy of the source card,
    /// except it's white, has no mana cost, and is a Zombie in addition to its
    /// other types (CR 702.128a).
    ///
    /// `source_card_id` is the CardId (registry key) of the card that was exiled as cost.
    /// The original card was exiled during activation (cost payment), so no ObjectId is
    /// available at resolution time (CR 400.7). The token's characteristics come from
    /// the CardDefinition in the registry.
    EmbalmAbility {
        source_card_id: Option<crate::state::player::CardId>,
    },
    /// CR 702.129a: Eternalize activated ability on the stack.
    ///
    /// When this ability resolves: create a token that's a copy of the source card,
    /// except it's black, it's 4/4, has no mana cost, and is a Zombie in addition
    /// to its other types (CR 702.129a).
    ///
    /// `source_card_id` is the CardId (registry key) of the card that was exiled as cost.
    /// The original card was exiled during activation (cost payment), so no ObjectId is
    /// available at resolution time (CR 400.7). The token's characteristics come from
    /// the CardDefinition in the registry.
    /// `source_name` is the name of the source card, stored for display in the TUI
    /// (since the card is already in exile at activation time).
    EternalizeAbility {
        source_card_id: Option<crate::state::player::CardId>,
        source_name: String,
    },
    /// CR 702.141a: Encore activated ability on the stack.
    ///
    /// When this ability resolves: for each active opponent, create a token
    /// copy of the exiled card with haste, tagged `encore_sacrifice_at_end_step = true`
    /// and `encore_must_attack = Some(opponent_id)`.
    ///
    /// `source_card_id` is the CardId of the exiled creature card (needed
    /// to look up the card definition for copying, since the GameObject was
    /// exiled as a cost and may have a new ObjectId in exile).
    /// `activator` is the player who activated the encore ability.
    EncoreAbility {
        source_card_id: Option<crate::state::player::CardId>,
        activator: crate::state::player::PlayerId,
    },
    // ImpendingCounterTrigger (disc 33): migrated to KeywordTrigger { keyword: Impending, data: CounterRemoval }
    // VanishingCounterTrigger (disc 37) and VanishingSacrificeTrigger (disc 38):
    // migrated to KeywordTrigger { keyword: Vanishing, data: CounterRemoval/CounterSacrifice }
    // FadingTrigger (disc 39): migrated to KeywordTrigger { keyword: Fading, data: CounterRemoval }
    // EchoTrigger (disc 40): migrated to KeywordTrigger { keyword: Echo, data: UpkeepCost }
    // CumulativeUpkeepTrigger (disc 41): migrated to KeywordTrigger { keyword: CumulativeUpkeep, data: UpkeepCost }
    /// CR 702.57a: Forecast activated ability on the stack.
    ///
    /// The source card remains in the player's hand. The effect is captured
    /// at activation time from the card definition's `AbilityDefinition::Forecast`.
    ///
    /// When this ability resolves, execute the embedded effect.
    /// If the effect has targets, validate them at resolution time (CR 608.2b).
    ///
    /// Discriminant 43.
    ForecastAbility {
        source_object: ObjectId,
        embedded_effect: Box<crate::cards::card_definition::Effect>,
    },
    /// CR 702.97a: Scavenge activated ability on the stack.
    ///
    /// When this ability resolves: put `power_snapshot` +1/+1 counters on the
    /// target creature. The card was already exiled as cost; `power_snapshot`
    /// is the card's power as it last existed in the graveyard (Varolz ruling
    /// 2013-04-15). The target creature is stored in the StackObject's `targets`
    /// field.
    ///
    /// Discriminant 45.
    ScavengeAbility {
        source_card_id: Option<crate::state::player::CardId>,
        power_snapshot: u32,
    },
    /// CR 207.2c: Bloodrush activated ability on the stack.
    ///
    /// The source card has already been discarded (moved to graveyard as cost
    /// at activation time — CR 602.2b). If this is countered (e.g., by Stifle),
    /// the card remains in the graveyard.
    ///
    /// At resolution: apply +power_boost/+toughness_boost to `target_creature`
    /// (and optionally grant `grants_keyword`) until end of turn, but only if
    /// the target is still a legal creature on the battlefield and still
    /// registered as an attacker in CombatState (CR 608.2b).
    ///
    /// Discriminant 51.
    BloodrushAbility {
        /// The ObjectId the source card had before it was discarded as cost.
        /// Used for event attribution only; the card is already in the graveyard.
        source_object: ObjectId,
        /// The attacking creature to pump.
        target_creature: ObjectId,
        /// +N to power until end of turn (Layer 7c).
        power_boost: i32,
        /// +M to toughness until end of turn (Layer 7c).
        toughness_boost: i32,
        /// Optional keyword to grant until end of turn (Layer 6).
        grants_keyword: Option<KeywordAbility>,
    },
    /// CR 702.171a: Saddle activated ability on the stack.
    ///
    /// The saddle cost has already been paid (saddling creatures tapped at activation
    /// time — CR 602.2b). If this is countered (e.g., by Stifle), the creatures remain
    /// tapped but the Mount is NOT saddled.
    ///
    /// At resolution: set `is_saddled = true` on `source_object` (the Mount), but only
    /// if the Mount is still on the battlefield (CR 608.2b: fizzle-like behavior if not).
    ///
    /// Discriminant 55.
    SaddleAbility {
        /// The ObjectId of the Mount being saddled.
        source_object: ObjectId,
    },
    /// CR 702.140a / CR 729.2: A mutating creature spell on the stack.
    ///
    /// When a spell is cast for its mutate cost targeting a non-Human creature
    /// the caster owns, it becomes a `MutatingCreatureSpell` rather than a plain
    /// `Spell`. On resolution:
    /// - If the target is still legal (CR 702.140b), the spell merges with the
    ///   target permanent (CR 729.2). The card is absorbed into the target's
    ///   `merged_components` list; no ETB triggers fire (CR 729.2c).
    /// - If the target is no longer legal, the spell resolves as a normal creature
    ///   spell (enters the battlefield as though not mutating — CR 702.140b).
    ///
    /// `source_object`: the ObjectId of the spell card in the Stack zone.
    /// `target`: the ObjectId of the non-Human creature being mutated onto.
    ///
    /// Discriminant 59.
    MutatingCreatureSpell {
        /// The ObjectId of the card in the Stack zone (the mutating spell itself).
        source_object: ObjectId,
        /// The ObjectId of the target non-Human creature on the battlefield.
        target: ObjectId,
    },
    /// CR 701.27a / CR 712.18: Transform trigger — a triggered ability that causes
    /// a permanent to transform. Used for card-defined triggers like Delver of Secrets
    /// ("At the beginning of your upkeep, if there's an instant or sorcery on top of
    /// your library, transform Delver of Secrets.").
    ///
    /// At resolution: check the `permanent` is still on the battlefield, check CR 701.27d
    /// (can't transform to instant/sorcery back face), check CR 701.27f (already transformed
    /// since ability was put on the stack — ignored if so), then flip `is_transformed`.
    ///
    /// Discriminant 60.
    TransformTrigger {
        /// The ObjectId of the permanent to transform.
        permanent: ObjectId,
        /// Timestamp when this trigger was put on the stack (for CR 701.27f guard).
        ability_timestamp: u64,
    },
    /// CR 702.167a: Craft activated ability on the stack.
    ///
    /// The source permanent and material objects have already been exiled as cost.
    /// At resolution: if the source card is still in exile (CR 400.7 — may have been
    /// moved), return it to the battlefield transformed (with `is_transformed = true`).
    /// If the card has no back_face, it stays in exile (non-DFC craft guard).
    ///
    /// Discriminant 61.
    CraftAbility {
        /// The CardId of the source card (needed for registry lookup after exile move).
        source_card_id: Option<crate::state::player::CardId>,
        /// The ObjectId of the exiled source card (used to find it in exile zone).
        exiled_source: ObjectId,
        /// ObjectIds of the material cards exiled as cost (for CR 702.167c tracking).
        material_ids: Vec<ObjectId>,
        /// The activating player (becomes controller of the returned permanent).
        activator: PlayerId,
    },
    /// CR 702.145b/f: Daybound or Nightbound immediate transform trigger.
    ///
    /// Not a true "trigger" in the stack sense (CR 702.145c/f says it happens
    /// "immediately and isn't a state-based action"), but modeled as a stack object
    /// to preserve APNAP ordering and allow Stifle-style counter interactions.
    ///
    /// At resolution: check the permanent is still on the battlefield; check that
    /// the transform is still required (day/night state matches); transform it.
    ///
    /// Discriminant 62.
    DayboundTransformTrigger {
        /// The permanent to transform (daybound: front→back on night; nightbound: back→front on day).
        permanent: ObjectId,
    },
    /// CR 708.8: "When this permanent is turned face up" triggered ability.
    ///
    /// Fires when a face-down permanent (morph, megamorph, disguise, manifest, or cloak)
    /// is turned face up via `Command::TurnFaceUp`. Unlike ETB triggers, these DO fire
    /// when a permanent is turned face up — the permanent already entered the battlefield
    /// face-down, so ETB was suppressed at that time (CR 708.3).
    ///
    /// At resolution: look up the permanent's `WhenTurnedFaceUp` triggered ability in
    /// the CardDefinition and execute its effect.
    ///
    /// Discriminant 63.
    TurnFaceUpTrigger {
        /// The permanent that was turned face up.
        permanent: ObjectId,
        /// CardId for registry lookup after zone-change (CR 400.7 resilience).
        source_card_id: Option<crate::state::player::CardId>,
        /// Index into the CardDefinition abilities list for the specific WhenTurnedFaceUp
        /// ability that triggered. Cards may have multiple WhenTurnedFaceUp abilities
        /// (rare but rules-legal); each gets its own TurnFaceUpTrigger SOK.
        ability_index: usize,
    },
    /// Consolidated keyword trigger (replaces many one-off trigger variants).
    ///
    /// Discriminant 64.
    KeywordTrigger {
        source_object: ObjectId,
        keyword: KeywordAbility,
        data: TriggerData,
    },
    /// CR 309.4c: A room ability triggered when the venture marker entered a room.
    ///
    /// Room abilities are triggered abilities of the form "When you move your venture
    /// marker into this room, [effect]." They go on the stack and resolve through the
    /// normal stack resolution path. The dungeon itself lives in the command zone —
    /// there is no `source_object` in the traditional sense.
    ///
    /// At resolution (`resolution.rs`): look up the `DungeonDef` via `get_dungeon(dungeon)`,
    /// get the `RoomDef` at `room` index, call `room.effect()` to get the `Effect`, and
    /// execute it with `owner` as the controller.
    ///
    /// Discriminant 65.
    RoomAbility {
        /// The player who controls this room ability (the venturer).
        owner: crate::state::player::PlayerId,
        /// Which dungeon the room is in.
        dungeon: crate::state::dungeon::DungeonId,
        /// The room index in the dungeon's room list.
        room: crate::state::dungeon::RoomIndex,
    },
    /// CR 701.54c: Ring-bearer triggered ability (ring level 2, 3, or 4).
    ///
    /// Fires when the appropriate ring event occurs (attacker declared for level 2,
    /// blocker declared for level 3, combat damage dealt for level 4).
    ///
    /// At resolution: execute the embedded effect with `controller` as the controller.
    ///
    /// Discriminant 66.
    RingAbility {
        /// The ring-bearer creature that caused this trigger.
        source_object: ObjectId,
        /// The effect to execute when this resolves.
        effect: Box<crate::cards::card_definition::Effect>,
        /// The player who controls the ring.
        controller: crate::state::player::PlayerId,
    },
    /// CR 716.2a: Class level-up activated ability on the stack.
    ///
    /// Level-up is an activated ability that uses the stack and can be responded
    /// to. ("Gaining a level is a normal activated ability. It uses the stack and
    /// can be responded to." — Druid Class rulings.)
    ///
    /// The mana cost was already paid at activation time (CR 602.2b).
    /// At resolution: set `class_level = target_level` on `source_object` and
    /// register any continuous effects declared at that level.
    ///
    /// If `source_object` is no longer on the battlefield at resolution time,
    /// the ability does nothing (CR 608.2b analog for non-targeted abilities).
    ///
    /// Discriminant 68.
    ClassLevelAbility {
        /// The ObjectId of the Class permanent that is leveling up.
        source_object: ObjectId,
        /// The level the Class is leveling up to.
        target_level: u32,
    },
    /// CR 603.7: A delayed triggered ability fires.
    ///
    /// Created when a delayed trigger's timing condition is met (e.g., "at next end step",
    /// "when source leaves"). On resolution, executes the stored `action` on `target`.
    ///
    /// Discriminant 69.
    DelayedActionTrigger {
        /// The source that originally created this delayed trigger.
        source_object: ObjectId,
        /// The object to act on.
        target: ObjectId,
        /// What to do when this resolves.
        action: super::stubs::DelayedTriggerAction,
    },
}
