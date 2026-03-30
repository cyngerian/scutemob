// Thespian's Stage — Land
// {T}: Add {C}.
// {2}, {T}: This land becomes a copy of target land, except it has this ability.
// CR 707.2: Copy effect with Indefinite duration (no end-of-turn expiry).
// Ruling 2018-12-07: "The copy effect doesn't have a duration. It will last until
// Thespian's Stage leaves the battlefield or another copy effect overwrites it."
// TODO: "except it has this ability" — retained ability after copy not yet expressible.
//   Currently the copy overwrites all abilities including the {2},{T} copy ability.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thespians-stage"),
        name: "Thespian's Stage".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{2}, {T}: This land becomes a copy of target land, except it has this ability.".to_string(),
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
            },
            // {2}, {T}: This land becomes a copy of target land.
            // CR 707.2: Indefinite copy effect (no duration).
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 2,
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::BecomeCopyOf {
                    copier: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    duration: EffectDuration::Indefinite,
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_type: Some(CardType::Land),
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
