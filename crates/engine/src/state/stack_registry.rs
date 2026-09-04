//! `StackObjectKind` card-ownership classification (PB-DX25, `OOS-SIM3-5`).
//!
//! CR 701.6a: "A countered spell is put into its owner's graveyard." That
//! sentence is about a CARD. Which `StackObjectKind`s own a card sitting in
//! `ZoneId::Stack`, and which don't, is not a property you can read off a
//! variant's NAME — `casting.rs::handle_cast_spell` moves a spell's card into
//! `ZoneId::Stack` once (`move_object_to_zone(card, ZoneId::Stack)`), and only
//! AFTER that move does it choose between `StackObjectKind::Spell` and
//! `StackObjectKind::MutatingCreatureSpell` (CR 702.140a / CR 729.2), on
//! `cast_with_mutate` alone. So both kinds own a Stack-zone card, and every
//! other kind puts an ability or a trigger on the stack and moves no card there.
//!
//! `Effect::CounterSpell` (`effects/mod.rs`) got this wrong before PB-DX25:
//! its zone-move matched the literal `Spell` variant and fell through every
//! other kind — including `MutatingCreatureSpell` — to a no-op. This module is
//! the fix, made ONCE, exhaustively:
//!
//! [`card_in_stack_zone`] is deliberately exhaustive with **no wildcard arm**.
//! Adding a `StackObjectKind` variant is a compile error here until someone
//! decides which side of this question the new variant is on — the same
//! forcing function SR-5 applies to `KeywordAbility`
//! (`state::keyword_registry::handling`). Guessing from the variant's name is
//! exactly what produced the defect this module exists to have fixed.
//!
//! **This is NOT "is it a spell".** A copy of a spell IS a spell (CR 707.10)
//! and owns no card of its own — `copy.rs` clones the original's `kind`
//! wholesale, so a copy's `source_object` names the ORIGINAL's card, not a
//! card of the copy's own. `casting.rs`'s `is_spell` check for
//! `TargetSpellWithSingleTarget` / `TargetSpellOrAbilityWithSingleTarget`
//! answers that OTHER question ("is this stack object a spell") and must
//! never be re-expressed through this function — doing so would make a copy
//! an illegal target for "target spell", which CR 707.10 forbids. See the
//! comment at that call site (`casting.rs`, near line 6503) for the other
//! half of this note.
//!
//! **Deliberately duplicated**, not delegated to, by
//! `mtg_simulator::invariants::stack_card_of` — the simulator's
//! `check_stack_consistency` exists specifically to catch the ENGINE getting
//! this classification wrong, so it must not simply read the engine's own
//! answer back (a wrong `Some`/`None` here would then make the check agree
//! with the defect and go silent, which is the exact failure mode
//! `check_stack_consistency`'s own history note records). What keeps the two
//! answers honest without coupling them: both are exhaustive with no wildcard
//! (so a new variant is a compile error in BOTH crates independently), and a
//! behavioural cross-check (`crates/simulator/tests/
//! pb_dx25_counter_on_mutate_is_consistent.rs`) proves the two agree on the
//! case that matters by running a real counter-on-mutate game and asserting
//! zero `stack_consistency` violations, rather than by sharing code.

use super::stack::{StackObject, StackObjectKind};
use crate::state::game_object::ObjectId;

/// The card this stack object owns in `ZoneId::Stack`, if it owns one.
///
/// `Some(*source_object)` for `Spell` and `MutatingCreatureSpell` (CR 601.2c /
/// CR 702.140a / CR 729.2 -- one `move_object_to_zone(card, ZoneId::Stack)` at
/// cast time, then a `cast_with_mutate` branch that only picks the kind).
///
/// `None` for every other variant: each puts an ability or a trigger on the
/// stack and moves no card to `ZoneId::Stack`. Several of them DO move a card
/// somewhere at RESOLUTION -- Ninjutsu and the graveyard-recursion abilities
/// (Unearth, Embalm, Eternalize, Encore, Scavenge, Bloodrush) move their
/// source from hand/graveyard/exile to the battlefield or apply it there;
/// Madness/Miracle name a card already sitting in exile/hand -- but none of
/// those destinations is `ZoneId::Stack`, which is the only thing this
/// classification is about. This function answers "does the stack entry
/// itself own a card sitting in the Stack zone right now", not "will
/// resolving this entry eventually touch some card somewhere".
pub fn card_in_stack_zone(kind: &StackObjectKind) -> Option<ObjectId> {
    use StackObjectKind as K;
    match kind {
        // CR 601.2c: a spell's card is moved into ZoneId::Stack as part of
        // casting it.
        K::Spell { source_object } => Some(*source_object),
        // CR 702.140a / CR 729.2: a mutating creature spell is cast down the
        // SAME code path as a plain Spell (casting.rs: one
        // move_object_to_zone, then a cast_with_mutate branch that picks the
        // kind afterwards) -- its card is in ZoneId::Stack exactly like a
        // Spell's is.
        K::MutatingCreatureSpell { source_object, .. } => Some(*source_object),

        // Everything below puts an ability or a trigger on the stack. None of
        // them moves a card into ZoneId::Stack.
        K::ActivatedAbility { .. } => None,
        K::LoyaltyAbility { .. } => None,
        K::TriggeredAbility { .. } => None,
        K::MadnessTrigger { .. } => None,
        K::MiracleTrigger { .. } => None,
        K::UnearthAbility { .. } => None,
        K::SuspendCounterTrigger { .. } => None,
        K::SuspendCastTrigger { .. } => None,
        K::NinjutsuAbility { .. } => None,
        K::EmbalmAbility { .. } => None,
        K::EternalizeAbility { .. } => None,
        K::EncoreAbility { .. } => None,
        K::ForecastAbility { .. } => None,
        K::ScavengeAbility { .. } => None,
        K::BloodrushAbility { .. } => None,
        K::SaddleAbility { .. } => None,
        K::TransformTrigger { .. } => None,
        K::CraftAbility { .. } => None,
        K::DayboundTransformTrigger { .. } => None,
        K::TurnFaceUpTrigger { .. } => None,
        K::KeywordTrigger { .. } => None,
        K::RoomAbility { .. } => None,
        K::RingAbility { .. } => None,
        K::ClassLevelAbility { .. } => None,
        K::DelayedActionTrigger { .. } => None,
    }
}

