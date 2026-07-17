// Golgari Findbroker — {B}{B}{G}{G}, Creature — Elf Shaman 3/4
// When this creature enters, return target permanent card from your graveyard to your hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("golgari-findbroker"),
        name: "Golgari Findbroker".to_string(),
        mana_cost: Some(ManaCost {
            black: 2,
            green: 2,
            ..Default::default()
        }),
        types: creature_types(&["Elf", "Shaman"]),
        oracle_text: "When this creature enters, return target permanent card from your graveyard \
                      to your hand."
            .to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            // CR 603.3: ETB trigger — return target permanent card from your graveyard to hand.
            // TargetFilter uses OR semantics on has_card_types to match any permanent-type card
            // (creature, artifact, battle, enchantment, land, planeswalker). Instants and
            // sorceries are excluded.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Hand {
                        owner: PlayerTarget::Controller,
                    },
                    controller_override: None,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_types: vec![
                        CardType::Creature,
                        CardType::Artifact,
                        CardType::Battle,
                        CardType::Enchantment,
                        CardType::Land,
                        CardType::Planeswalker,
                    ],
                    ..Default::default()
                })],
                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
