// Shizo, Death's Storehouse — Legendary Land, {T}: Add {B}; fear grant ability (TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shizo-deaths-storehouse"),
        name: "Shizo, Death's Storehouse".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {B}.\n{B}, {T}: Target legendary creature gains fear until end of turn. (It can't be blocked except by artifact creatures and/or black creatures.)".to_string(),
        abilities: vec![
            // {T}: Add {B}
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // {B}, {T}: Target legendary creature gains fear until end of turn (CR 702.36).
            // Note: TargetFilter lacks has_supertype field — using TargetCreature (over-permissive,
            // allows targeting non-legendary creatures). TODO: add legendary filter when available.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { black: 1, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(KeywordAbility::Fear),
                        filter: EffectFilter::DeclaredTarget { index: 0 },
                        duration: EffectDuration::UntilEndOfTurn,
                    }),
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreature],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
