// 75. Frogmite — {4}, Artifact Creature — Frog 2/2; Affinity for artifacts.
// (This spell costs {1} less to cast for each artifact you control.)
// With 4 artifacts controlled, Frogmite costs {0} to cast.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("frogmite"),
        name: "Frogmite".to_string(),
        mana_cost: Some(ManaCost { generic: 4, ..Default::default() }),
        types: {
            let mut tl = creature_types(&["Frog"]);
            tl.card_types.insert(CardType::Artifact);
            tl
        },
        oracle_text: "Affinity for artifacts (This spell costs {1} less to cast for each artifact you control.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 702.41a: Affinity for artifacts — costs {1} less for each artifact controlled.
            AbilityDefinition::Keyword(KeywordAbility::Affinity(AffinityTarget::Artifacts)),
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
    }
}
