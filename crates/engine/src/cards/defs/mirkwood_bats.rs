// Mirkwood Bats — {3}{B}, Creature — Bat 2/3
// Flying
// Whenever you create or sacrifice a token, each opponent loses 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mirkwood-bats"),
        name: "Mirkwood Bats".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Bat"]),
        oracle_text: "Flying\nWhenever you create or sacrifice a token, each opponent loses 1 life.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // Whenever you sacrifice a token, each opponent loses 1 life.
            // (Create-a-token half is already covered by TokenCreated event via existing triggers.)
            // TODO: token-only filter on sacrifice (is_token field not in TargetFilter).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouSacrifice {
                    filter: None,
                    player_filter: None,
                },
                effect: Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::LoseLife {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(1),
                    }),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
