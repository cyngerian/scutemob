// Mox Opal — {0}, Legendary Artifact
// Metalcraft — {T}: Add one mana of any color. Activate only if you control
// three or more artifacts.
//
// CR 207.2c / CR 604.2 (Metalcraft): an ABILITY WORD, not a keyword -- CR 207.2c names
// it in its own list and says ability words have "no individual entries in the
// Comprehensive Rules". This line said CR 702.45a, which is BUSHIDO; corrected by
// PB-DX42b (`scutemob-233`, 2026-09-05). `OOS-DX42b-2`'s site list was a FLOOR: it
// named indomitable_archangel alone and this def carried the identical wrong cite twice.: The activation condition checks that you control
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
            // CR 604.2: Metalcraft -- only active when you control 3+ artifacts (see the header).
            activation_condition: Some(Condition::YouControlNOrMoreWithFilter {
                count: 3,
                filter: TargetFilter {
                    has_card_type: Some(CardType::Artifact),
                    ..Default::default()
                },
            }),

            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        // PB-EF12 (EF-W-PB2-3): un-marked, see birds_of_paradise.rs for the fix.
        ..Default::default()
    }
}
