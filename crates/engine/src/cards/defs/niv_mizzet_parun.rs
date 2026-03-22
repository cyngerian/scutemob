// Niv-Mizzet, Parun — {U}{U}{U}{R}{R}{R}, Legendary Creature — Dragon Wizard 5/5
// This spell can't be countered.
// Flying
// Whenever you draw a card, Niv-Mizzet deals 1 damage to any target.
// Whenever a player casts an instant or sorcery spell, you draw a card.
//
// NOTE: "Whenever a player casts an instant or sorcery spell, you draw a card" requires a
// TriggerCondition that fires for ANY player casting instant/sorcery (not just you). No such
// variant exists in the DSL (WheneverYouCastSpell only covers controller's own casts). Omitted
// per W5 policy until a WheneverAnyPlayerCastsInstantOrSorcery trigger is added.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("niv-mizzet-parun"),
        name: "Niv-Mizzet, Parun".to_string(),
        mana_cost: Some(ManaCost { blue: 3, red: 3, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "Wizard"],
        ),
        oracle_text: "This spell can't be countered.\nFlying\nWhenever you draw a card, Niv-Mizzet deals 1 damage to any target.\nWhenever a player casts an instant or sorcery spell, you draw a card.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // Whenever you draw a card, Niv-Mizzet deals 1 damage to any target.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouDrawACard,
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetAny],
            },
            // TODO: "Whenever a player casts an instant or sorcery spell, you draw a card" —
            // requires TriggerCondition::WheneverAnyPlayerCastsInstantOrSorcery which does not
            // exist in the DSL. WheneverYouCastSpell only covers the controller's own casts.
            // Omitted per W5 policy.
        ],
        // "This spell can't be countered" is a property of the spell on the stack.
        // The cant_be_countered field is on AbilityDefinition::Spell, not on CardDefinition.
        // This card is a permanent, not a spell ability — no Spell ability to mark. Per W5,
        // cant_be_countered for permanents is a known DSL gap (no field on CardDefinition).
        // TODO: Add cant_be_countered: bool field to CardDefinition to handle permanents.
        ..Default::default()
    }
}
