// Pir, Imaginative Rascal — {2}{G}, Legendary Creature — Human 1/1
// Partner with Toothy, Imaginary Friend (ETB trigger handled by PartnerWith keyword).
// Counter-doubling replacement effect: "If one or more counters would be put on a permanent
// your team controls, that many plus one of each of those kinds of counters are put on that
// permanent instead." — CR 614.1 replacement effect.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("pir-imaginative-rascal"),
        name: "Pir, Imaginative Rascal".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human"]),
        oracle_text:
            "Partner with Toothy, Imaginary Friend (When this creature enters the battlefield, \
             target player may search their library for a card named Toothy, Imaginary Friend, \
             reveal it, put it into their hand, then shuffle.)\n\
             If one or more counters would be put on a permanent your team controls, that many \
             plus one of each of those kinds of counters are put on that permanent instead."
                .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 702.124j: "Partner with [name]" — ETB trigger searches for named partner.
            AbilityDefinition::Keyword(KeywordAbility::PartnerWith(
                "Toothy, Imaginary Friend".to_string(),
            )),
            // CR 122.6 / CR 614.1: Add one extra counter of each kind placed on
            // permanents you control. PlayerId(0) is a placeholder — bound at registration.
            // In Commander, "your team" = you.
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
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
    }
}
