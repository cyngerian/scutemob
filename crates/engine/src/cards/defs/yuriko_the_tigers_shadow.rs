// Yuriko, the Tiger's Shadow — {1}{U}{B}, Legendary Creature — Human Ninja
// Commander ninjutsu {U}{B}
// Whenever a Ninja you control deals combat damage to a player, reveal the top card of
// your library and put that card into your hand. Each opponent loses life equal to that
// card's mana value.
//
// PARTIAL: The combat damage trigger fires correctly filtered to Ninja creatures.
// RevealAndRoute puts the top card into hand correctly (all cards match → Hand dest).
// ENGINE-BLOCKED: "each opponent loses life equal to that card's mana value" requires
// EffectAmount::ManaValueOf pointing at the just-revealed card. No EffectTarget variant
// for the revealed card exists in the DSL. Life-loss clause omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("yuriko-the-tigers-shadow"),
        name: "Yuriko, the Tiger's Shadow".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Ninja"],
        ),
        oracle_text: "Commander ninjutsu {U}{B} ({U}{B}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand or the command zone tapped and attacking.)\nWhenever a Ninja you control deals combat damage to a player, reveal the top card of your library and put that card into your hand. Each opponent loses life equal to that card's mana value.".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::CommanderNinjutsu),
            AbilityDefinition::CommanderNinjutsu {
                cost: ManaCost { blue: 1, black: 1, ..Default::default() },
            },
            // Whenever a Ninja you control deals combat damage to a player, reveal the
            // top card of your library and put that card into your hand.
            // ENGINE-BLOCKED (life-loss clause): "each opponent loses life equal to that
            // card's mana value" — needs EffectAmount::ManaValueOf(revealed card), but no
            // EffectTarget variant for the card revealed by RevealAndRoute exists.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Ninja".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::RevealAndRoute {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                    filter: TargetFilter::default(), // all cards match — reveal the top card
                    matched_dest: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    unmatched_dest: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
