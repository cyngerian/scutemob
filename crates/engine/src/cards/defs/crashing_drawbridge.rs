// Crashing Drawbridge — {2}, Artifact Creature — Wall 0/4
// Defender
// {T}: Creatures you control gain haste until end of turn.
// TODO: DSL gap — {T}: grant haste to all creatures you control until end of turn requires
// ApplyContinuousEffect with EffectFilter::CreaturesYouControl granting Haste; not supported.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crashing-drawbridge"),
        name: "Crashing Drawbridge".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Wall"]),
        oracle_text: "Defender\n{T}: Creatures you control gain haste until end of turn.".to_string(),
        power: Some(0),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Defender),
            // TODO: {T}: grant haste to all creatures you control until end of turn
        ],
        ..Default::default()
    }
}
