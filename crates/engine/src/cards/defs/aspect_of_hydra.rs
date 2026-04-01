// Aspect of Hydra — {G}, Instant
// Target creature gets +X/+X until end of turn, where X is your devotion to green.
// (Your devotion to green is the number of green mana symbols in the mana costs of permanents you control.)
//
// TODO: DSL gap — LayerModification::ModifyBoth(i32) takes a static i32, not EffectAmount.
// Implementing "+X/+X where X = devotion" requires LayerModification::ModifyBothDynamic(EffectAmount)
// which does not exist. Approximated as Nothing to avoid wrong game state.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("aspect-of-hydra"),
        name: "Aspect of Hydra".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Target creature gets +X/+X until end of turn, where X is your devotion to green. (Your devotion to green is the number of green mana symbols in the mana costs of permanents you control.)".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: CR 702.5c: devotion to green = number of {G} symbols in mana costs of
            // permanents you control. EffectAmount::DevotionTo(Color::Green) computes this,
            // but LayerModification::ModifyBoth only takes a static i32.
            // Needs LayerModification::ModifyBothDynamic(Box<EffectAmount>) variant.
            effect: Effect::Nothing,
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
