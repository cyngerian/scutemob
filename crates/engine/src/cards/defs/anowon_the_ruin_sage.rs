// Anowon, the Ruin Sage — {3}{B}{B}, Legendary Creature — Vampire Shaman 4/3
// At the beginning of your upkeep, each player sacrifices a non-Vampire creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("anowon-the-ruin-sage"),
        name: "Anowon, the Ruin Sage".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Vampire", "Shaman"]),
        oracle_text: "At the beginning of your upkeep, each player sacrifices a non-Vampire creature of their choice.".to_string(),
        power: Some(4),
        toughness: Some(3),
        abilities: vec![
            // CR 603.6d: "At the beginning of your upkeep, each player sacrifices a non-Vampire creature."
            // PB-SFT (CR 701.21a + CR 109.1): creature filter with Vampire subtype exclusion.
            // Note: trigger fires on controller's upkeep only (AtBeginningOfYourUpkeep fires for
            // the controller). "Each player" via EachPlayer still applies to all players' choices.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::SacrificePermanents {
                    player: PlayerTarget::EachPlayer,
                    count: EffectAmount::Fixed(1),
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        exclude_subtypes: vec![SubType("Vampire".to_string())],
                        ..Default::default()
                    }),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
