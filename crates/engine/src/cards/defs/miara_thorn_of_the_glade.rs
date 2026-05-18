// Miara, Thorn of the Glade — {1}{B}, Legendary Creature — Elf Scout 1/2
// Whenever Miara or another Elf you control dies, you may pay {1} and 1 life.
// If you do, draw a card.
// Partner
//
// PARTIAL: Elf-filtered death trigger is authored (trigger condition correct).
// ENGINE-BLOCKED: "you may pay {1} and 1 life. If you do, draw a card." —
// this is a beneficial optional-pay rider. MayPayOrElse has TAX semantics
// ("if you DON'T pay, run or_else") and cannot express "if you DO pay, draw".
// No beneficial-optional-cost effect construct exists in the DSL.
// Trigger is kept without an effect body (Nothing) rather than unconditional draw.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("miara-thorn-of-the-glade"),
        name: "Miara, Thorn of the Glade".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elf", "Scout"],
        ),
        oracle_text: "Whenever Miara, Thorn of the Glade or another Elf you control dies, you may pay {1} and 1 life. If you do, draw a card.\nPartner (You can have two commanders if both have partner.)".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            // Trigger fires when Miara herself OR any Elf you control dies (exclude_self: false).
            // ENGINE-BLOCKED: "you may pay {1} and 1 life. If you do, draw a card." —
            // beneficial optional-pay-to-draw requires a MayPay { cost, if_paid } construct
            // that does not exist. MayPayOrElse expresses tax semantics only (if you don't pay,
            // run or_else). Unconditional draw would produce wrong game state — omitted.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: false,
                    nontoken_only: false,
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Elf".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::Nothing,
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Partner),
        ],
        ..Default::default()
    }
}
