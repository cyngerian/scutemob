// Mistblade Shinobi — {2}{U}, Creature — Human Ninja 1/1
// Ninjutsu {U}
// Whenever this creature deals combat damage to a player, you may return target creature
// that player controls to its owner's hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mistblade-shinobi"),
        name: "Mistblade Shinobi".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Ninja"]),
        oracle_text: "Ninjutsu {U} ({U}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)\nWhenever this creature deals combat damage to a player, you may return target creature that player controls to its owner's hand.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost { blue: 1, ..Default::default() },
            },
            // CR 510.3a: "Whenever this creature deals combat damage to a player, you may
            // return target creature that player controls to its owner's hand."
            // Authored as mandatory (no "you may" wrapper — authored consistently with Sigil of Sleep
            // and per PB-D plan R7). TODO(MayEffect): "you may" optionality deferred to a future
            // MayEffect primitive; authoring as mandatory is correct for the DamagedPlayer scoping.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Hand {
                        owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 })),
                    },
                    controller_override: None,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::DamagedPlayer,
                    ..Default::default()
                })],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
