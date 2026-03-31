// Yuriko, the Tiger's Shadow — {1}{U}{B}, Legendary Creature — Human Ninja
// Commander ninjutsu {U}{B}
// Whenever a Ninja you control deals combat damage to a player, reveal top card of
// your library, put it in hand. Each opponent loses life equal to that card's mana value.
//
// TODO: Ninja-subtype filter for the combat damage trigger not available in DSL
// (WheneverCreatureYouControlDealsCombatDamageToPlayer filter doesn't support subtype).
// Also needs reveal-top-card + EffectAmount::LastRevealedManaValue for the life-loss.
// W5: partial trigger (wrong scope) or wrong amount would produce incorrect game state — omitted.
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
            // TODO: Ninja-subtype filter for WheneverCreatureYouControlDealsCombatDamageToPlayer
            // not wired in DSL. Also needs reveal-top-card + EffectAmount::LastRevealedManaValue
            // for the life-loss portion. W5: omitted to avoid wrong game state.
        ],
        ..Default::default()
    }
}
