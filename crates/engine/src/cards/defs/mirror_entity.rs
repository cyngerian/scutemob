// Mirror Entity — {2}{W}, Creature — Shapeshifter 1/1
// Changeling (This card is every creature type.)
// {X}: Until end of turn, creatures you control have base power and toughness X/X and
// gain all creature types.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mirror-entity"),
        name: "Mirror Entity".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: creature_types(&["Shapeshifter"]),
        oracle_text: "Changeling (This card is every creature type.)\n{X}: Until end of turn, creatures you control have base power and toughness X/X and gain all creature types.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Changeling),
            // CR 107.3k: {X}: Until EOT, creatures you control have base P/T X/X and gain all creature types.
            // x_value is now passed through ActivateAbility command and into the EffectContext.
            // TODO: Dynamic P/T setting (base X/X) requires LayerModification::SetBothDynamic(EffectAmount)
            // which does not exist. LayerModification::SetBoth(power, toughness) takes fixed i32.
            // Deferred until dynamic P/T layer modification is added.
            // TODO: "gain all creature types" has no DSL representation (AddAllCreatureTypes variant missing).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { x_count: 1, ..Default::default() }),
                effect: Effect::Nothing,
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
