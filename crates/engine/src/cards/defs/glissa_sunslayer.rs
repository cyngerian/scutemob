// Glissa Sunslayer — {1}{B}{G}, Legendary Creature — Phyrexian Zombie Elf 3/3
// First strike, deathtouch
// Whenever Glissa Sunslayer deals combat damage to a player, choose one —
// • You draw a card and lose 1 life.
// • Destroy target enchantment.
// • Remove up to three counters from target permanent.
// TODO: DSL gap — mode 2 "remove up to three counters from target permanent" cannot be
// fully implemented. Effect::RemoveCounter (singular) exists in card_definition.rs but
// requires a specific CounterType; Glissa's ability removes "any type" of counter without
// specifying which type. There is no any-type or multi-type counter removal effect, and
// no "up to N" count modifier. Mode 2 uses a no-op placeholder; modes 0 and 1 work correctly.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("glissa-sunslayer"),
        name: "Glissa Sunslayer".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Phyrexian", "Zombie", "Elf"],
        ),
        oracle_text: "First strike, deathtouch\nWhenever Glissa Sunslayer deals combat damage to a player, choose one —\n• You draw a card and lose 1 life.\n• Destroy target enchantment.\n• Remove up to three counters from target permanent.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            // Whenever Glissa Sunslayer deals combat damage to a player, choose one —
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::Choose {
                    prompt: "Choose one — draw a card and lose 1 life; or destroy target enchantment; or remove up to three counters from target permanent".to_string(),
                    choices: vec![
                        // Mode 0: You draw a card and lose 1 life.
                        Effect::Sequence(vec![
                            Effect::DrawCards {
                                player: PlayerTarget::Controller,
                                count: EffectAmount::Fixed(1),
                            },
                            Effect::LoseLife {
                                player: PlayerTarget::Controller,
                                amount: EffectAmount::Fixed(1),
                            },
                        ]),
                        // Mode 1: Destroy target enchantment.
                        Effect::DestroyPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                        },
                        // Mode 2: Remove up to three counters from target permanent.
                        // TODO: DSL gap — RemoveCounter requires a specific CounterType; this
                        // ability removes "any type" of counter (up to 3) which cannot be expressed.
                        // No-op placeholder; choosing this mode does nothing.
                        Effect::Sequence(vec![]),
                    ],
                },
                intervening_if: None,
                targets: vec![
                    // index 0: target enchantment (mode 1)
                    // index 1: target permanent for counter removal (mode 2)
                    TargetRequirement::TargetEnchantment,
                    TargetRequirement::TargetPermanent,
                ],
            },
        ],
        ..Default::default()
    }
}
