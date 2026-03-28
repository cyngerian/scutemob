// Bonecrusher Giant // Stomp — {2}{R} Creature — Giant 4/3 + Adventure
//
// Main face: {2}{R} Giant 4/3
// "Whenever Bonecrusher Giant becomes the target of a spell, Bonecrusher Giant deals
//  2 damage to that spell's controller."
// Adventure face: "Stomp" {1}{R} Instant — Adventure
// "Damage can't be prevented this turn. Stomp deals 2 damage to any target."
//
// TODO(1): Trigger condition WhenBecomesTargetByOpponent is WRONG — should be
// WhenBecomesTargetBySpell (any spell, not just opponent spells). DSL gap.
// TODO(2): Effect should target "that spell's controller" — needs EffectTarget::TriggeringPlayer.
// TODO(3): "Damage can't be prevented this turn" — DSL gap: no Effect::PreventionShieldRemoval.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bonecrusher-giant-stomp"),
        name: "Bonecrusher Giant // Stomp".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types_sub(&[CardType::Creature], &["Giant"]),
        oracle_text: "Whenever Bonecrusher Giant becomes the target of a spell, Bonecrusher Giant deals 2 damage to that spell's controller.".to_string(),
        power: Some(4),
        toughness: Some(3),
        abilities: vec![
            // CR 603.5: Triggered ability — "Whenever Bonecrusher Giant becomes the target of
            // a spell, Bonecrusher Giant deals 2 damage to that spell's controller."
            // TODO(1): TriggerCondition::WhenBecomesTargetByOpponent is WRONG — fires only when
            // an opponent targets it. Oracle says "becomes the target of a spell" with no
            // controller restriction; trigger should fire for ANY spell (including one controlled
            // by the same player). Requires TriggerCondition::WhenBecomesTargetBySpell (DSL gap).
            // TODO(2): Effect target is WRONG — should deal 2 damage to "that spell's controller"
            // (the triggering player), not all opponents. Requires EffectTarget::TriggeringPlayer
            // DSL variant (does not exist yet).
            // TODO(3): "Damage can't be prevented this turn" — no Effect::PreventionShieldRemoval
            // DSL variant exists. Currently omitted.
            // Using EachOpponent as approximation until (1) and (2) are resolved.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenBecomesTargetByOpponent,
                effect: Effect::DealDamage {
                    target: EffectTarget::EachOpponent,
                    amount: EffectAmount::Fixed(2),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        // CR 715.2: Adventure face — Stomp.
        adventure_face: Some(CardFace {
            name: "Stomp".to_string(),
            mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
            types: TypeLine {
                card_types: [CardType::Instant].iter().copied().collect(),
                subtypes: [SubType("Adventure".to_string())]
                    .iter()
                    .cloned()
                    .collect(),
                supertypes: Default::default(),
            },
            oracle_text: "Damage can't be prevented this turn. Stomp deals 2 damage to any target.".to_string(),
            power: None,
            toughness: None,
            color_indicator: None,
            abilities: vec![AbilityDefinition::Spell {
                // CR 120.4a: Stomp deals 2 damage to any target.
                // TODO: "Damage can't be prevented this turn" — no prevention-removal DSL effect.
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(2),
                },
                targets: vec![TargetRequirement::TargetAny],
                modes: None,
                cant_be_countered: false,
            }],
        }),
        ..Default::default()
    }
}
