// Necropanther — {1}{W}{B}, Creature — Cat Nightmare 3/3
// Mutate {2}{W/B}{W/B}
// Whenever this creature mutates, return target creature card with mana value 3 or less
// from your graveyard to the battlefield.
//
// CR 702.140a: Mutate is an alternative cost targeting a non-Human creature you own.
// CR 702.140d: "Whenever this creature mutates" fires after a successful merge.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("necropanther"),
        name: "Necropanther".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Cat", "Nightmare"]),
        oracle_text: "Mutate {2}{W/B}{W/B} (If you cast this spell for its mutate cost, put it over or under target non-Human creature you own. They mutate into the creature on top plus all abilities from under it.)\nWhenever this creature mutates, return target creature card with mana value 3 or less from your graveyard to the battlefield.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // CR 702.140a: Mutate keyword marker for presence-checking.
            AbilityDefinition::Keyword(KeywordAbility::Mutate),
            // CR 702.140a: Mutate cost {2}{W/B}{W/B}.
            AbilityDefinition::MutateCost {
                cost: ManaCost {
                    generic: 2,
                    hybrid: vec![
                        HybridMana::ColorColor(ManaColor::White, ManaColor::Black),
                        HybridMana::ColorColor(ManaColor::White, ManaColor::Black),
                    ],
                    ..Default::default()
                },
            },
            // CR 702.140d: "Whenever this creature mutates, return target creature card with
            // mana value 3 or less from your graveyard to the battlefield."
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenMutates,
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Battlefield { tapped: false },
                    controller_override: Some(PlayerTarget::Controller),
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    max_cmc: Some(3),
                    ..Default::default()
                })],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
