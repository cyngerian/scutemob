// Workhorse — {6}, Artifact Creature — Horse 0/0
// This creature enters with four +1/+1 counters on it.
// Remove a +1/+1 counter from this creature: Add {C}.
//
// PB-OS11 (OOS-LKI-3 reframed, CR 605.1a): "Remove a +1/+1 counter: Add {C}" is a mana
// ability -- no target, could add mana, not a loyalty ability -- so it resolves without
// the stack (CR 605.1a/605.3b). It has NO {T} in its cost, only a self-referential
// remove-counter cost, which `mana_ability_lowering` now accepts (ManaAbility.remove_counter,
// PB-OS11) and `handle_tap_for_mana` pays (self-exhausting: bounded by counters actually on
// the permanent, exactly like `exile_self_from_hand`'s no-tap relaxation precedent, PB-EF8).
// Workhorse produces a FIXED {C} (not any-color), so it doesn't touch the AddManaAnyColor
// color bug that keeps Gemstone Array / Druids' Repository known_wrong -- this is the clean
// flip the primitive was built for.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("workhorse"),
        name: "Workhorse".to_string(),
        mana_cost: Some(ManaCost {
            generic: 6,
            ..Default::default()
        }),
        types: full_types(&[], &[CardType::Artifact, CardType::Creature], &["Horse"]),
        oracle_text: "This creature enters with four +1/+1 counters on it.\nRemove a +1/+1 \
                      counter from this creature: Add {C}."
            .to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![
            // CR 614.1c: "enters with" is a self-replacement effect, not a triggered
            // ability -- mirrors eomer_king_of_rohan.rs / ingenious_prodigy.rs.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersWithCounters {
                    counter: CounterType::PlusOnePlusOne,
                    count: Box::new(EffectAmount::Fixed(4)),
                },
                is_self: true,
                unless_condition: None,
            },
            // CR 605.1a: mana ability -- "Remove a +1/+1 counter: Add {C}." No {T}.
            // Lowered into a true ManaAbility by mana_ability_lowering (PB-OS11).
            AbilityDefinition::Activated {
                cost: Cost::RemoveCounter {
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1), // {C}
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        // PB-OS11: enters-with-counters replacement + RemoveCounter mana-ability
        // lowering both execution-verified (see engine test suite). Fixed {C}
        // production, so the AddManaAnyColor color bug affecting Gemstone Array /
        // Druids' Repository does not apply here.
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
