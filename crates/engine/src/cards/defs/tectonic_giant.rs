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
            // TODO: Dual trigger condition (attacks OR becomes target of opponent's spell).
            // TODO: Modal triggered ability — choose one of two modes at resolution.
            // Mode 1: deal 3 to each opponent. Mode 2: impulse draw (exile top 2, play 1).
            // Partial: implement the attack trigger with damage mode only.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(3),
                    }),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
