// Tekuthal, Inquiry Dominus — {2}{U}{U}, Legendary Creature — Phyrexian Horror 3/5
// Flying
// If you would proliferate, proliferate twice instead.
// {1}{U/P}{U/P}, Remove three counters from among other artifacts, creatures, and planeswalkers
// you control: Put an indestructible counter on Tekuthal.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tekuthal-inquiry-dominus"),
        name: "Tekuthal, Inquiry Dominus".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Phyrexian", "Horror"],
        ),
        oracle_text: "Flying\nIf you would proliferate, proliferate twice instead.\n{1}{U/P}{U/P}, Remove three counters from among other artifacts, creatures, and planeswalkers you control: Put an indestructible counter on Tekuthal. ({U/P} can be paid with either {U} or 2 life.)".to_string(),
        power: Some(3),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 701.34 / CR 614.1: Proliferate doubling replacement.
            // PlayerId(0) placeholder — bound to controller at registration.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldProliferate {
                    player_filter: PlayerFilter::Specific(PlayerId(0)),
                },
                modification: ReplacementModification::DoubleProliferate,
                is_self: false,
                unless_condition: None,
            },
            // TODO: Activated ability — {1}{U/P}{U/P}, remove three counters from among other
            // artifacts/creatures/planeswalkers you control: put an indestructible counter on this.
            // DSL gap: remove-counters-from-others cost not yet expressible.
        ],
        ..Default::default()
    }
}
