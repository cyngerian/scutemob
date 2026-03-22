// Flawless Maneuver — {2}{W}, Instant
// Oracle: "If you control a commander, you may cast this spell without paying its mana cost.
// Creatures you control gain indestructible until end of turn."
// Note: Conditional free cast (commander presence) not expressible — main effect implemented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("flawless-maneuver"),
        name: "Flawless Maneuver".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "If you control a commander, you may cast this spell without paying its mana cost.\nCreatures you control gain indestructible until end of turn.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: DSL gap — conditional free cast ("if you control a commander") not expressible.
            // Main effect: creatures you control gain indestructible until end of turn.
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
                    }),
                }),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
