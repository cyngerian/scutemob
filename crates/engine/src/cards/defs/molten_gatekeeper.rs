// Molten Gatekeeper — {2}{R}, Artifact Creature — Golem 2/3
// Whenever another creature you control enters, this creature deals 1 damage to each opponent.
// Unearth {R}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("molten-gatekeeper"),
        name: "Molten Gatekeeper".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Golem"]),
        oracle_text: "Whenever another creature you control enters, this creature deals 1 damage to each opponent.\nUnearth {R} ({R}: Return this card from your graveyard to the battlefield. It gains haste. Exile it at the beginning of the next end step or if it would leave the battlefield. Unearth only as a sorcery.)".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            // Whenever another creature you control enters, deal 1 to each opponent.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(1),
                    }),
                },
                intervening_if: None,
                targets: vec![],
            },
            AbilityDefinition::Keyword(KeywordAbility::Unearth),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Unearth,
                cost: ManaCost { red: 1, ..Default::default() },
                details: None,
            },
        ],
        ..Default::default()
    }
}
