// Flawless Maneuver — {2}{W}, Instant
// If you control a commander, you may cast this spell without paying its mana cost.
// Creatures you control gain indestructible until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("flawless-maneuver"),
        name: "Flawless Maneuver".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "If you control a commander, you may cast this spell without paying its mana cost.\nCreatures you control gain indestructible until end of turn.".to_string(),
        abilities: vec![
            // CR 118.9 / 2020-04-17 ruling: cast without paying mana cost if you control
            // any commander on the battlefield (any player's commander qualifies).
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::CommanderFreeCast,
                cost: ManaCost::default(),
                details: None,
            },
            AbilityDefinition::Spell {
                // CR 611.3a: Until-end-of-turn continuous effect via Layer 6.
                effect: Effect::ForEach {
                    over: ForEachTarget::EachCreatureYouControl,
                    effect: Box::new(Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(
                                KeywordAbility::Indestructible,
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    }),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
