// Satoru, the Infiltrator — {U}{B}, Legendary Creature — Human Ninja Rogue 2/3
// Menace
// Whenever Satoru and/or one or more other nontoken creatures you control enter, if none
// of them were cast or no mana was spent to cast them, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("satoru-the-infiltrator"),
        name: "Satoru, the Infiltrator".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            black: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Ninja", "Rogue"],
        ),
        oracle_text: "Menace\nWhenever Satoru and/or one or more other nontoken creatures you \
                      control enter, if none of them were cast or no mana was spent to cast them, \
                      draw a card."
            .to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            // TODO: Complex ETB condition "if none were cast or no mana spent" not in DSL.
            //   Using generic creature ETB as approximation.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::known_wrong(
            "ETB draw fires on every nontoken/token creature you control entering; oracle \
             requires 'none of them were cast or no mana was spent to cast them' (no Condition \
             variant for cast-ness/mana-spent of an entering permanent) and a batched \
             Satoru-and/or-others trigger. Current def over-draws.",
        ),
        ..Default::default()
    }
}
