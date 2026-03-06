// Grim Harvest — {1}{B}, Instant; return target creature card from graveyard to hand; Recover {2}{B}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("grim-harvest"),
        name: "Grim Harvest".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Return target creature card from your graveyard to your hand.\nRecover {2}{B} (When a creature is put into your graveyard from the battlefield, you may pay {2}{B}. If you do, return this card from your graveyard to your hand. Otherwise, exile this card.)".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                // TODO: Effect::ReturnFromGraveyard does not exist in the DSL.
                // The main effect should return target creature card from graveyard to hand.
                // Using Effect::DrawCards as a placeholder until ReturnFromGraveyard is implemented.
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(0),
                },
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                cant_be_countered: false,
            },
            AbilityDefinition::Keyword(KeywordAbility::Recover),
            AbilityDefinition::Recover {
                cost: ManaCost { generic: 2, black: 1, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}
