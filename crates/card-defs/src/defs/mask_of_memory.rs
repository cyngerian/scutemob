// Mask of Memory — {2}, Artifact — Equipment
// Whenever equipped creature deals combat damage to a player, you may draw two cards.
// If you do, discard a card.
// Equip {1}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mask-of-memory"),
        name: "Mask of Memory".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: full_types(&[], &[CardType::Artifact], &["Equipment"]),
        oracle_text: "Whenever equipped creature deals combat damage to a player, you may draw \
                      two cards. If you do, discard a card.\nEquip {1}"
            .to_string(),
        abilities: vec![
            // CR 510.3a: "Whenever equipped creature deals combat damage to a player,
            // draw two cards, then discard a card." (approximation — "may" draw 2 not in DSL)
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer,
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(2),
                    },
                    Effect::DiscardCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Equip),
            // Equip {1}: attach this Equipment to target creature you control.
            // CR 702.6b: Equip is an activated ability; CR 702.6d: sorcery speed only.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 1,
                    ..Default::default()
                }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                // PB-DX26 (OOS-CARDS1-3) / CR 702.6a: "Equip {1}" means "[Cost]: Attach this
                // permanent to target creature you control." Printed line MCP-verified as
                // plain "Equip {1}" with no CR 702.6c quality restriction, so the requirement
                // is the unmodified 702.6a one.
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::known_wrong(
            "'you may draw two cards, then discard' implemented as a mandatory draw",
        ),
        ..Default::default()
    }
}
