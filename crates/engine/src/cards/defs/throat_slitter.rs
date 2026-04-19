// Throat Slitter — {4}{B}, Creature — Rat Ninja 2/2
// Ninjutsu {2}{B} ({2}{B}, Return an unblocked attacker you control to hand: Put this card
//   onto the battlefield from your hand tapped and attacking.)
// Whenever this creature deals combat damage to a player, destroy target nonblack creature
//   that player controls.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("throat-slitter"),
        name: "Throat Slitter".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 1, ..Default::default() }),
        types: creature_types(&["Rat", "Ninja"]),
        oracle_text: "Ninjutsu {2}{B} ({2}{B}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)\nWhenever this creature deals combat damage to a player, destroy target nonblack creature that player controls.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost { generic: 2, black: 1, ..Default::default() },
            },
            // CR 510.3a: "Whenever this deals combat damage to a player, destroy target nonblack
            // creature that player controls." — DamagedPlayer scopes the target to the specific
            // player dealt damage (precision fix: multiplayer correctness).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    exclude_colors: Some([Color::Black].iter().copied().collect()),
                    controller: TargetController::DamagedPlayer,
                    ..Default::default()
                })],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
