// Mardu Ascendancy — {R}{W}{B}, Enchantment
// Whenever a nontoken creature you control attacks, create a 1/1 red Goblin creature
// token that's tapped and attacking.
// Sacrifice this enchantment: Creatures you control get +0/+3 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mardu-ascendancy"),
        name: "Mardu Ascendancy".to_string(),
        mana_cost: Some(ManaCost { red: 1, white: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a nontoken creature you control attacks, create a 1/1 red Goblin creature token that's tapped and attacking.\nSacrifice this enchantment: Creatures you control get +0/+3 until end of turn.".to_string(),
        abilities: vec![
            // CR 508.1m: "Whenever a nontoken creature you control attacks, create a 1/1 red
            // Goblin token tapped and attacking."
            // PB-23: WheneverCreatureYouControlAttacks.
            // TODO: Nontoken filter not yet in DSL for attack triggers — over-triggers on token
            // attackers (including Goblin tokens created by this ability itself).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Goblin".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: [Color::Red].into_iter().collect(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
                        count: 1,
                        tapped: true,
                        enters_attacking: true,
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: Sacrifice activated ability with +0/+3 buff to all creatures you control —
            // DSL ModifyBoth(3) would be +3/+3, not +0/+3.
        ],
        ..Default::default()
    }
}
