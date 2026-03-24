// Boros Charm — {R}{W} Instant
// Choose one — • Boros Charm deals 4 damage to target player or planeswalker.
// • Permanents you control gain indestructible until end of turn.
// • Target creature gains double strike until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("boros-charm"),
        name: "Boros Charm".to_string(),
        mana_cost: Some(ManaCost { red: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n• Boros Charm deals 4 damage to target player or planeswalker.\n• Permanents you control gain indestructible until end of turn.\n• Target creature gains double strike until end of turn.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            targets: vec![
                TargetRequirement::TargetPlayerOrPlaneswalker,
                TargetRequirement::TargetCreature,
            ],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: 4 damage to target player or planeswalker.
                    Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(4),
                    },
                    // Mode 1: Permanents you control gain indestructible until EOT.
                    // TODO: Mass indestructible grant to all permanents you control
                    // not expressible as a single ApplyContinuousEffect (needs all-permanents filter).
                    Effect::Nothing,
                    // Mode 2: Target creature gains double strike until EOT.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::DoubleStrike),
                            filter: EffectFilter::DeclaredTarget { index: 1 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
