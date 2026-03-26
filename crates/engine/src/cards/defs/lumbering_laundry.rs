// Lumbering Laundry — {5}, Artifact Creature — Golem 4/5
// Disguise {5} (You may cast this card face down for {3} as a 2/2 creature with ward {2}.
// Turn it face up any time for its disguise cost.)
//
// The activated ability "{2}: Until end of turn, you may look at face-down creatures you
// don't control any time." has no DSL primitive — omitted with TODO below.
// AbilityDefinition::Disguise carries the turn-face-up cost {5}.
// KeywordAbility::Disguise is the marker for quick presence-checking.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lumbering-laundry"),
        name: "Lumbering Laundry".to_string(),
        mana_cost: Some(ManaCost { generic: 5, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Golem"]),
        oracle_text:
            "{2}: Until end of turn, you may look at face-down creatures you don't control any time.\n\
             Disguise {5} (You may cast this card face down for {3} as a 2/2 creature with ward {2}. \
             Turn it face up any time for its disguise cost.)"
                .to_string(),
        power: Some(4),
        toughness: Some(5),
        abilities: vec![
            // TODO: "{2}: look at face-down creatures you don't control" — no hidden-info
            // peek effect primitive in DSL. Omitted.
            AbilityDefinition::Keyword(KeywordAbility::Disguise),
            AbilityDefinition::Disguise { cost: ManaCost { generic: 5, ..Default::default() } },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
    }
}
