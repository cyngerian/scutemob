// Kogla, the Titan Ape — {3}{G}{G}{G} Legendary Creature — Ape 7/6
// When Kogla enters, it fights up to one target creature you don't control.
// Whenever Kogla attacks, destroy target artifact or enchantment defending player controls.
// {1}{G}: Return target Human you control to its owner's hand. Kogla gains indestructible until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kogla-the-titan-ape"),
        name: "Kogla, the Titan Ape".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 3, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Ape"],
        ),
        oracle_text:
            "When Kogla enters, it fights up to one target creature you don't control.\nWhenever Kogla attacks, destroy target artifact or enchantment defending player controls.\n{1}{G}: Return target Human you control to its owner's hand. Kogla gains indestructible until end of turn."
                .to_string(),
        power: Some(7),
        toughness: Some(6),
        abilities: vec![
            // CR 603.1: ETB trigger — fight up to one target creature you don't control.
            // "up to one" means the target is optional (can choose 0 targets).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Fight {
                    attacker: EffectTarget::Source,
                    defender: EffectTarget::DeclaredTarget { index: 0 },
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],
            },
            // CR 603.1: Attack trigger — destroy target artifact or enchantment defending player controls.
            // "defending player controls" is approximated as opponent; full defender-tracking
            // is not in DSL (TargetController::Opponent is accurate in most multiplayer contexts).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],
            },
            // {1}{G}: Return target Human you control to its owner's hand.
            //         Kogla gains indestructible until end of turn.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 1, green: 1, ..Default::default() }),
                effect: Effect::Sequence(vec![
                    Effect::MoveZone {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        to: ZoneTarget::Hand {
                            owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 })),
                        },
                        controller_override: None,
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(
                                KeywordAbility::Indestructible,
                            ),
                            filter: EffectFilter::Source,
                            duration: EffectDuration::UntilEndOfTurn,
                        }),
                    },
                ]),
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    has_subtype: Some(SubType("Human".to_string())),
                    controller: TargetController::You,
                    ..Default::default()
                })],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
