// Poisonous Viper — test card (no real-world equivalent), {2}{B}, Creature — Snake 2/2;
// Poisonous 1. Created for script validation of CR 702.70 (Poisonous keyword).
// The unit tests in tests/poisonous.rs use the same name via ObjectSpec::creature.
// CR 702.70a: "Whenever this creature deals combat damage to a player, that player
// gets N poison counters." Poisonous is additive — it does NOT replace normal damage.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("poisonous-viper"),
        name: "Poisonous Viper".to_string(),
        mana_cost: Some(ManaCost { black: 1, generic: 2, ..Default::default() }),
        types: creature_types(&["Snake"]),
        oracle_text: "Poisonous 1 (Whenever this creature deals combat damage to a player, that player gets 1 poison counter.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Poisonous(1)),
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    }
}
