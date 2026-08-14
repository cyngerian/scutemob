// Voldaren Epicure — {R}, Creature — Vampire 1/1
// When this creature enters, it deals 1 damage to each opponent. Create a Blood token.
// (It's an artifact with "{1}, {T}, Discard a card, Sacrifice this token: Draw a card.")
//
// CR 111.10g: Blood is a predefined artifact token type.
// CR 603.3: ETB trigger deals the damage and creates the token.
//
// PB-DX27 (2026-08-13), OOS-CARDS2-10: this def was `Complete` and deck-legal while
// SILENTLY DROPPING the first printed sentence — both from `oracle_text` and from the
// authored ability. The damage half is expressible and always was
// (`EffectTarget::EachOpponent`, executor `effects/mod.rs:7698`); nobody reached for it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("voldaren-epicure"),
        name: "Voldaren Epicure".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Vampire"]),
        oracle_text: "When this creature enters, it deals 1 damage to each opponent. Create a \
                      Blood token. (It's an artifact with \"{1}, {T}, Discard a card, Sacrifice \
                      this token: Draw a card.\")"
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 603.3: one ETB trigger carrying both printed clauses in printed order.
            // CR 119.3: "it deals 1 damage" — the source is this creature, which is
            // `ctx.source`, i.e. `source: None` (the existing default).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    Effect::DealDamage {
                        source: None,
                        target: EffectTarget::EachOpponent,
                        amount: EffectAmount::Fixed(1),
                    },
                    Effect::CreateToken {
                        spec: blood_token_spec(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    }
}
