// Ainok Bond-Kin — {1}{W}, Creature — Dog Soldier 2/1; Outlast {1}{W};
// other creatures you control with +1/+1 counters have first strike (TODO: static grant).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ainok-bond-kin"),
        name: "Ainok Bond-Kin".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: creature_types(&["Dog", "Soldier"]),
        oracle_text: "Outlast {1}{W} ({1}{W}, {T}: Put a +1/+1 counter on this creature. Outlast only as a sorcery.)\nEach creature you control with a +1/+1 counter on it has first strike.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Outlast),
            AbilityDefinition::Outlast {
                cost: ManaCost { generic: 1, white: 1, ..Default::default() },
            },
            // TODO: Layer 6 static grant — "Each creature you control with a +1/+1 counter
            // on it has first strike." Requires a ContinuousEffectDef with a
            // Modification::GainKeyword(FirstStrike) filtered to creatures with +1/+1 counters.
            // Omitted until the DSL supports a counter-presence filter on static grants.
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
    }
}
