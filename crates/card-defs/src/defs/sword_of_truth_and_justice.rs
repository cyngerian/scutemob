// Sword of Truth and Justice — {3}, Artifact — Equipment
// Equipped creature gets +2/+2 and has protection from white and from blue.
// Whenever equipped creature deals combat damage to a player, put a +1/+1 counter on a
// creature you control, then proliferate.
// Equip {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sword-of-truth-and-justice"),
        name: "Sword of Truth and Justice".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +2/+2 and has protection from white and from \
                      blue.\nWhenever equipped creature deals combat damage to a player, put a \
                      +1/+1 counter on a creature you control, then proliferate.\nEquip {2}"
            .to_string(),
        abilities: vec![
            // Layer 7c: equipped creature gets +2/+2.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(2),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Layer 6: equipped creature has protection from white.
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
            // Layer 6: equipped creature has protection from blue.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::ProtectionFrom(
                        ProtectionQuality::FromColor(Color::Blue),
                    )),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 510.3a: "Whenever equipped creature deals combat damage to a player,
            // put a +1/+1 counter on a creature you control, then proliferate."
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer,
                effect: Effect::Sequence(vec![
                    Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    },
                    Effect::Proliferate,
                ]),
                intervening_if: None,
                // CR 601.2c: "put a +1/+1 counter on a creature you control" — controller-
                // restricted, not "another", so no exclude_self.
                // PB-DX4 fixed the CONTROLLER axis here ("a creature **you control**"; this
                // was a bare `TargetRequirement::TargetCreature`, so the counter could land on
                // an opponent's creature).
                //
                // The TARGETING axis is a second, unfixed deviation and is recorded rather
                // than left silent (fix cycle, review Finding 6): the printed clause is "put a
                // +1/+1 counter on a creature you control" with **no "target"** (CR 115.10 —
                // an effect only targets when it says so), so the choice is made on
                // resolution, cannot be responded to, and is unaffected by hexproof, shroud,
                // protection or a "can't be the target of" restriction. Authoring it as a real
                // target makes all five of those bite, and lets CR 608.2b fizzle the whole
                // trigger when the chosen creature leaves. The DSL has no
                // choose-on-resolution-without-targeting channel for this shape, so it is not
                // authorable today; filed as the class it is — **OOS-DX4-6**, whose second
                // known member (`frantic_search`, printed "untap **up to three** lands" with
                // no "target") this batch's own triage found independently.
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                })],

                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        ..Default::default()
    }
}
