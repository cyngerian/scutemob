// Chained to the Rocks — {W} Enchantment — Aura
// Enchant Mountain you control
// When Chained to the Rocks enters the battlefield, exile target creature an opponent
// controls until Chained to the Rocks leaves the battlefield.
//
// CR 702.5a: "Enchant Mountain you control" — requires Mountain subtype and
//   aura controller must control the enchanted land.
// CR 303.4a / 704.5m: enforced at cast time and via Aura SBA.
// CR 610.3: ExileWithDelayedReturn — exile returns when source leaves battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    let enchant_filter = EnchantFilter {
        has_card_type: Some(CardType::Land),
        has_subtype: Some(SubType("Mountain".to_string())),
        controller: EnchantControllerConstraint::You,
        ..Default::default()
    };
    CardDefinition {
        card_id: cid("chained-to-the-rocks"),
        name: "Chained to the Rocks".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text:
            "Enchant Mountain you control\nWhen Chained to the Rocks enters the battlefield, \
             exile target creature an opponent controls until Chained to the Rocks leaves the battlefield."
                .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Filtered(
                enchant_filter,
            ))),
            // ETB: exile target creature opponent controls until this leaves.
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
