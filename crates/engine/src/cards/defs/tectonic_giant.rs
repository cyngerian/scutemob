// Tectonic Giant — {2}{R}{R}, Creature — Elemental Giant 3/4
// Whenever this creature attacks or becomes the target of a spell an opponent controls, choose one —
// • This creature deals 3 damage to each opponent.
// • Exile the top two cards of your library. Choose one of them. Until the end of your
//   next turn, you may play that card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tectonic-giant"),
        name: "Tectonic Giant".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 2, ..Default::default() }),
        types: creature_types(&["Elemental", "Giant"]),
        oracle_text: "Whenever this creature attacks or becomes the target of a spell an opponent controls, choose one —\n• This creature deals 3 damage to each opponent.\n• Exile the top two cards of your library. Choose one of them. Until the end of your next turn, you may play that card.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            // TODO: Dual trigger condition (attacks OR becomes target of opponent's spell)
            // not in DSL. Only WhenAttacks fires currently.
            // CR 700.2b / PB-35: Modal triggered ability.
            // Mode 0: Deal 3 damage to each opponent.
            // Mode 1: Impulse draw (exile top 2, play 1) — DSL gap, Nothing placeholder.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::Nothing,
                intervening_if: None,
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    modes: vec![
                        // Mode 0: This creature deals 3 damage to each opponent.
                        Effect::ForEach {
                            over: ForEachTarget::EachOpponent,
                            effect: Box::new(Effect::DealDamage {
                                target: EffectTarget::DeclaredTarget { index: 0 },
                                amount: EffectAmount::Fixed(3),
                            }),
                        },
                        // Mode 1: Impulse draw (exile top 2, play 1 until end of next turn).
                        // DSL gap: no "exile top N, play one" effect. Nothing placeholder.
                        Effect::Nothing,
                    ],
                    allow_duplicate_modes: false,
                    mode_costs: None,
                }),
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
