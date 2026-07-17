// Vein Ripper — {3}{B}{B}{B}, Creature — Vampire Assassin 6/5
// Flying
// Ward—Sacrifice a creature.
// Whenever a creature dies, target opponent loses 2 life and you gain 2 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vein-ripper"),
        name: "Vein Ripper".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            black: 3,
            ..Default::default()
        }),
        types: creature_types(&["Vampire", "Assassin"]),
        oracle_text: "Flying\nWard—Sacrifice a creature.\nWhenever a creature dies, target \
                      opponent loses 2 life and you gain 2 life."
            .to_string(),
        power: Some(6),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — "Ward—Sacrifice a creature." Ward with non-mana cost
            // (sacrifice) not in KeywordAbility::Ward variant.
            // "Whenever a creature dies" = any creature, no filter needed.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: None,
                    exclude_self: false,
                    nontoken_only: false,
                    filter: None,
                },
                effect: Effect::DrainLife {
                    amount: EffectAmount::Fixed(2),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::partial(
            "Ward—Sacrifice a creature is inexpressible (KeywordAbility::Ward carries only a mana \
             value, no non-mana cost). Additionally the dies-trigger drain is untargeted (oracle: \
             'target opponent'); DrainLife is declared with targets: vec![], so the life loss \
             does not go to a chosen opponent.",
        ),
        ..Default::default()
    }
}
