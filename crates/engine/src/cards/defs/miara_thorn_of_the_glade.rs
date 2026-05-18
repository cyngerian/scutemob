// Miara, Thorn of the Glade — {1}{B}, Legendary Creature — Elf Scout 1/2
// Whenever Miara or another Elf you control dies, you may pay {1} and 1 life.
// If you do, draw a card.
// Partner
//
// ENGINE-BLOCKED (death trigger): "Whenever Miara or another Elf you control dies"
// IS expressible (WheneverCreatureDies with an Elf filter), but the effect — "you may
// pay {1} and 1 life. If you do, draw a card." — is a beneficial optional-pay rider.
// MayPayOrElse has TAX semantics ("if you DON'T pay, run or_else") and cannot express
// "if you DO pay, draw". No beneficial-optional-cost construct exists in the DSL.
// The whole triggered ability is omitted (rather than a do-nothing trigger or an
// unconditional draw) — consistent with crossway_troublemakers.
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
            // ENGINE-BLOCKED: "Whenever Miara or another Elf you control dies, you may
            // pay {1} and 1 life. If you do, draw a card." — the trigger condition is
            // expressible (WheneverCreatureDies + Elf filter, exclude_self: false), but
            // the beneficial optional-pay-to-draw rider has no DSL construct. MayPayOrElse
            // is tax semantics only. The triggered ability is omitted entirely.
            AbilityDefinition::Keyword(KeywordAbility::Partner),
        ],
        ..Default::default()
    }
}
