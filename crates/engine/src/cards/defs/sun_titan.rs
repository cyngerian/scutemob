// Sun Titan — {4}{W}{W}, Creature — Giant 6/6
// Vigilance
// Whenever Sun Titan enters or attacks, you may return target permanent card with mana
// value 3 or less from your graveyard to the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sun-titan"),
        name: "Sun Titan".to_string(),
        mana_cost: Some(ManaCost { generic: 4, white: 2, ..Default::default() }),
        types: creature_types(&["Giant"]),
        oracle_text: "Vigilance\nWhenever Sun Titan enters or attacks, you may return target permanent card with mana value 3 or less from your graveyard to the battlefield.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            // ETB trigger: return permanent card with MV 3 or less from GY to battlefield.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Battlefield { tapped: false },
                    controller_override: Some(PlayerTarget::Controller),
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    max_cmc: Some(3),
                    has_card_types: vec![
                        CardType::Creature, CardType::Artifact, CardType::Enchantment,
                        CardType::Planeswalker, CardType::Land,
                    ],
                    ..Default::default()
                })],
                modes: None,
                trigger_zone: None,
            },
            // Attack trigger: same effect.
            // TODO: TriggerCondition::WhenAttacks not yet available for self-attack triggers.
            // When implemented, duplicate the above trigger with WhenAttacks condition.
        ],
        ..Default::default()
    }
}
