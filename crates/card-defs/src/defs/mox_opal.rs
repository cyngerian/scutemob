// Mox Opal — {0}, Legendary Artifact
// Metalcraft — {T}: Add one mana of any color. Activate only if you control
// three or more artifacts.
//
// CR 702.45a (Metalcraft ability word): The activation condition checks that you control
// 3+ artifacts. Using Condition::YouControlNOrMoreWithFilter with count: 3 and
// has_card_type: Some(CardType::Artifact).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mox-opal"),
        name: "Mox Opal".to_string(),
        mana_cost: Some(ManaCost {
            ..Default::default()
        }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Artifact]),
        oracle_text: "Metalcraft — {T}: Add one mana of any color. Activate only if you control \
                      three or more artifacts."
            .to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddManaAnyColor {
                player: PlayerTarget::Controller,
            },
            timing_restriction: None,
            targets: vec![],
            // CR 702.45a: Metalcraft — only active when you control 3+ artifacts.
            activation_condition: Some(Condition::YouControlNOrMoreWithFilter {
                count: 3,
                filter: TargetFilter {
                    has_card_type: Some(CardType::Artifact),
                    ..Default::default()
                },
            }),

            activation_zone: None,
            once_per_turn: false,
        }],
        completeness: Completeness::known_wrong(
            "SF-11 (CR 106.1a/106.1b): this card's \"any color\" mana resolves to one COLORLESS \
             mana (Effect::AddManaAnyColor family; effects/mod.rs and handle_tap_for_mana step 8 \
             both add ManaColor::Colorless). Colorless is a mana TYPE, not a color, so {C} is \
             outside the option set \"any color\" offers — wrong game state, not an omitted \
             clause. Un-mark when a color channel for any-color mana lands (SR-37 / scutemob-93).",
        ),
        ..Default::default()
    }
}
