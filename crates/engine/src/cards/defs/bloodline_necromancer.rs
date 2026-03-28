// Bloodline Necromancer — {4}{B}, Creature — Vampire Wizard 3/2
// Lifelink
// When this creature enters, you may return target Vampire or Wizard creature card
// from your graveyard to the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloodline-necromancer"),
        name: "Bloodline Necromancer".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Wizard"]),
        oracle_text: "Lifelink\nWhen this creature enters, you may return target Vampire or Wizard creature card from your graveyard to the battlefield.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            // CR 603.1: ETB trigger — return target Vampire or Wizard creature from your GY to BF.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Battlefield { tapped: false },
                    controller_override: None,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    has_subtypes: vec![SubType("Vampire".to_string()), SubType("Wizard".to_string())],
                    ..Default::default()
                })],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
