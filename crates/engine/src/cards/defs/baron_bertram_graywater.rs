// Baron Bertram Graywater — {2}{W}{B}, Legendary Creature — Vampire Noble 3/4
// Whenever one or more tokens you control enter, create a 1/1 black Vampire Rogue creature
// token with lifelink. This ability triggers only once each turn.
// {1}{B}, Sacrifice another creature or artifact: Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("baron-bertram-graywater"),
        name: "Baron Bertram Graywater".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Vampire", "Noble"],
        ),
        oracle_text: "Whenever one or more tokens you control enter, create a 1/1 black Vampire Rogue creature token with lifelink. This ability triggers only once each turn.\n{1}{B}, Sacrifice another creature or artifact: Draw a card.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            // TODO: "Whenever tokens enter" trigger + "once each turn" not in DSL.
            //   Using generic creature ETB as approximation.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Vampire Rogue".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Vampire".to_string()), SubType("Rogue".to_string())].into_iter().collect(),
                        colors: [Color::Black].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::Lifelink].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // {1}{B}, Sacrifice: Draw a card
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, black: 1, ..Default::default() }),
                    Cost::Sacrifice(TargetFilter::default()),
                ]),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
