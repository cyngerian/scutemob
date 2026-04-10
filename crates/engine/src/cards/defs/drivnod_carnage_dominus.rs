// Drivnod, Carnage Dominus — {3}{B}{B}, Legendary Creature — Phyrexian Horror 8/3
// If a creature dying causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.
// {B/P}{B/P}, Exile three creature cards from your graveyard: Put an indestructible counter on Drivnod.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("drivnod-carnage-dominus"),
        name: "Drivnod, Carnage Dominus".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Phyrexian", "Horror"],
        ),
        oracle_text: "If a creature dying causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.\n{B/P}{B/P}, Exile three creature cards from your graveyard: Put an indestructible counter on Drivnod. ({B/P} can be paid with either {B} or 2 life.)".to_string(),
        power: Some(8),
        toughness: Some(3),
        abilities: vec![
            // CR 603.2d: If a creature dying causes a triggered ability of a permanent you
            // control to trigger, that ability triggers an additional time.
            AbilityDefinition::TriggerDoubling {
                filter: TriggerDoublerFilter::CreatureDeath,
                additional_triggers: 1,
            },
            // TODO: Activated ability — {B/P}{B/P}, exile three creature cards from your graveyard:
            // put an indestructible counter on this.
            // DSL gap: exile-from-graveyard activated cost + AddCounters self-target not yet supported.
        ],
        ..Default::default()
    }
}
