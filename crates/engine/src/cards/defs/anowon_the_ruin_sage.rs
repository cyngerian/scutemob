// Anowon, the Ruin Sage — {3}{B}{B}, Legendary Creature — Vampire Shaman 4/3
// At the beginning of your upkeep, each player sacrifices a non-Vampire creature.
//
// TODO: SacrificePermanents has no creature-type exclusion filter — it picks the lowest-ID
// permanent rather than specifically a non-Vampire creature. The "non-Vampire" constraint
// cannot be expressed in the current DSL. This is a known gap (same issue as Butcher of Malakir).
// Engine picks any permanent; in a well-populated game this will often be a creature but
// may be wrong in edge cases. Noted here to fix when SacrificePermanents gets type filters.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("anowon-the-ruin-sage"),
        name: "Anowon, the Ruin Sage".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Vampire", "Shaman"]),
        oracle_text: "At the beginning of your upkeep, each player sacrifices a non-Vampire creature.".to_string(),
        power: Some(4),
        toughness: Some(3),
        abilities: vec![
            // CR 603.6d: "At the beginning of your upkeep, each player sacrifices a non-Vampire creature."
            // AtBeginningOfYourUpkeep + SacrificePermanents(EachPlayer, 1).
            // TODO: "non-Vampire creature" filter — SacrificePermanents has no type exclusion.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::SacrificePermanents {
                    player: PlayerTarget::EachPlayer,
                    count: EffectAmount::Fixed(1),
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
