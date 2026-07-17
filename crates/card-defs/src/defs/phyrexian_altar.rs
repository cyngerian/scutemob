// Phyrexian Altar — {3}, Artifact
// "Sacrifice a creature: Add one mana of any color."
// CR 602.2: Activated ability with sacrifice cost. Mana ability (CR 605.1a).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("phyrexian-altar"),
        name: "Phyrexian Altar".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Sacrifice a creature: Add one mana of any color.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Sacrifice(TargetFilter {
                has_card_type: Some(CardType::Creature),
                ..Default::default()
            }),
            effect: Effect::AddManaAnyColor {
                player: PlayerTarget::Controller,
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        completeness: Completeness::known_wrong(
            "CR 106.1b: 'Sacrifice a creature: Add one mana of any color' adds one COLORLESS mana \
             (probed: +1 colorless via the stack). Colorless is not a color, so this is wrong \
             state, not an omitted clause. Additionally a CR 605.1a/605.3b violation: it is a \
             mana ability but uses the stack, blocked on the Cost::Sacrifice(filter) ObjectId \
             channel (see Ashnod's Altar). The color bug is the reason for known_wrong rather \
             than partial.",
        ),
        ..Default::default()
    }
}
