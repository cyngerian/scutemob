// Volatile Stormdrake — {1}{U}, Creature — Drake 3/2
// Flying, hexproof from activated and triggered abilities
// ETB: exchange control with target opponent's creature; pay {E} equal to mana value or sacrifice
// TODO: "hexproof from activated and triggered abilities" is a partial protection variant not in DSL
//       ETB exchange-control and energy payment mechanics not in DSL
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("volatile-stormdrake"),
        name: "Volatile Stormdrake".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Drake"]),
        oracle_text: "Flying, hexproof from activated and triggered abilities\nWhen this creature enters, exchange control of this creature and target creature an opponent controls. If you do, you get {E}{E}{E}{E}, then sacrifice that creature unless you pay an amount of {E} equal to its mana value.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "Hexproof from activated and triggered abilities" is a partial-source
            // hexproof variant not representable as KeywordAbility::Hexproof (which is total).
            // TODO: ETB trigger exchanging control and energy counter mechanics not in DSL.
        ],
        ..Default::default()
    }
}
