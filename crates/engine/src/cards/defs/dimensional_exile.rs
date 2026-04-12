// Dimensional Exile — {1}{W} Enchantment — Aura
// Enchant basic land you control
// When Dimensional Exile enters the battlefield, exile target creature an opponent
// controls until Dimensional Exile leaves the battlefield.
//
// Same as Ossification but exiles creatures only (not planeswalkers).
// CR 205.4a: "basic land you control" — must have Basic supertype and controller constraint.
// CR 702.5a / 303.4a / 704.5m: cast-time and SBA enforcement.
// CR 610.3: ExileWithDelayedReturn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    let enchant_filter = EnchantFilter {
        has_card_type: Some(CardType::Land),
        basic: true,
        controller: EnchantControllerConstraint::You,
        ..Default::default()
    };
    CardDefinition {
        card_id: cid("dimensional-exile"),
        name: "Dimensional Exile".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text:
            "Enchant basic land you control\nWhen Dimensional Exile enters the battlefield, exile \
             target creature an opponent controls until Dimensional Exile leaves the battlefield."
                .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Filtered(
                enchant_filter,
            ))),
            // ETB: exile target creature an opponent controls until this leaves.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::ExileWithDelayedReturn {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    return_timing:
                        crate::state::stubs::DelayedTriggerTiming::WhenSourceLeavesBattlefield,
                    return_tapped: false,
                    return_to:
                        crate::cards::card_definition::DelayedReturnDestination::Battlefield,
                },
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