/// PB-DX52 (`OOS-DX25c-3`): CR 113.7a's **source** of a stack object -- the object that
/// generated the ability, or the card the spell was cast from.
///
/// The sibling of [`card_in_stack_zone`], and the two answer genuinely different
/// questions. `card_in_stack_zone` asks *"does this entry own a card sitting in
/// `ZoneId::Stack` right now"* and returns `None` for every ability, because an ability
/// on the stack owns no card. This asks *"what object is this ability's SOURCE"*
/// (CR 113.7a: *"the source of an ability is the object that generated it"*), which for
/// an ability is a permanent somewhere else -- usually the battlefield.
///
/// # Why this exists, and what it repairs
///
/// `rules::retarget::plan_target_change` derives TWO things from the victim stack object:
/// `source_chars` (CR 702.16b protection -- *"protection from red"* must refuse a red
/// source) and `self_id` (CR 601.2c self-exclusion / `TargetFilter.exclude_self`). Both
/// came from `card_in_stack_zone`, and that was correct only while a `ChangeTargets`
/// victim could only ever be a spell -- which is exactly what `OOS-DX25b-1` guaranteed,
/// and exactly what PB-DX52 stopped guaranteeing. `OOS-DX25c-3` predicted this: *"an
/// ability-shaped `ChangeTargets` victim would get the WRONG `self_id`/`source_chars`,
/// and the gap is blocked behind `OOS-DX25b-1` today"*. PB-DX52 unblocks it, so PB-DX52
/// closes it -- shipping the id space without this would have let Bolt Bend redirect a
/// red ability onto a creature with protection from red.
///
/// The values this returns are the SAME ones the ability's own activation validated
/// against: `abilities.rs::handle_activate_ability` passes `Some(source)` as `self_id`
/// and the source's characteristics as `source_chars`. So a retarget is now checked
/// against the same source the original announcement was.
///
/// # Deliberately NOT unified with `mtg_view_model::stack_kind_info`
///
/// That function returns a source too, and for `KeywordTrigger` it deliberately returns
/// the AFFECTED PERMANENT for `CounterRemoval` / `CounterSacrifice` / `UpkeepCost`
/// (`TriggerData`) rather than `source_object`, because it is a DISPLAY helper and the
/// affected permanent is what a player wants named in the stack panel. CR 113.7a's source
/// is `source_object` in all three cases. The two answers differ on purpose; collapsing
/// them would make one of the two wrong, and which one depends on the caller. Pinned by
/// `core::pb_dx52_stack_target_roster`.
///
/// Exhaustive with **no wildcard arm**, for the same reason [`card_in_stack_zone`] is: a
/// new `StackObjectKind` must be a compile error here until someone decides what its
/// source is, rather than silently becoming `None` and losing a protection check.
pub fn source_of(kind: &StackObjectKind) -> Option<ObjectId> {
    use StackObjectKind as K;
    match kind {
        K::Spell { source_object } => Some(*source_object),
        K::MutatingCreatureSpell { source_object, .. } => Some(*source_object),
        K::ActivatedAbility { source_object, .. } => Some(*source_object),
        K::LoyaltyAbility { source_object, .. } => Some(*source_object),
        K::TriggeredAbility { source_object, .. } => Some(*source_object),
        K::MadnessTrigger { source_object, .. } => Some(*source_object),
        K::MiracleTrigger { source_object, .. } => Some(*source_object),
        K::UnearthAbility { source_object } => Some(*source_object),
        K::SuspendCounterTrigger { source_object, .. } => Some(*source_object),
        K::SuspendCastTrigger { source_object, .. } => Some(*source_object),
        K::NinjutsuAbility { source_object, .. } => Some(*source_object),
        K::ForecastAbility { source_object, .. } => Some(*source_object),
        // CR 702.71a: bloodrush's `source_object` is the PRE-discard card id -- the card
        // is in the graveyard by the time the ability is on the stack (it was the cost).
        // It is still the object that generated the ability, so CR 113.7a names it.
        K::BloodrushAbility { source_object, .. } => Some(*source_object),
        K::SaddleAbility { source_object } => Some(*source_object),
        K::RingAbility { source_object, .. } => Some(*source_object),
        K::ClassLevelAbility { source_object, .. } => Some(*source_object),
        K::DelayedActionTrigger { source_object, .. } => Some(*source_object),
        // CR 113.7a with the source named by a differently-spelled field.
        K::TransformTrigger { permanent, .. } => Some(*permanent),
        K::DayboundTransformTrigger { permanent } => Some(*permanent),
        K::TurnFaceUpTrigger { permanent, .. } => Some(*permanent),
        K::CraftAbility { exiled_source, .. } => Some(*exiled_source),
        // CR 113.7a: `source_object` UNCONDITIONALLY -- including for the `CounterRemoval`
        // / `CounterSacrifice` / `UpkeepCost` `TriggerData` shapes, where
        // `mtg_view_model::stack_kind_info` deliberately reports the AFFECTED PERMANENT
        // instead. See this function's doc for why the two differ.
        K::KeywordTrigger { source_object, .. } => Some(*source_object),
        // No source object survives: the card was exiled or moved as the COST of putting
        // this ability on the stack (CR 400.7 -- it is a new object now), so there is no
        // live id to name. `None` here is a measured absence, not an unhandled case.
        K::EmbalmAbility { .. } => None,
        K::EternalizeAbility { .. } => None,
        K::EncoreAbility { .. } => None,
        K::ScavengeAbility { .. } => None,
        // CR 309.4c: a Room's ability is generated by a dungeon in the command zone,
        // which is not a `state.objects` permanent.
        K::RoomAbility { .. } => None,
    }
}

