// Lae'zel, Vlaakith's Champion — {2}{W}, Legendary Creature — Gith Warrior 3/3
// Choose a Background.
// If you would put one or more counters on a creature or planeswalker you control or on
// yourself, put that many plus one instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("laez-el-vlaakiths-champion"),
        name: "Lae'zel, Vlaakith's Champion".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Gith", "Warrior"]),
        oracle_text: "Choose a Background.\nIf you would put one or more counters on a creature or planeswalker you control or on yourself, put that many plus one instead.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // CR 702.124k: "Choose a Background" — partner variant for Background enchantments.
            AbilityDefinition::Keyword(KeywordAbility::ChooseABackground),
            // CR 122.6 / CR 614.1: "If you would put one or more counters on a creature or
            // planeswalker you control or on yourself, put that many plus one instead."
            // ReplacementModification::AddExtraCounter adds +1 to each batch of counters placed.
            // ObjectFilter::ControlledBy(PlayerId(0)) covers permanents you control (creatures,
            // planeswalkers). The "or on yourself" (player counters) part is handled by
            // PlayerFilter::Any on the placer side; the receiver filter matches your permanents.
            // PlayerId(0) is a placeholder bound at registration.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldPlaceCounters {
                    placer_filter: PlayerFilter::Any,
                    receiver_filter: ObjectFilter::ControlledBy(PlayerId(0)),
                },
                modification: ReplacementModification::AddExtraCounter,
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}
