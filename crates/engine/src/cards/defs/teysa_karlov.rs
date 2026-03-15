// Teysa Karlov — {2}{W}{B}, Legendary Creature — Human Advisor 2/4
// If a creature dying causes a triggered ability of a permanent you control to trigger,
// that ability triggers an additional time.
// Creature tokens you control have vigilance and lifelink.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("teysa-karlov"),
        name: "Teysa Karlov".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Advisor"],
        ),
        oracle_text: "If a creature dying causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.\nCreature tokens you control have vigilance and lifelink.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            // CR 603.2d: Death trigger doubling — creature dying causes triggers
            // to fire an additional time.
            AbilityDefinition::TriggerDoubling {
                filter: TriggerDoublerFilter::CreatureDeath,
                additional_triggers: 1,
            },
            // TODO: "Creature tokens you control have vigilance and lifelink."
            // Requires a token-only EffectFilter (EffectFilter::TokenCreatures) for
            // the static ability grant. Not yet expressible in the DSL.
        ],
        ..Default::default()
    }
}
