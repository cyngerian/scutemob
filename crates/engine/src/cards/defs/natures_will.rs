// Nature's Will — {2}{G}{G}, Enchantment
// Whenever one or more creatures you control deal combat damage to a player,
// tap all lands that player controls and untap all lands you control.
//
// TODO: "tap all lands that player controls" — ForEach over DamagedPlayer's lands requires
//   TargetController::DamagedPlayer support in ForEach filters. Deferred to PB-37.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("natures-will"),
        name: "Nature's Will".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever one or more creatures you control deal combat damage to a player, tap all lands that player controls and untap all lands you control.".to_string(),
        abilities: vec![
            // CR 510.3a / CR 603.2c: Partial fix — "untap all lands you control" implemented;
            // "tap all lands that player controls" deferred (needs DamagedPlayer ForEach filter).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer { filter: None },
                effect: Effect::ForEach {
                    over: ForEachTarget::EachPermanentMatching(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    effect: Box::new(Effect::UntapPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    }),
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: "tap all lands that player controls" — DamagedPlayer ForEach not in DSL.
        ],
        ..Default::default()
    }
}
