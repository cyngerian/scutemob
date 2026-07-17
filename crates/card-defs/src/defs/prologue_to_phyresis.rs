// Prologue to Phyresis — {1}{U}, Instant
// Each opponent gets a poison counter. Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("prologue-to-phyresis"),
        name: "Prologue to Phyresis".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Each opponent gets a poison counter.\nDraw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // TODO: "Each opponent gets a poison counter" — EffectTarget::AllOpponents
                //   doesn't exist. Need ForEach::EachOpponent or per-player counter.
                //   Using ForEach pattern.
                Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        counter: CounterType::Poison,
                        count: 1,
                    }),
                },
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        completeness: Completeness::known_wrong(
            "'Each opponent gets a poison counter' is a silent no-op. Effect::AddCounter has no \
             ResolvedTarget::Player arm (effects/mod.rs:2320-2346), and ForEach::EachOpponent \
             binds a Player target (:3104-3118), so the counter is discarded and only the draw \
             happens. No effect can grant a player a poison counter today (poison_counters is \
             written only by infect damage and Proliferate). Needs a player-counter effect before \
             this card is authorable.",
        ),
        ..Default::default()
    }
}
