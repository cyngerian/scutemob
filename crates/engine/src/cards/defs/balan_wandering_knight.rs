// Balan, Wandering Knight — {2}{W}{W}, Legendary Creature — Cat Knight 3/3
// First strike
// Balan has double strike as long as two or more Equipment are attached to it.
// {1}{W}: Attach all Equipment you control to Balan.
//
// First strike is implemented.
// TODO: DSL gap — conditional double strike requiring count of attached Equipment (2+)
//   needs a Condition::AttachedEquipmentCount threshold static — not in DSL
// TODO: DSL gap — {1}{W} activated ability attaching all Equipment you control to this
//   creature — no "attach all equipment" mass-equip effect in DSL
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("balan-wandering-knight"),
        name: "Balan, Wandering Knight".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Cat", "Knight"]),
        oracle_text: "First strike\nBalan has double strike as long as two or more Equipment are attached to it.\n{1}{W}: Attach all Equipment you control to Balan.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            // TODO: Conditional double strike (2+ Equipment attached) — count threshold static
            // TODO: {1}{W} activated — attach all Equipment you control to Balan
        ],
        ..Default::default()
    }
}
