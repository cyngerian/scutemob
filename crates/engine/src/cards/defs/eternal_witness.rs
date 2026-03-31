// Eternal Witness — {1}{G}{G}, Creature — Human Shaman 2/1
// When this creature enters, you may return target card from your graveyard to your hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("eternal-witness"),
        name: "Eternal Witness".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 2, ..Default::default() }),
        types: creature_types(&["Human", "Shaman"]),
        oracle_text: "When this creature enters, you may return target card from your graveyard to your hand.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    controller_override: None,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    ..Default::default()
                })],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
