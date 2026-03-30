// Hanweir Battlements — Land
// {T}: Add {C}.
// {R}, {T}: Target creature gains haste until end of turn.
// {3}{R}{R}, {T}: If you both own and control this land and a creature named
// Hanweir Garrison, exile them, then meld them into Hanweir, the Writhing Township.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hanweir-battlements"),
        name: "Hanweir Battlements".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{R}, {T}: Target creature gains haste until end of turn.\n{3}{R}{R}, {T}: If you both own and control this land and a creature named Hanweir Garrison, exile them, then meld them into Hanweir, the Writhing Township.".to_string(),
        abilities: vec![
            // {T}: Add {C}
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // {R}, {T}: Target creature gains haste until end of turn.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { red: 1, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: crate::state::EffectLayer::Ability,
                        modification: crate::state::LayerModification::AddKeyword(
                            KeywordAbility::Haste,
                        ),
                        filter: crate::state::EffectFilter::DeclaredTarget { index: 0 },
                        duration: crate::state::EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreature],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // {3}{R}{R}, {T}: Meld with Hanweir Garrison (CR 701.42a / CR 712.4a)
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 3, red: 2, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::Meld,
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        meld_pair: Some(MeldPair {
            pair_card_id: CardId("hanweir-garrison".to_string()),
            melded_card_id: CardId("hanweir-the-writhing-township".to_string()),
        }),
        ..Default::default()
    }
}
