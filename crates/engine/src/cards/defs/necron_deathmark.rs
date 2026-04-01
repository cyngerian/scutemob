// Necron Deathmark — {3}{B}, Creature — Necron 4/2
// Flash
// When this enters, destroy target creature an opponent controls. Each player mills two cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("necron-deathmark"),
        name: "Necron Deathmark".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Necron"]),
        oracle_text: "Flash\nWhen this enters, destroy target creature an opponent controls. Each player mills two cards.".to_string(),
        power: Some(4),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    // Destroy target creature an opponent controls.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        cant_be_regenerated: false,
                    },
                    // Each player mills two cards.
                    Effect::MillCards {
                        player: PlayerTarget::EachPlayer,
                        count: EffectAmount::Fixed(2),
                    },
                ]),
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
