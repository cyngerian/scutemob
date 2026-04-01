// Oath of Teferi — {3}{W}{U}, Legendary Enchantment
// When this enters, exile another target permanent you control. Return it at
// the beginning of the next end step.
// You may activate loyalty abilities of planeswalkers you control twice each turn
// rather than only once.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("oath-of-teferi"),
        name: "Oath of Teferi".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, blue: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Enchantment], &[]),
        oracle_text: "When Oath of Teferi enters, exile another target permanent you control. Return it to the battlefield under its owner's control at the beginning of the next end step.\nYou may activate loyalty abilities of planeswalkers you control twice each turn rather than only once.".to_string(),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::ExileWithDelayedReturn {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    return_tapped: false,
                    return_timing: crate::state::stubs::DelayedTriggerTiming::AtNextEndStep,
                    return_to: crate::cards::card_definition::DelayedReturnDestination::Battlefield,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                })],
                modes: None,
                trigger_zone: None,
            },
            // TODO: "activate loyalty abilities twice per turn" — no Permission for this.
        ],
        ..Default::default()
    }
}
