//! Stack object types: spells and abilities on the stack (CR 405).
//!
//! The stack is an ordered zone (LIFO). When a spell is cast or an ability
//! is activated/triggered, a StackObject is pushed onto the stack.
//! Resolution pops the top object off the stack (CR 608.1).
//!
//! For spells, the corresponding card has moved to `ZoneId::Stack` and appears
//! as a `GameObject` there. For abilities, no corresponding `GameObject` exists
//! in the Stack zone — the `StackObject` alone represents the ability on the stack.

use serde::{Deserialize, Serialize};

use super::game_object::ObjectId;
use super::player::PlayerId;
use super::targeting::SpellTarget;

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

    /// A triggered ability (CR 603).
    ///
    /// The `source_object` may be in any zone (it triggered from wherever it
    /// was when the trigger condition was met). `ability_index` identifies
    /// which ability triggered.
    TriggeredAbility {
        source_object: ObjectId,
        ability_index: usize,
    },

    /// CR 702.85a: Cascade triggered ability on the stack.
    ///
    /// Cascade is a triggered ability that triggers when the cascade spell is
    /// cast. When this trigger resolves, the cascade procedure runs: exile
    /// cards until a qualifying nonland card with mana value strictly less than
    /// `spell_mana_value` is found, cast it for free, put the rest on the
    /// bottom of the library.
    ///
    /// `spell_mana_value` is captured at trigger time (when the cascade spell
    /// is cast) because continuous effects could change the mana value later.
    CascadeTrigger {
        source_object: ObjectId,
        spell_mana_value: u32,
    },

    /// CR 702.40a: Storm triggered ability on the stack.
    ///
    /// Storm is a triggered ability that triggers when the storm spell is cast.
    /// When this trigger resolves, `storm_count` copies of the original spell
    /// are created on the stack. Storm copies are NOT cast (CR 702.40c) —
    /// they do not trigger "whenever you cast a spell."
    ///
    /// `storm_count` and `original_stack_id` are captured at trigger time
    /// (when the storm spell is cast).
    StormTrigger {
        source_object: ObjectId,
        original_stack_id: ObjectId,
        storm_count: u32,
    },

    /// CR 702.74a: Evoke sacrifice trigger on the stack.
    ///
    /// When an evoked permanent enters the battlefield, this trigger fires:
    /// "When this permanent enters, if its evoke cost was paid, its controller
    /// sacrifices it." Resolves to sacrifice the source permanent.
    ///
    /// If the source has left the battlefield by resolution time (blinked,
    /// bounced, etc.), the trigger does nothing per CR 400.7 — the source
    /// is a new object and is no longer the evoked permanent.
    EvokeSacrificeTrigger { source_object: ObjectId },
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
    /// CR 702.84a: Unearth delayed triggered ability on the stack.
    ///
    /// "Exile [this permanent] at the beginning of the next end step."
    /// This is a delayed triggered ability created when the unearthed permanent
    /// enters the battlefield. It fires at the beginning of the next end step.
    ///
    /// If the source has left the battlefield by resolution time (CR 400.7),
    /// the trigger does nothing. If countered (e.g., by Stifle), the permanent
    /// stays on the battlefield but the replacement effect still applies.
    UnearthTrigger { source_object: ObjectId },
    /// CR 702.110a: Exploit triggered ability on the stack.
    ///
    /// "When this creature enters, you may sacrifice a creature."
    /// When this trigger resolves, the controller may sacrifice a creature they
    /// control. The default (deterministic, no interactive choice) is to decline.
    ///
    /// If the source has left the battlefield by resolution time (CR 400.7),
    /// the trigger does nothing (no creature to exploit with).
    ExploitTrigger { source_object: ObjectId },
    /// CR 702.43a: Modular triggered ability on the stack.
    ///
    /// "When this permanent is put into a graveyard from the battlefield,
    /// you may put a +1/+1 counter on target artifact creature for each
    /// +1/+1 counter on this permanent."
    ///
    /// `counter_count` is the number of +1/+1 counters on the creature at
    /// death time (last-known information from pre_death_counters — Arcbound
    /// Worker ruling 2006-09-25). The target artifact creature is in
    /// `StackObject.targets[0]`.
    ///
    /// If no legal artifact creature target exists at trigger time,
    /// the trigger is not placed on the stack (CR 603.3d).
    ModularTrigger {
        source_object: ObjectId,
        counter_count: u32,
    },

    /// CR 702.100a: Evolve trigger on the stack.
    ///
    /// When a creature with evolve sees another creature its controller controls
    /// enter the battlefield with greater power and/or toughness, this trigger
    /// fires. The `entering_creature` field carries the ObjectId of the creature
    /// that entered, needed for the resolution-time intervening-if re-check
    /// (CR 603.4 — compare entering creature P/T vs source P/T at resolution).
    ///
    /// If the entering creature left the battlefield before resolution, use
    /// last-known information for the P/T comparison (ruling 2013-04-15).
    EvolveTrigger {
        source_object: ObjectId,
        entering_creature: ObjectId,
    },

    /// CR 702.116a: Myriad triggered ability on the stack.
    ///
    /// "Whenever this creature attacks, for each opponent other than defending
    /// player, you may create a token that's a copy of this creature that's
    /// tapped and attacking that player."
    ///
    /// `source_object` is the attacking creature (the one with myriad).
    /// `defending_player` is the player being attacked (the one NOT to copy for).
    ///
    /// When this trigger resolves: for each opponent of the source's controller
    /// who is NOT `defending_player`, create a token copy of the source that is
    /// tapped, attacking that opponent, and tagged `myriad_exile_at_eoc = true`.
    /// Tokens are exiled at end of combat by `end_combat()` in turn_actions.rs.
    ///
    /// CR 702.116b: Multiple instances trigger separately (each creates its own
    /// set of copies).
    MyriadTrigger {
        source_object: ObjectId,
        defending_player: crate::state::player::PlayerId,
    },

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
    /// CR 702.75a: Hideaway ETB triggered ability on the stack.
    ///
    /// "When this permanent enters, look at the top N cards of your library.
    /// Exile one of them face down and put the rest on the bottom of your
    /// library in a random order."
    ///
    /// `source_object` is the Hideaway permanent's ObjectId on the battlefield.
    /// `hideaway_count` is N (how many cards to look at).
    ///
    /// When this trigger resolves:
    /// 1. Take the top N cards from the controller's library.
    /// 2. Exile one face-down (deterministic: exile the top card).
    /// 3. Set `exiled_by_hideaway = Some(source_object)` on the exiled card.
    /// 4. Put the rest on the bottom in a random order (seeded shuffle).
    ///
    /// CR 603.3: The trigger goes on the stack and can be countered.
    /// If the source has left the battlefield by resolution time (CR 400.7),
    /// the trigger still resolves (it is already on the stack).
    HideawayTrigger {
        source_object: ObjectId,
        hideaway_count: u32,
    },
    /// CR 702.124j: Partner With ETB triggered ability on the stack.
    ///
    /// "When this permanent enters, target player may search their library
    /// for a card named [name], reveal it, put it into their hand, then
    /// shuffle."
    ///
    /// `source_object` is the permanent with "Partner with [name]" on the
    /// battlefield.
    /// `partner_name` is the exact name of the card to search for.
    /// `target_player` is the targeted player who will search their library.
    ///
    /// When this trigger resolves:
    /// 1. The target player searches their library for a card with the exact
    ///    name `partner_name`.
    /// 2. If found, reveal it and put it into their hand.
    /// 3. Shuffle the target player's library.
    ///
    /// CR 603.3: The trigger goes on the stack and can be countered.
    /// If the source has left the battlefield by resolution time (CR 400.7),
    /// the trigger still resolves (it is already on the stack).
    PartnerWithTrigger {
        source_object: ObjectId,
        partner_name: String,
        target_player: crate::state::player::PlayerId,
    },
    /// CR 702.115a: Ingest triggered ability on the stack.
    ///
    /// "Whenever this creature deals combat damage to a player, that player
    /// exiles the top card of their library."
    ///
    /// `source_object` is the creature with ingest on the battlefield.
    /// `target_player` is the player who was dealt combat damage (whose library
    /// will be exiled from).
    ///
    /// When this trigger resolves:
    /// 1. Check if the target player has cards in their library.
    /// 2. If yes, exile the top card face-up.
    /// 3. If no, do nothing (ruling 2015-08-25).
    ///
    /// CR 603.10: The source creature must be on the battlefield when the trigger
    /// fires, but does NOT need to be on the battlefield at resolution time
    /// (the trigger is already on the stack).
    IngestTrigger {
        source_object: ObjectId,
        target_player: crate::state::player::PlayerId,
    },
}
