// Scavenger Grounds — Land — Desert
// {T}: Add {C}. {2},{T}, Sacrifice a Desert: Exile all graveyards (not expressible).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scavenger-grounds"),
        name: "Scavenger Grounds".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Desert"]),
        oracle_text: "{T}: Add {C}.\n{2}, {T}, Sacrifice a Desert: Exile all graveyards.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // CR 406.2: {2}, {T}, Sacrifice a Desert: Exile all cards from all graveyards.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                    Cost::Tap,
                    Cost::Sacrifice(TargetFilter {
                        has_subtype: Some(SubType("Desert".to_string())),
                        ..Default::default()
                    }),
                ]),
                effect: Effect::ForEach {
                    over: ForEachTarget::EachCardInAllGraveyards,
                    effect: Box::new(Effect::ExileObject {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    }),
                },
                timing_restriction: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
