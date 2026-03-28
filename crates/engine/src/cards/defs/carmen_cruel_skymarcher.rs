// Carmen, Cruel Skymarcher — {3}{W}{B}, Legendary Creature — Vampire Soldier 2/2
// Flying
// Whenever a player sacrifices a permanent, put a +1/+1 counter on Carmen and you gain 1 life.
// Whenever Carmen attacks, return up to one target permanent card with mana value less than
// or equal to Carmen's power from your graveyard to the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("carmen-cruel-skymarcher"),
        name: "Carmen, Cruel Skymarcher".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Vampire", "Soldier"],
        ),
        oracle_text: "Flying\nWhenever a player sacrifices a permanent, put a +1/+1 counter on Carmen, Cruel Skymarcher and you gain 1 life.\nWhenever Carmen attacks, return up to one target permanent card with mana value less than or equal to Carmen's power from your graveyard to the battlefield.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // Whenever a player sacrifices a permanent, put +1/+1 counter and gain 1 life.
            // player_filter: Any = fires for all players (the dispatch already sends for any player).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouSacrifice {
                    filter: None,
                    player_filter: Some(TargetController::Any),
                },
                effect: Effect::Sequence(vec![
                    Effect::AddCounter {
                        target: EffectTarget::Source,
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: Attack trigger returning GY permanent with power-based MV filter not expressible.
        ],
        ..Default::default()
    }
}
