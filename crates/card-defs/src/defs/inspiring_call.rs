// Inspiring Call — {2}{G}, Instant
// Draw a card for each creature you control with a +1/+1 counter on it.
// Those creatures gain indestructible until end of turn.
//
// TODO: "Draw for each creature with +1/+1 counter" — EffectAmount lacks
//   PermanentCountWithCounter variant. Using PermanentCount approximation.
// TODO: "Those creatures gain indestructible until EOT" — needs targeted grant.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("inspiring-call"),
        name: "Inspiring Call".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Draw a card for each creature you control with a +1/+1 counter on it. Those creatures gain indestructible until end of turn.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: Count creatures with +1/+1 counters — approximation using all creatures.
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::PermanentCount {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::You,
                        ..Default::default()
                    },
                    controller: PlayerTarget::Controller,
                },
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        completeness: Completeness::partial("Draw count must be rewired to PermanentCount { filter: { has_card_type: Creature, controller: You, has_counter_type: Some(PlusOnePlusOne) } } (armorcraft_judge.rs precedent) — the current def draws for ALL your creatures, which is wrong. Remaining blocker: no EffectFilter variant selects 'creatures you control with a +1/+1 counter' for the indestructible grant."),
        ..Default::default()
    }
}
