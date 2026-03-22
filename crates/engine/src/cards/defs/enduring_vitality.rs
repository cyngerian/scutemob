// Enduring Vitality — {1}{G}{G}, Enchantment Creature — Elk Glimmer 3/3
// Vigilance
// Creatures you control have "{T}: Add one mana of any color."
// When Enduring Vitality dies, if it was a creature, return it to the battlefield
// under its owner's control. It's an enchantment. (It's not a creature.)
//
// TODO: Two DSL gaps:
//   (1) "Creatures you control have '{T}: Add one mana of any color.'" — no mechanism
//       to grant activated abilities via static effects (only keywords via AddKeyword).
//   (2) "When this dies, if it was a creature, return as enchantment" — Enduring cycle
//       die-return mechanic not in DSL (needs zone-change replacement + type change).
// Implementing only Vigilance keyword.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("enduring-vitality"),
        name: "Enduring Vitality".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 2, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment, CardType::Creature], &["Elk", "Glimmer"]),
        oracle_text: "Vigilance\nCreatures you control have \"{T}: Add one mana of any color.\"\nWhen Enduring Vitality dies, if it was a creature, return it to the battlefield under its owner's control. It's an enchantment. (It's not a creature.)".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            // TODO: grant "{T}: Add any color" to creatures you control (no GrantActivatedAbility in DSL)
            // TODO: die-return-as-enchantment (Enduring cycle mechanic not in DSL)
        ],
        ..Default::default()
    }
}
