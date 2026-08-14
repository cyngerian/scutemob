// Sword of War and Peace — {3}, Artifact — Equipment
// Equipped creature gets +2/+2 and has protection from red and from white.
// Whenever equipped creature deals combat damage to a player, this Equipment deals damage
// to that player equal to the number of cards in their hand and you gain 1 life for each
// card in your hand.
// Equip {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sword-of-war-and-peace"),
        name: "Sword of War and Peace".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +2/+2 and has protection from red and from \
                      white.\nWhenever equipped creature deals combat damage to a player, Sword \
                      of War and Peace deals damage to that player equal to the number of cards \
                      in their hand and you gain 1 life for each card in your hand.\nEquip {2}"
            .to_string(),
        abilities: vec![
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(2),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::ProtectionFrom(
                        ProtectionQuality::FromColor(Color::Red),
                    )),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::ProtectionFrom(
                        ProtectionQuality::FromColor(Color::White),
                    )),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 510.3a / CR 115.10 (PB-DX28): "Whenever equipped creature deals combat
            // damage to a player, [this Equipment] deals damage to THAT PLAYER equal to
            // cards in their hand; gain 1 life per card in your hand." This is
            // DETERMINED, not targeted — "that player" is whoever the equipped
            // creature just damaged, not a chosen target (CR 115.10 lists no "target"
            // word in the printed text). The old comment already claimed
            // `ctx.damaged_player` resolution while the code actually declared a
            // `TargetPlayer` requirement and read `DeclaredTarget { index: 0 }` — the
            // PB-DX27 stale-note class, live: in a 4-player game the CR 601.2c
            // auto-target picker chose *a* player, so the Sword could damage the
            // wrong seat. `EffectTarget::DamagedPlayer` / `PlayerTarget::DamagedPlayer`
            // both resolve from `EffectContext::damaged_player`, so no `targets` slot
            // is declared at all.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer,
                effect: Effect::Sequence(vec![
                    Effect::DealDamage {
                        source: None,
                        target: EffectTarget::DamagedPlayer,
                        amount: EffectAmount::CardCount {
                            zone: ZoneTarget::Hand {
                                owner: PlayerTarget::DamagedPlayer,
                            },
                            player: PlayerTarget::DamagedPlayer,
                            filter: None,
                        },
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::CardCount {
                            zone: ZoneTarget::Hand {
                                owner: PlayerTarget::Controller,
                            },
                            player: PlayerTarget::Controller,
                            filter: None,
                        },
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Equip),
            // Equip {2}: attach this Equipment to target creature you control.
            // CR 702.6b: Equip is an activated ability; CR 702.6d: sorcery speed only.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 2,
                    ..Default::default()
                }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                // PB-DX26 (OOS-CARDS1-3) / CR 702.6a: "Equip {2}" means "[Cost]: Attach this
                // permanent to target creature you control." Printed line MCP-verified as
                // plain "Equip {2}" with no CR 702.6c quality restriction, so the requirement
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
        ..Default::default()
    }
}
