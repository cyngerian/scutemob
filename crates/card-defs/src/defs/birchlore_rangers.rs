// Birchlore Rangers — {G}, Creature — Elf Druid Ranger 1/1
// Tap two untapped Elves you control: Add one mana of any color.
// Morph {G} (You may cast this card face down as a 2/2 creature for {3}.
// Turn it face up any time for its morph cost.)
//
// CARDS-2 (scutemob-181) second fix cycle: Morph cost was {0} (free to turn face up);
// printed cost is {G}, now fixed.
//
// The tap-two-Elves mana ability requires tapping N other permanents you control
// matching a filter (Cost::TapCreatures(2) or similar). No such Cost variant exists in
// the DSL (see nullmage_shepherd.rs for the identical gap on a different N). Left
// unimplemented; def demoted from Complete since a printed clause is missing.
// AbilityDefinition::Morph carries the turn-face-up cost {G}.
// KeywordAbility::Morph is the marker for quick presence-checking.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("birchlore-rangers"),
        name: "Birchlore Rangers".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Creature], &["Elf", "Druid", "Ranger"]),
        oracle_text: "Tap two untapped Elves you control: Add one mana of any color.\nMorph {G} \
                      (You may cast this card face down as a 2/2 creature for {3}. Turn it face \
                      up any time for its morph cost.)"
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // Tap-two-Elves mana ability is a DSL gap — omitted (no multi-tap-creature cost
            // primitive; see nullmage_shepherd.rs for the identical gap).
            AbilityDefinition::Keyword(KeywordAbility::Morph),
            AbilityDefinition::Morph {
                cost: ManaCost {
                    green: 1,
                    ..Default::default()
                },
            },
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
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::partial(
            "The activated mana ability's cost requires tapping two untapped Elves you control \
             (Cost::TapCreatures(2) or similar, filtered to Elf). No such Cost variant exists in \
             the DSL; see nullmage_shepherd.rs for the identical gap. Morph is implemented.",
        ),
    }
}
