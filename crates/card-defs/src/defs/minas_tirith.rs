// Minas Tirith
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("minas-tirith"),
        name: "Minas Tirith".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "Minas Tirith enters tapped unless you control a legendary creature.\n{T}: \
                      Add {W}.\n{1}{W}, {T}: Draw a card. Activate only if you attacked with two \
                      or more creatures this turn."
            .to_string(),
        abilities: vec![
            // CR 614.1c: enters tapped unless you control a legendary creature.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: Some(Condition::ControlLegendaryCreature),
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            // ENGINE-BLOCKED: "{1}{W}, {T}: Draw a card. Activate only if you attacked with two
            // or more creatures this turn." Needs a count-based attacked condition
            // (Condition::AttackedWithNCreatures(2)). PB-AC6's Condition::YouAttackedThisTurn is
            // a bool and is insufficient — it cannot distinguish one attacker from two.
        ],
        completeness: Completeness::partial(
            "'{1}{W}, {T}: Draw a card. Activate only if you attacked with two or more creatures \
             this turn.' Needs a count-based...",
        ),
        ..Default::default()
    }
}
