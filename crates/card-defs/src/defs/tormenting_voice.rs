// Tormenting Voice — {1}{R}, Sorcery
// As an additional cost to cast this spell, discard a card.
// Draw two cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tormenting-voice"),
        name: "Tormenting Voice".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "As an additional cost to cast this spell, discard a card.\nDraw two cards."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(2),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        // TODO: AdditionalCost::DiscardCard not wired to Spell definition yet.
        completeness: Completeness::known_wrong(
            "'As an additional cost, discard a card' is silently dropped — the spell resolves as \
             an unconditional Draw 2 for {1}{R}. SpellAdditionalCost (card_definition.rs:1265) \
             has only Sacrifice* variants; no Discard variant exists, so a CardDefinition cannot \
             declare this cost.",
        ),
        ..Default::default()
    }
}
