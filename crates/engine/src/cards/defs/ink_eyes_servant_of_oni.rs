// Ink-Eyes, Servant of Oni — {4}{B}{B}, Legendary Creature — Rat Ninja 5/4
// Ninjutsu {3}{B}{B}
// Whenever Ink-Eyes deals combat damage to a player, you may put target creature
// card from that player's graveyard onto the battlefield under your control.
// {1}{B}: Regenerate Ink-Eyes.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ink-eyes-servant-of-oni"),
        name: "Ink-Eyes, Servant of Oni".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Rat", "Ninja"]),
        oracle_text: "Ninjutsu {3}{B}{B} ({3}{B}{B}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)\nWhenever Ink-Eyes, Servant of Oni deals combat damage to a player, you may put target creature card from that player's graveyard onto the battlefield under your control.\n{1}{B}: Regenerate Ink-Eyes.".to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost { generic: 3, black: 2, ..Default::default() },
            },
            // Combat damage trigger: reanimate from opponent's graveyard.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                // TODO: "target creature from that player's graveyard" — needs
                // TargetCardInOpponentGraveyard + "under your control" zone move.
                effect: Effect::Nothing,
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCardInGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                })],
                modes: None,
                trigger_zone: None,
            },
            // {1}{B}: Regenerate
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 1, black: 1, ..Default::default() }),
                effect: Effect::Regenerate {
                    target: EffectTarget::Source,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
