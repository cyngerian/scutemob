// Butcher of Malakir — {5}{B}{B}, Creature — Vampire Warrior 5/4
// Flying
// Whenever this creature or another creature you control dies, each opponent sacrifices
// a creature of their choice.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("butcher-of-malakir"),
        name: "Butcher of Malakir".to_string(),
        mana_cost: Some(ManaCost { generic: 5, black: 2, ..Default::default() }),
        types: creature_types(&["Vampire", "Warrior"]),
        oracle_text: "Flying\nWhenever Butcher of Malakir or another creature you control dies, each opponent sacrifices a creature of their choice.".to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 603.10a: "Whenever Butcher of Malakir or another creature you control dies,
            // each opponent sacrifices a creature."
            // PB-SFT (CR 701.17a + CR 109.1c): creature-only filter applied.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: false,
                    nontoken_only: false,
                    filter: None,
                },
                effect: Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::SacrificePermanents {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        count: EffectAmount::Fixed(1),
                        filter: Some(TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            ..Default::default()
                        }),
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
