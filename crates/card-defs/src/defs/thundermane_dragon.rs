// Thundermane Dragon — {3}{R}, Creature — Dragon 4/4
// Flying
// You may look at the top card of your library any time.
// You may cast creature spells with power 4 or greater from the top of your library.
// If you cast a creature spell this way, it gains haste until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thundermane-dragon"),
        name: "Thundermane Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nYou may look at the top card of your library any time.\nYou may cast creature spells with power 4 or greater from the top of your library. If you cast a creature spell this way, it gains haste until end of turn.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 601.3 (PB-A): "You may look at the top card of your library any time.
            // You may cast creature spells with power 4 or greater from the top of your library.
            // If you cast a creature spell this way, it gains haste until end of turn."
            // on_cast_effect: grant Haste (AddKeyword layer 6) until end of turn to the cast spell.
            AbilityDefinition::StaticPlayFromTop {
                filter: PlayFromTopFilter::CreaturesWithMinPower(4),
                look_at_top: true,
                reveal_top: false,
                pay_life_instead: false,
                condition: None,
                on_cast_effect: Some(Box::new(Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                        filter: EffectFilter::Source,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                })),
            },
        ],
        ..Default::default()
    }
}
