// Shambling Ghast — {B}, Creature — Zombie 1/1; Decayed; ETB "choose one" omitted (modal choice DSL gap).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shambling-ghast"),
        name: "Shambling Ghast".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Zombie"]),
        oracle_text:
            "Decayed (This creature can't block. When it attacks, sacrifice it at end of combat.)\nWhen Shambling Ghast enters, create a Treasure token or put a -1/-1 counter on target creature. Choose one."
                .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Decayed),
            // TODO: ETB "choose one" trigger (create Treasure token OR put -1/-1 counter on
            // target creature) requires modal triggered ability support. The DSL does not yet
            // have a ChooseOne/Modal wrapper for triggered abilities (modal choice DSL gap).
            // CR 700.2 governs modal spells and abilities. Add when the engine supports
            // mode selection on triggered abilities.
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
    }
}
