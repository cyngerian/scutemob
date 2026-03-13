// Elesh Norn, Mother of Machines — {4}{W}, Legendary Creature — Phyrexian Praetor 4/7
// Vigilance
// TODO: DSL gap — static ability "If a permanent entering causes a triggered ability of a
//   permanent you control to trigger, that ability triggers an additional time."
//   (ETB trigger doubling not supported in card DSL)
// TODO: DSL gap — static ability "Permanents entering don't cause abilities of permanents
//   your opponents control to trigger."
//   (ETB trigger suppression for opponents not supported in card DSL)
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
        ],
        ..Default::default()
    }
}
