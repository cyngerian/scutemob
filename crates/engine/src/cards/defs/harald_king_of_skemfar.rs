// Harald, King of Skemfar — {1}{B}{G}, Legendary Creature — Elf Warrior 3/2
// Menace; when Harald enters, look at top 5, may reveal Elf/Warrior/Tyvar to hand.
// TODO: ETB "look at top 5, may reveal [subtype or name] to hand" pattern.
// DSL gap: SearchLibrary / PutOnLibrary has no "look at top N, reveal one matching filter"
// pattern that handles multi-type OR filter (Elf OR Warrior OR named Tyvar).
// Deferred (count_threshold + multi_type_filter gap).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("harald-king-of-skemfar"),
        name: "Harald, King of Skemfar".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elf", "Warrior"],
        ),
        oracle_text: "Menace (This creature can't be blocked except by two or more creatures.)\nWhen Harald enters, look at the top five cards of your library. You may reveal an Elf, Warrior, or Tyvar card from among them and put it into your hand. Put the rest on the bottom of your library in a random order.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            // TODO: ETB — look at top 5, may reveal Elf/Warrior/Tyvar to hand, rest to bottom.
            // DSL gap: no look-at-top-N-choose-one with multi-subtype OR filter.
        ],
        ..Default::default()
    }
}
