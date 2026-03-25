// Tyrranax Rex — {X}{G}{G}{G}{G}, Creature — Phyrexian Dinosaur 8/8
// Trample, ward {4}, Ravenous (enters with X +1/+1 counters; draw a card if X >= 5)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tyrranax-rex"),
        name: "Tyrranax Rex".to_string(),
        mana_cost: Some(ManaCost { green: 4, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Dinosaur"]),
        oracle_text: "Trample, ward {4}\nRavenous (This creature enters with X +1/+1 counters on it. If X is 5 or more, draw a card when it enters.)".to_string(),
        power: Some(8),
        toughness: Some(8),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // CR 702.21a: Ward {4} — triggers whenever this permanent becomes the
            // target of a spell or ability an opponent controls; counter it unless
            // that player pays {4}.
            AbilityDefinition::Keyword(KeywordAbility::Ward(4)),
            // CR 702.156: Ravenous — ETB replacement adds X +1/+1 counters.
            // CR 702.156a: If X >= 5, a triggered ability puts a draw on the stack.
            AbilityDefinition::Keyword(KeywordAbility::Ravenous),
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        activated_ability_cost_reductions: vec![],
    }
}
