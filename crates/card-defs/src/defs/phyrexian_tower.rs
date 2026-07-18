// Phyrexian Tower — Legendary Land, {T}: Add {C}; sacrifice creature for {B}{B} (TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("phyrexian-tower"),
        name: "Phyrexian Tower".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}, Sacrifice a creature: Add {B}{B}.".to_string(),
        abilities: vec![
            // {T}: Add {C}
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // {T}, Sacrifice a creature: Add {B}{B}
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Tap,
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                ]),
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 2, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::partial(
            "CR 605.1a/605.3b: the '{T}: Add {C}' ability IS a correctly registered mana ability, \
             but '{T}, Sacrifice a creature: Add {B}{B}' is registered as a stack-using activated \
             ability. The MANA is correct (probed: +2 black, creature sacrificed). Same \
             Cost::Sacrifice(filter) / no-ObjectId-channel blocker as Ashnod's Altar.",
        ),
        ..Default::default()
    }
}
