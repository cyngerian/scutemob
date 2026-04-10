// Elesh Norn, Mother of Machines — {4}{W}, Legendary Creature — Phyrexian Praetor 4/7
// Vigilance
// If a permanent entering causes a triggered ability of a permanent you control to
// trigger, that ability triggers an additional time.
// Permanents entering don't cause abilities of permanents your opponents control to trigger.
//
// CR 603.2d: The ETB trigger doubling (AnyPermanentETB) is now implemented via PB-M.
// TODO: DSL gap — "Permanents entering don't cause abilities of permanents your opponents
//   control to trigger." (controller-scoped ETB suppression for opponents not yet supported;
//   requires a new ETBSuppressor variant scoped to opposing controllers. Separate PB needed.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elesh-norn-mother-of-machines"),
        name: "Elesh Norn, Mother of Machines".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            white: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Phyrexian", "Praetor"],
        ),
        oracle_text: "Vigilance\nIf a permanent entering causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.\nPermanents entering don't cause abilities of permanents your opponents control to trigger.".to_string(),
        power: Some(4),
        toughness: Some(7),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            // CR 603.2d: If a permanent entering causes a triggered ability of a permanent
            // you control to trigger, that ability triggers an additional time.
            AbilityDefinition::TriggerDoubling {
                filter: TriggerDoublerFilter::AnyPermanentETB,
                additional_triggers: 1,
            },
        ],
        ..Default::default()
    }
}
