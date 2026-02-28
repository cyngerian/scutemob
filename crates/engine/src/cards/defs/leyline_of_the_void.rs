// 53. Leyline of the Void — {2BB}, Enchantment.
// "If Leyline of the Void is in your opening hand, you may begin the game
// with it on the battlefield. If a card an opponent owns would be put into
// that player's graveyard from anywhere, exile it instead."
//
// Simplification: The "opening hand" leyline rule is not modelled — Leyline
// enters play normally when cast. The opponent-only filter uses
// ObjectFilter::OwnedByOpponentsOf with a placeholder PlayerId(0); the
// registration function (register_permanent_replacement_abilities) binds the
// actual controller's PlayerId at registration time (MR-M8-09).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("leyline-of-the-void"),
        name: "Leyline of the Void".to_string(),
        mana_cost: Some(ManaCost { black: 2, generic: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text:
            "If Leyline of the Void is in your opening hand, you may begin the game with it on the battlefield.\n\
             If a card an opponent owns would be put into that player's graveyard from anywhere, exile it instead."
                .to_string(),
        abilities: vec![
            // CR 113.6b: If Leyline of the Void is in your opening hand, you may begin
            // the game with it on the battlefield. Handled by start_game pre-game check.
            AbilityDefinition::OpeningHand,
            // CR 614.1a: Replacement — opponent-owned cards going to graveyard → exile.
            // PlayerId(0) is a placeholder bound to the actual controller at
            // registration time by register_permanent_replacement_abilities.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldChangeZone {
                    from: None,
                    to: ZoneType::Graveyard,
                    filter: ObjectFilter::OwnedByOpponentsOf(PlayerId(0)),
                },
                modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
                is_self: false,
            },
        ],
        ..Default::default()
    }
}
