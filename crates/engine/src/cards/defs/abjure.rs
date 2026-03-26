// Abjure — {U}, Instant
// As an additional cost to cast this spell, sacrifice a blue permanent.
// Counter target spell.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("abjure"),
        name: "Abjure".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "As an additional cost to cast this spell, sacrifice a blue permanent.\nCounter target spell.".to_string(),
        // CR 118.8: Mandatory sacrifice of a blue permanent as additional cost.
        spell_additional_costs: vec![SpellAdditionalCost::SacrificeColorPermanent(Color::Blue)],
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::CounterSpell {
                target: EffectTarget::DeclaredTarget { index: 0 },
            },
            targets: vec![TargetRequirement::TargetSpell],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
