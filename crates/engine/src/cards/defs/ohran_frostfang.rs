// Ohran Frostfang — {3}{G}{G}, Snow Creature — Snake 2/6
// Attacking creatures you control have deathtouch.
// Whenever a creature you control deals combat damage to a player, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ohran-frostfang"),
        name: "Ohran Frostfang".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 2, ..Default::default() }),
        types: full_types(&[SuperType::Snow], &[CardType::Creature], &["Snake"]),
        oracle_text: "Attacking creatures you control have deathtouch.\nWhenever a creature you control deals combat damage to a player, draw a card.".to_string(),
        power: Some(2),
        toughness: Some(6),
        abilities: vec![
            // CR 613.1f / CR 611.3a: "Attacking creatures you control have deathtouch."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Deathtouch),
                    filter: EffectFilter::AttackingCreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 510.3a: "Whenever a creature you control deals combat damage to a player,
            // draw a card." (PB-23: WheneverCreatureYouControlDealsCombatDamageToPlayer)
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
