// Cabal Coffers — Land
// {2}, {T}: Add {B} for each Swamp you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cabal-coffers"),
        name: "Cabal Coffers".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{2}, {T}: Add {B} for each Swamp you control.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::AddManaScaled {
                    player: PlayerTarget::Controller,
                    color: ManaColor::Black,
                    count: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Land),
                            has_subtype: Some(SubType("Swamp".to_string())),
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    },
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
