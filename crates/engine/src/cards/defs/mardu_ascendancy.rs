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
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks { filter: None },
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
            // Sacrifice Mardu Ascendancy: Creatures you control get +0/+3 until end of turn.
            AbilityDefinition::Activated {
                cost: Cost::SacrificeSelf,
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyToughness(3),
                        filter: EffectFilter::CreaturesYouControl,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