/// PB-DX25b (`OOS-DX25-3`): the index in `stack_objects` of the stack object an
/// **announced target id** names, if any (CR 601.2c).
///
/// Two disjoint id spaces meet here, and the whole point of this function is that
/// the decision is made once:
///
/// * a **card** id — `casting.rs::handle_cast_spell` moves the card into
///   `ZoneId::Stack` (CR 601.2a) with a fresh `ObjectId` (CR 400.7), and that is
///   the id the offer layer (`rules::queries::legal_targets_per_slot`) enumerates
///   and the player announces (CR 601.2c). `casting.rs:6308-6311` states this
///   invariant in prose.
/// * a **stack-entry** id — `state.next_object_id()`, one line later. It is never
///   in `state.objects`, so it can never be announced by a player, but it IS
///   passed as a target by engine-internal triggers (Ward, CR 702.21a, via
///   `PermanentTargeted`/`targeting_stack_id`).
///
/// Both are minted from the one monotone `timestamp_counter`
/// (`state/mod.rs:1012-1015`), so an id lives in exactly one of them and a bare
/// `so.id == announced` type-checks while being unsatisfiable on any real cast —
/// `OOS-SIM3-5` and `OOS-DX25-3` are the same defect, two functions apart.
///
/// CR 707.10: a COPY of a spell owns no card of its own. `copy.rs` clones the
/// ORIGINAL's `kind` wholesale, so a copy's `source_object` names the ORIGINAL's
/// card; without the `!so.is_copy` guard a single card id would match BOTH the
/// original and every copy of it, and `position` would silently return whichever
/// came first. The guard is therefore load-bearing twice over: for
/// disambiguation here, and for the CR 702.99c cipher-copy exile leak documented
/// at `Effect::CounterSpell`'s call site.
///
/// **This is not "is it a spell".** See this module's header: a copy IS a spell
/// (CR 707.10) and is deliberately NOT findable here. A consequence, stated
/// rather than hidden: a copy of a spell can never be the announced target of
/// Misdirection/Bolt Bend (`OOS-DX25b-2`).
///
/// **Why an index and not `Option<&StackObject>`**: `Effect::CounterSpell` needs
/// `state.stack_objects.remove(pos)`; `Effect::ChangeTargets` needs a shared read
/// *and* an `iter_mut()` write; `casting.rs` needs a shared read. An index serves
/// all three with one function.
pub fn stack_index_for_announced_target(
    stack_objects: &imbl::Vector<StackObject>,
    announced: ObjectId,
) -> Option<usize> {
    stack_objects.iter().position(|so| {
        so.id == announced || (!so.is_copy && card_in_stack_zone(&so.kind) == Some(announced))
    })
}
