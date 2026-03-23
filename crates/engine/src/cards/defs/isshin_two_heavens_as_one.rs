// Isshin, Two Heavens as One — {R}{W}{B}, Legendary Creature — Human Samurai 3/4
// If a creature attacking causes a triggered ability of a permanent you control to trigger,
// that ability triggers an additional time.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("isshin-two-heavens-as-one"),
        name: "Isshin, Two Heavens as One".to_string(),
        mana_cost: Some(ManaCost { red: 1, white: 1, black: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Samurai"]),
        oracle_text: "If a creature attacking causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            // TODO: DSL gap — Isshin's ability doubles attack-triggered abilities of permanents
            // you control. This requires a static TriggerDoubler scoped to attack triggers,
            // similar to Panharmonicon but for attack triggers rather than ETB triggers.
            // No such LayerModification or TriggerDoublerFilter variant exists in the current DSL.
        ],
        ..Default::default()
    }
}
