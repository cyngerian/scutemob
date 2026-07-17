// Aura Shards — {1}{G}{W} Enchantment
// Whenever a creature you control enters, you may destroy target artifact or enchantment.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("aura-shards"),
        name: "Aura Shards".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            white: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control enters, you may destroy target artifact or \
                      enchantment."
            .to_string(),
        abilities: vec![
            // CR 603.1: Triggered — fires when any creature you control enters the battlefield.
            // Oracle says "a creature" (not "another"), so exclude_self: false matches the
            // text. Aura Shards is an enchantment and cannot be the entering creature anyway,
            // so the setting is moot in practice but kept faithful to oracle (PB-XS-E).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                    ..Default::default()
                })],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
