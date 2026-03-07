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

use super::game_object::{ManaCost, ObjectId};
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
    /// CR 702.42a: If true, this spell was cast with its entwine cost paid.
    /// At resolution, all modes of the modal spell execute in printed order (CR 702.42b).
    ///
    /// Propagated to copies per CR 707.10 (copies copy all decisions including modes and
    /// additional costs).
    #[serde(default)]
    pub was_entwined: bool,
    /// CR 702.120a: Number of additional modes paid for via escalate. 0 = only mode[0]
    /// executes at resolution. N = modes 0..=N execute. Escalate cost was paid N times.
    ///
    /// Propagated to copies per CR 707.10 (copies copy all decisions including modes and
    /// additional costs).
    #[serde(default)]
    pub escalate_modes_paid: u32,
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
    /// CR 702.82a: Creatures to sacrifice when this permanent enters the battlefield.
    /// Populated from CastSpell.devour_sacrifices at cast time; consumed at resolution
    /// time in resolution.rs for the Devour ETB replacement.
    ///
    /// Empty vec = player chose not to sacrifice (zero devour). The sacrifice and
    /// counter placement happen during ETB resolution, not at cast time.
    #[serde(default)]
    pub devour_sacrifices: Vec<ObjectId>,
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
    /// CR 702.25a: Flanking triggered ability on the stack.
    ///
    /// "Whenever this creature becomes blocked by a creature without flanking,
    /// the blocking creature gets -1/-1 until end of turn."
    ///
    /// `source_object` is the creature with flanking (the attacker).
    /// `blocker_id` is the blocking creature that will receive -1/-1.
    ///
    /// When this trigger resolves:
    /// 1. Check if the blocker is still on the battlefield (CR 400.7).
    /// 2. If yes, register a ContinuousEffect with ModifyBoth(-1) in Layer 7c
    ///    (PtModify) targeting SingleObject(blocker_id) with UntilEndOfTurn duration.
    /// 3. If the blocker left the battlefield, do nothing (trigger fizzles).
    ///
    /// CR 702.25b: Multiple instances trigger separately (each creates its own
    /// trigger with the same blocker_id).
    FlankingTrigger {
        source_object: ObjectId,
        blocker_id: ObjectId,
    },
    /// CR 702.23a: Rampage N triggered ability on the stack.
    ///
    /// "Whenever this creature becomes blocked, it gets +N/+N until end of
    /// turn for each creature blocking it beyond the first."
    ///
    /// When this trigger resolves:
    /// 1. Count blockers for the source attacker from `state.combat`.
    /// 2. Compute bonus = (blocker_count - 1) * rampage_n.
    /// 3. If bonus > 0 and source is on the battlefield, apply +bonus/+bonus as
    ///    two ContinuousEffects (UntilEndOfTurn) in Layer 7c (PtModify).
    ///
    /// CR 702.23b: Bonus calculated once at resolution. Later blocker
    /// changes do not affect it.
    /// CR 603.10: Source need not be on battlefield at resolution time
    /// (trigger is already on the stack), but bonus only applies if source
    /// is still on the battlefield.
    /// CR 702.23c: Multiple instances trigger separately.
    RampageTrigger {
        source_object: ObjectId,
        rampage_n: u32,
    },
    /// CR 702.39a: Provoke triggered ability on the stack.
    ///
    /// "Whenever this creature attacks, you may have target creature defending
    /// player controls untap and block this creature this combat if able."
    ///
    /// `source_object` is the creature with provoke (the attacker).
    /// `provoked_creature` is the target creature (defending player controls).
    ///
    /// When this trigger resolves:
    /// 1. Check if the provoked creature is still on the battlefield
    ///    (target legality, CR 608.2b).
    /// 2. Untap the provoked creature (CR 702.39a: "untap that creature").
    /// 3. Add a forced-block requirement to `CombatState::forced_blocks`:
    ///    provoked_creature must block source_object "if able" (CR 509.1c).
    ///
    /// If the provoked creature has left the battlefield, the trigger fizzles.
    ///
    /// CR 702.39b: Multiple instances each trigger separately, each with their
    /// own `ProvokeTrigger` stack object.
    ProvokeTrigger {
        source_object: ObjectId,
        provoked_creature: ObjectId,
    },
    /// CR 702.112a: Renown N triggered ability on the stack.
    ///
    /// "When this creature deals combat damage to a player, if it isn't renowned,
    /// put N +1/+1 counters on it and it becomes renowned."
    ///
    /// `source_object` is the creature with renown.
    /// `renown_n` is the number of +1/+1 counters to place.
    ///
    /// When this trigger resolves:
    /// 1. Re-check the intervening-if (CR 603.4): source must still be on the
    ///    battlefield AND not yet renowned.
    /// 2. If check passes: place N +1/+1 counters on source and set is_renowned.
    /// 3. If source left the battlefield before resolution, do nothing
    ///    (Ruling 2015-06-22).
    ///
    /// CR 702.112c: Multiple instances each create their own RenownTrigger.
    /// The first to resolve sets is_renowned; subsequent triggers fail the
    /// intervening-if (CR 603.4) and do nothing.
    RenownTrigger {
        source_object: ObjectId,
        renown_n: u32,
    },
    /// CR 702.121a: Melee triggered ability on the stack.
    ///
    /// "Whenever this creature attacks, it gets +1/+1 until end of turn for
    /// each opponent you attacked with a creature this combat."
    ///
    /// When this trigger resolves:
    /// 1. Count distinct opponents (players) targeted by any attacker in
    ///    `state.combat.attackers` (only `AttackTarget::Player` entries count,
    ///    NOT planeswalkers -- ruling 2016-08-23).
    /// 2. If count > 0 and source is on the battlefield, apply +count/+count
    ///    as a ContinuousEffect (UntilEndOfTurn) in Layer 7c (PtModify).
    ///
    /// CR 702.121b: Multiple instances trigger separately (each creates its
    /// own MeleeTrigger; each computes the bonus independently).
    ///
    /// The bonus is computed at resolution time (ruling 2016-08-23: "You
    /// determine the size of the bonus as the melee ability resolves").
    /// `state.combat.attackers` retains all declared attackers even if they
    /// leave the battlefield, so the count is stable.
    MeleeTrigger { source_object: ObjectId },
    /// CR 702.70a: Poisonous N triggered ability on the stack.
    ///
    /// "Whenever this creature deals combat damage to a player, that player
    /// gets N poison counters."
    ///
    /// `source_object` is the creature with poisonous on the battlefield.
    /// `target_player` is the player who was dealt combat damage (who receives
    /// the poison counters).
    /// `poisonous_n` is the number of poison counters to give.
    ///
    /// When this trigger resolves:
    /// 1. Give `target_player` exactly `poisonous_n` poison counters.
    /// 2. Emit `PoisonCountersGiven` event (reusing the existing Infect event).
    ///
    /// CR 702.70b: Multiple instances trigger separately (each creates its own
    /// trigger with its own N value).
    ///
    /// CR 603.10: The source creature does NOT need to be on the battlefield
    /// at resolution time (the trigger is already on the stack). The poison
    /// counters are given regardless of the source's current state.
    PoisonousTrigger {
        source_object: ObjectId,
        target_player: crate::state::player::PlayerId,
        poisonous_n: u32,
    },
    /// CR 702.154a: Enlist triggered ability on the stack.
    ///
    /// "When you [tap a creature for enlist], this creature gets +X/+0 until
    /// end of turn, where X is the tapped creature's power."
    ///
    /// `source_object` is the attacking creature with Enlist.
    /// `enlisted_creature` is the creature that was tapped to pay the
    /// enlist cost.
    ///
    /// When this trigger resolves:
    /// 1. Check if the source (enlisting) creature is still on the battlefield.
    /// 2. Read the enlisted creature's power via calculate_characteristics
    ///    (if still on battlefield) or raw characteristics (if departed).
    /// 3. If the source creature is alive and power != 0, register a
    ///    ContinuousEffect with ModifyPower(X) in Layer 7c (PtModify)
    ///    targeting SingleObject(source_object) with UntilEndOfTurn duration.
    /// 4. If the source left the battlefield, do nothing (CR 400.7).
    ///
    /// CR 702.154d: Multiple instances each create their own EnlistTrigger
    /// with different `enlisted_creature` values.
    EnlistTrigger {
        source_object: ObjectId,
        enlisted_creature: ObjectId,
    },
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
    /// CR 702.141a: Encore delayed triggered ability on the stack.
    ///
    /// "Sacrifice them at the beginning of the next end step."
    /// This is a delayed triggered ability created when the encore tokens
    /// are created. Each token gets its own sacrifice trigger.
    ///
    /// When this trigger resolves:
    /// 1. Check if the token is still on the battlefield (CR 400.7).
    /// 2. Check if the token is still controlled by the encore activator
    ///    (ruling 2020-11-10: can't sacrifice if under another player's control).
    /// 3. If both checks pass, sacrifice the token (move to graveyard via
    ///    replacement effects).
    ///
    /// If countered (e.g., by Stifle), the token stays on the battlefield.
    EncoreSacrificeTrigger {
        source_object: ObjectId,
        activator: crate::state::player::PlayerId,
    },
    /// CR 702.109a: Dash delayed triggered ability on the stack.
    ///
    /// "Return the permanent this spell becomes to its owner's hand at the
    /// beginning of the next end step."
    /// This is a delayed triggered ability created when the dash spell resolves
    /// and the permanent enters the battlefield.
    ///
    /// When this trigger resolves:
    /// 1. Check if the source is still on the battlefield (CR 400.7).
    /// 2. If yes, return it to its owner's hand.
    /// 3. If no (died, blinked, bounced), do nothing.
    ///
    /// If countered (e.g., by Stifle), the permanent stays on the battlefield
    /// but retains haste (the haste is a static ability linked to was_dashed,
    /// not to this trigger).
    DashReturnTrigger { source_object: ObjectId },
    /// CR 702.152a: Blitz delayed triggered ability on the stack.
    ///
    /// "Sacrifice the permanent this spell becomes at the beginning of the
    /// next end step."
    /// This is a delayed triggered ability created when the blitz spell resolves
    /// and the permanent enters the battlefield.
    ///
    /// When this trigger resolves:
    /// 1. Check if the source is still on the battlefield (CR 400.7).
    /// 2. If yes, sacrifice it (move to graveyard, which fires CreatureDied).
    /// 3. If no (already died, blinked, bounced), do nothing.
    ///
    /// If countered (e.g., by Stifle), the permanent stays on the battlefield
    /// but retains haste and the draw-on-death trigger (those are static
    /// abilities linked to cast_alt_cost, not to this trigger -- CR 702.152a).
    BlitzSacrificeTrigger { source_object: ObjectId },
    /// CR 702.153a: Casualty triggered ability on the stack.
    ///
    /// "When you cast this spell, if a casualty cost was paid for it, copy it."
    /// This trigger fires after the casualty cost has been paid and the spell
    /// is on the stack. When this trigger resolves, one copy of the original
    /// spell is created on the stack above the original (LIFO order means the
    /// copy resolves first).
    ///
    /// `source_object` is the ObjectId of the card now in ZoneId::Stack.
    /// `original_stack_id` is the id of the spell StackObject to copy.
    ///
    /// The copy is NOT cast (CR 707.10 / ruling 2022-04-29) — it does not
    /// trigger "whenever you cast a spell" abilities, and it does not
    /// increment `spells_cast_this_turn`.
    ///
    /// CR 702.153b: Multiple instances of casualty each trigger separately
    /// and produce their own copy. (Not yet supported — no cards have multiple
    /// instances in initial implementation.)
    CasualtyTrigger {
        source_object: ObjectId,
        original_stack_id: ObjectId,
    },

    /// CR 702.176a: Impending end-step counter-removal trigger.
    ///
    /// "At the beginning of your end step, if this permanent's impending cost
    /// was paid and it has a time counter on it, remove a time counter from it."
    ///
    /// When this trigger resolves:
    /// 1. Re-check intervening-if: permanent must still be on battlefield,
    ///    must have `cast_alt_cost == Some(AltCostKind::Impending)`, and must
    ///    have at least one time counter (CR 603.4).
    /// 2. If yes, remove one time counter.
    /// 3. If no (conditions no longer met), do nothing.
    ///
    /// Unlike Suspend, there is no follow-up trigger when the last counter is
    /// removed -- the permanent simply becomes a creature because the Layer 4
    /// type-removal effect in calculate_characteristics stops applying.
    ImpendingCounterTrigger {
        source_object: ObjectId,
        impending_permanent: ObjectId,
    },

    /// CR 702.56a: Replicate trigger — "When you cast this spell, if a replicate cost
    /// was paid for it, copy it for each time its replicate cost was paid."
    ///
    /// This is a triggered ability (CR 702.56a). It goes on the stack above the original
    /// spell and resolves through normal priority.
    ///
    /// Copies created by replicate are NOT cast (ruling 2024-01-12 for Shattering Spree)
    /// — they do not trigger "whenever you cast a spell" abilities and do not increment
    /// `spells_cast_this_turn`.
    ///
    /// `replicate_count` stores the number of times the replicate cost was paid, which
    /// determines the number of copies created on resolution.
    ReplicateTrigger {
        source_object: ObjectId,
        original_stack_id: ObjectId,
        replicate_count: u32,
    },
    /// CR 702.69a: Gravestorm triggered ability — fires when the spell with gravestorm
    /// is cast. Resolves to create `gravestorm_count` copies of `original_stack_id`.
    ///
    /// Copies created by gravestorm are NOT cast (CR 702.69a / CR 707.10) — they do not
    /// trigger "whenever you cast a spell" abilities and do not increment
    /// `spells_cast_this_turn`.
    ///
    /// `gravestorm_count` is captured at trigger-creation time (at cast) from
    /// `GameState::permanents_put_into_graveyard_this_turn` to prevent changes
    /// between cast and resolution from affecting the count.
    GravestormTrigger {
        source_object: ObjectId,
        original_stack_id: ObjectId,
        gravestorm_count: u32,
    },
    /// CR 702.63a: "At the beginning of your upkeep, if this permanent has a time counter
    /// on it, remove a time counter from it." Discriminant 37.
    VanishingCounterTrigger {
        source_object: ObjectId,
        vanishing_permanent: ObjectId,
    },
    /// CR 702.63a: "When the last time counter is removed from this permanent, sacrifice it."
    /// Discriminant 38.
    VanishingSacrificeTrigger {
        source_object: ObjectId,
        vanishing_permanent: ObjectId,
    },
    /// CR 702.32a: "At the beginning of your upkeep, remove a fade counter from
    /// this permanent. If you can't, sacrifice the permanent." Discriminant 39.
    FadingTrigger {
        source_object: ObjectId,
        fading_permanent: ObjectId,
    },
    /// CR 702.30a: "At the beginning of your upkeep, if this permanent came under
    /// your control since the beginning of your last upkeep, sacrifice it unless
    /// you pay [cost]." Discriminant 40.
    EchoTrigger {
        source_object: ObjectId,
        echo_permanent: ObjectId,
        echo_cost: ManaCost,
    },
    /// CR 702.24a: "At the beginning of your upkeep, if this permanent is on the
    /// battlefield, put an age counter on this permanent. Then you may pay [cost]
    /// for each age counter on it. If you don't, sacrifice it."
    /// Discriminant 41.
    CumulativeUpkeepTrigger {
        source_object: ObjectId,
        cu_permanent: ObjectId,
        per_counter_cost: crate::state::types::CumulativeUpkeepCost,
    },
    /// CR 702.59a: Recover trigger. Fires when a creature enters the card owner's
    /// graveyard from the battlefield and the Recover card is also in that graveyard.
    ///
    /// On resolution: check if recover_card is still in the graveyard (CR 400.7).
    /// If yes, emit RecoverPaymentRequired and add to pending_recover_payments.
    /// The game pauses until a Command::PayRecover is received.
    ///
    /// Discriminant 42.
    RecoverTrigger {
        source_object: ObjectId,
        /// The ObjectId of the Recover card in the graveyard (the trigger source).
        recover_card: ObjectId,
        /// The mana cost to pay for Recover.
        recover_cost: ManaCost,
    },
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
    /// CR 702.58a: Graft triggered ability on the stack.
    /// "Whenever another creature enters, if this permanent has a +1/+1 counter
    /// on it, you may move a +1/+1 counter from this permanent onto that creature."
    ///
    /// At resolution: re-check intervening-if (source has +1/+1 counter, CR 603.4),
    /// then move one counter from source to entering creature.
    ///
    /// Discriminant 44.
    GraftTrigger {
        source_object: ObjectId,
        entering_creature: ObjectId,
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
    /// CR 702.165a: Backup triggered ability on the stack.
    ///
    /// "When this creature enters, put N +1/+1 counters on target creature.
    /// If that's another creature, it also gains the non-backup abilities of
    /// this creature printed below this one until end of turn."
    ///
    /// At resolution: place N +1/+1 counters on the target creature. If the
    /// target is a different creature from the source, register a Layer 6
    /// UntilEndOfTurn continuous effect granting the stored keyword abilities.
    ///
    /// Discriminant 46.
    BackupTrigger {
        source_object: ObjectId,
        target_creature: ObjectId,
        counter_count: u32,
        /// Keyword abilities to grant (determined at trigger time per CR 702.165d).
        /// Empty if targeting self (CR 702.165a: "if that's another creature").
        abilities_to_grant: Vec<crate::state::types::KeywordAbility>,
    },
    /// CR 702.72a: Champion ETB trigger on the stack.
    ///
    /// "When this permanent enters, sacrifice it unless you exile another
    /// [object] you control." When this resolves, the engine auto-selects
    /// the first qualifying permanent to exile (simplified -- no player choice
    /// for now). If none exists, the champion is sacrificed.
    ///
    /// Discriminant 47.
    ChampionETBTrigger {
        source_object: ObjectId,
        champion_filter: crate::state::types::ChampionFilter,
    },
    /// CR 702.72a: Champion LTB trigger on the stack.
    ///
    /// "When this permanent leaves the battlefield, return the exiled card
    /// to the battlefield under its owner's control." When this resolves,
    /// the engine checks if the exiled card is still in exile; if so, it
    /// moves it to the battlefield under its owner's control.
    ///
    /// Discriminant 48.
    ChampionLTBTrigger {
        source_object: ObjectId,
        exiled_card: ObjectId,
    },
    /// CR 702.95a: Soulbond ETB trigger on the stack.
    ///
    /// Fired when a creature with soulbond enters (SelfETB) OR when another
    /// creature enters while an unpaired soulbond creature is on the battlefield
    /// (OtherETB). In both cases, source_object is the soulbond creature.
    ///
    /// At resolution (CR 702.95c): both source_object and pair_target must still
    /// be on the battlefield as creatures controlled by the same player and both
    /// unpaired, otherwise fizzle.
    ///
    /// Discriminant 49.
    SoulbondTrigger {
        /// The creature with the soulbond ability.
        source_object: ObjectId,
        /// The creature to pair with (auto-selected at trigger time).
        pair_target: ObjectId,
    },
}
