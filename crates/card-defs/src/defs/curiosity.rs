// Curiosity — {U}, Enchantment — Aura
// Enchant creature
// Whenever enchanted creature deals damage to an opponent, you may draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("curiosity"),
        name: "Curiosity".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature\nWhenever enchanted creature deals damage to an opponent, \
                      you may draw a card."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
            // CR 510.3a / CR 603.2 (PB-DX36, `OOS-CARDS2-6`): "Whenever enchanted creature
            // deals damage to an opponent, you may draw a card." — combat_only: false
            // covers both combat and noncombat damage (now genuinely dispatched from
            // both GameEvent::CombatDamageDealt and GameEvent::DamageDealt via
            // rules/abilities.rs::queue_damage_source_triggers); recipient: Opponent
            // closes the "an opponent" approximation this ability used to carry.
            // TODO: costless "you may draw a card" — CR 603.3c optionality with NO cost
            // attached. `Effect::MayPayThenEffect` requires a `Cost` and a free one always
            // trivially pays (PB-DX35 rejected a `{0}` cost as dishonest); `MayPayOrElse`
            // discards its cost (`OOS-DX48-2`); `Effect::Choose` is non-interactive. The
            // draw is therefore UNCONDITIONAL here. Tracked corpus-wide as `OOS-DX35-5`.
            // The printed "an opponent" clause is NO LONGER a deviation — PB-DX36 shipped
            // `DamageRecipient::Opponent` and it is declared below.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEnchantedCreatureDealsDamageToPlayer {
                    combat_only: false,
                    recipient: DamageRecipient::Opponent,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::partial(
            "oracle says 'you MAY draw a card' but the draw is unconditional — no \
             costless-optional-effect expression exists in the DSL (PB-DX36, `OOS-DX48-2`: \
             `Effect::MayPayOrElse` discards its cost, `Effect::MayPayThenEffect` needs a real \
             `Cost`, a {0} cost was rejected as dishonest by PB-DX35).",
        ),
        ..Default::default()
    }
}
