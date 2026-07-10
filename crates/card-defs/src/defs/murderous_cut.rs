// Murderous Cut — {4}{B}, Instant
// Delve (Each card you exile from your graveyard while casting this spell pays for {1}.)
// Destroy target creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("murderous-cut"),
        name: "Murderous Cut".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Delve (Each card you exile from your graveyard while casting this spell pays for {1}.)\nDestroy target creature.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Delve),
            AbilityDefinition::Spell {
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
