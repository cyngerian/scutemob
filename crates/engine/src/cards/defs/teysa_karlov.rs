// Teysa Karlov — {2}{W}{B}, Legendary Creature — Human Advisor 2/4
// If a creature dying causes a triggered ability of a permanent you control to trigger,
// that ability triggers an additional time.
// Creature tokens you control have vigilance and lifelink.
// TODO: DSL gap — doubling death triggers requires a replacement effect on trigger queueing
// with source-creature-dying filter; not supported. Token keyword grant is also unsupported.
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
        abilities: vec![],
        // TODO: double death-trigger ability (requires trigger-doubling replacement effect)
        // TODO: grant vigilance and lifelink to creature tokens you control
        ..Default::default()
    }
}
