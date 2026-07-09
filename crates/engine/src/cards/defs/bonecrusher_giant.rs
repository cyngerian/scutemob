// Bonecrusher Giant // Stomp — {2}{R} Creature — Giant 4/3 + Adventure
//
// Main face: {2}{R} Giant 4/3
// "Whenever Bonecrusher Giant becomes the target of a spell, Bonecrusher Giant deals
//  2 damage to that spell's controller."
// Adventure face: "Stomp" {1}{R} Instant — Adventure
// "Damage can't be prevented this turn. Stomp deals 2 damage to any target."
//
// PB-AC6 fixed the trigger condition: it is now TriggerCondition::WhenBecomesTarget
// { scope: None (the source itself), by_opponent: false (any controller), include_abilities:
// false (spells only) }, which matches the oracle exactly. The previous
// WhenBecomesTargetByOpponent is the Ward-only condition and fired only on opponents' spells.
//
// ENGINE-BLOCKED(1): the effect must deal 2 damage to "that spell's controller".
// Effect::DealDamage takes an EffectTarget, and EffectTarget has no ControllerOf /
// TriggeringPlayer variant (both exist only on PlayerTarget). The previous
// EffectTarget::EachOpponent was a wrong-game-state approximation: in multiplayer it damaged
// every opponent, not just the caster, and it damaged opponents even when the controller
// targeted their own Bonecrusher Giant. Replaced with Effect::Nothing rather than approximated.
// ENGINE-BLOCKED(2): "Damage can't be prevented this turn" — no prevention-removal Effect.
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
            // CR 603.2 / 601.2c: Triggered ability — "Whenever Bonecrusher Giant becomes the
            // target of a spell, Bonecrusher Giant deals 2 damage to that spell's controller."
            // The trigger condition is exact (PB-AC6). The effect stays unauthored: see
            // ENGINE-BLOCKED(1) in the file header — "that spell's controller" is not an
            // expressible EffectTarget, and the old EachOpponent approximation produced wrong
            // game state in multiplayer.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenBecomesTarget {
                    scope: None,
                    by_opponent: false,
                    include_abilities: false,
                },
                effect: Effect::Nothing,
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
