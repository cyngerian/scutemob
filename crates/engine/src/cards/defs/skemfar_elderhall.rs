// Skemfar Elderhall
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skemfar-elderhall"),
        name: "Skemfar Elderhall".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\n{T}: Add {G}.\n{2}{B}{B}{G}, {T}, Sacrifice this land: Up to one target creature you don't control gets -2/-2 until end of turn. Create two 1/1 green Elf Warrior creature tokens. Activate only as a sorcery.".to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement — this land enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // {T}: Add {G}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 0, 0, 1, 0) },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // {2}{B}{B}{G}, {T}, Sacrifice this land: Up to one target creature you don't control
            // gets -2/-2 until end of turn. Create two 1/1 green Elf Warrior creature tokens.
            // Activate only as a sorcery.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, black: 2, green: 1, ..Default::default() }),
                    Cost::Tap,
                    Cost::SacrificeSelf,
                ]),
                effect: Effect::Sequence(vec![
                    // Target creature you don't control gets -2/-2 until end of turn.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: crate::state::EffectLayer::PtModify,
                            modification: crate::state::LayerModification::ModifyBoth(-2),
                            filter: crate::state::EffectFilter::DeclaredTarget { index: 0 },
                            duration: crate::state::EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    // Create two 1/1 green Elf Warrior creature tokens.
                    Effect::CreateToken {
                        spec: TokenSpec {
                            name: "Elf Warrior".to_string(),
                            power: 1,
                            toughness: 1,
                            colors: [Color::Green].into_iter().collect(),
                            supertypes: im::OrdSet::new(),
                            card_types: [CardType::Creature].into_iter().collect(),
                            subtypes: [SubType("Elf".to_string()), SubType("Warrior".to_string())].into_iter().collect(),
                            keywords: im::OrdSet::new(),
                            count: 2,
                            tapped: false,
                            enters_attacking: false,
                            mana_color: None,
                            mana_abilities: vec![],
                            activated_abilities: vec![],
                            ..Default::default()
                        },
                    },
                ]),
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
