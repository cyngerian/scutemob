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
            // TODO: DSL gap — {X} activated ability where X determines the base P/T buff
            // for all creatures you control is not expressible. EffectAmount::XValue requires
            // X to be set on CastSpell; for activated abilities X cost parsing is not supported.
            // Additionally, "gain all creature types" as an effect is not in the DSL.
        ],
        ..Default::default()
    }
}
