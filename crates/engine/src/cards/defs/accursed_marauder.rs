// Accursed Marauder — {1}{B}, Creature — Zombie 2/1
// When this enters, each player sacrifices a nontoken creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("accursed-marauder"),
        name: "Accursed Marauder".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Zombie"]),
        oracle_text: "When this enters, each player sacrifices a nontoken creature.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            // CR 603.3: ETB trigger — each player sacrifices a nontoken creature.
            // PB-SFT (CR 701.17a + CR 109.1c): nontoken creature filter.
            // `is_nontoken` is a runtime GameObject field checked explicitly at the
            // SacrificePermanents resolution site (not in matches_filter).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::SacrificePermanents {
                    player: PlayerTarget::EachPlayer,
                    count: EffectAmount::Fixed(1),
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        is_nontoken: true,
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
