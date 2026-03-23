// Dionus, Elvish Archdruid — {3}{G}, Legendary Creature — Elf Druid 3/3
// Elves you control have "Whenever this creature becomes tapped during your turn, untap it
// and put a +1/+1 counter on it. This ability triggers only once each turn."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dionus-elvish-archdruid"),
        name: "Dionus, Elvish Archdruid".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elf", "Druid"],
        ),
        oracle_text: "Elves you control have \"Whenever this creature becomes tapped during your turn, untap it and put a +1/+1 counter on it. This ability triggers only once each turn.\"".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // TODO: DSL gap — granting triggered abilities to other creatures. Static grant
            // of a triggered ability (GrantTriggeredAbility) not in DSL.
        ],
        ..Default::default()
    }
}
