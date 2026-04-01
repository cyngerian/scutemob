// Nanogene Conversion — {3}{U}, Sorcery
// Choose target creature you control. Each other creature becomes a copy of that
// creature until end of turn, except it isn't legendary.
//
// TODO: "each other creature becomes a copy" — needs ForEach over all creatures +
// BecomeCopyOf(target) + except-not-legendary. ForEach(AllCreatures) → BecomeCopyOf
// with DeclaredTarget(0) doesn't exist as a one-shot copy-all pattern.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nanogene-conversion"),
        name: "Nanogene Conversion".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose target creature you control. Each other creature becomes a copy of that creature until end of turn, except it isn't legendary.".to_string(),
        abilities: vec![
            // TODO: ForEach(AllCreatures) + BecomeCopyOf(target) + except-not-legendary.
            // Multiple DSL gaps: mass copy effect, "except" clause on copies.
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
