// Bolas's Citadel — {3}{B}{B}{B}, Legendary Artifact
// You may look at the top card of your library any time.
// You may play lands and cast spells from the top of your library. If you cast a spell
// this way, pay life equal to its mana value rather than pay its mana cost.
// {T}, Sacrifice ten nonland permanents: Each opponent loses 10 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bolass-citadel"),
        name: "Bolas's Citadel".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 3, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Artifact], &[]),
        oracle_text: "You may look at the top card of your library any time.\nYou may play lands and cast spells from the top of your library. If you cast a spell this way, pay life equal to its mana value rather than pay its mana cost.\n{T}, Sacrifice ten nonland permanents: Each opponent loses 10 life.".to_string(),
        abilities: vec![
            // CR 601.3 / CR 305.1 (PB-A): "You may look at the top card of your library any time.
            // You may play lands and cast spells from the top of your library."
            // pay_life_instead: true — when casting a spell, pay life = mana value (AltCostKind::PayLifeForManaValue).
            // look_at_top: true — controller may look at top card any time (not revealed to all).
            // 2019-05-03 ruling: X must be 0 when casting this way; additional costs still apply.
            AbilityDefinition::StaticPlayFromTop {
                filter: PlayFromTopFilter::All,
                look_at_top: true,
                reveal_top: false,
                pay_life_instead: true,
                condition: None,
                on_cast_effect: None,
            },
            // TODO: "{T}, Sacrifice ten nonland permanents: Each opponent loses 10 life."
            // Requires a Cost variant for "sacrifice N permanents with filter" — DSL gap.
            // The sacrifice-10-nonland-permanents cost cannot be expressed yet. Deferred.
        ],
        ..Default::default()
    }
}
