// Tyrranax Rex — {4}{G}{G}{G}, Creature — Phyrexian Dinosaur 8/8
// "This spell can't be countered.
//  Trample, ward {4}, haste
//  Toxic 4 (Players dealt combat damage by this creature also get four poison counters.)"
//
// CARDS-2 (scutemob-181), review Issue 1. This def was the motivating example for the
// SR-37 fidelity gate — it shipped `Complete` at `{G}{G}{G}{G}`, three mana cheap on a
// seven-drop — and repairing the cost was not enough. The gate checks four *printed
// fields*, and Tyrranax Rex's remaining defects were all in the *abilities*, where it is
// blind:
//
//   * `KeywordAbility::Ravenous` was declared, and Ravenous (CR 702.156) is on **no
//     printing of this card**. It was invented, along with an `oracle_text` describing it.
//   * haste, Toxic 4 and "this spell can't be countered" were all missing.
//
// The batch's own heuristic — "more than one wrong printed field means the def was
// authored from a misremembered card" — could not have caught this, because only ONE
// printed field was wrong. The rule the heuristic should have been: **a wrong printed
// field is reason to re-read the whole oracle, not to fix the field.** Every primitive
// needed here already existed (`Haste`, `Toxic(u32)`, the `cant_be_countered` flag), so
// this is a full repair and the def stays `Complete`.
//
// The invented Ravenous had a golden script certifying it
// (`test-data/generated-scripts/etb-triggers/177_ravenous_tyrranax_rex_draw.json`), which
// is retired alongside this repair. Ravenous itself keeps its six engine tests in
// `crates/engine/tests/mechanics_m_z/ravenous.rs`, against a mock — the right home for a
// keyword test, for exactly this reason.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tyrranax-rex"),
        name: "Tyrranax Rex".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            green: 3,
            ..Default::default()
        }),
        types: creature_types(&["Phyrexian", "Dinosaur"]),
        oracle_text: "This spell can't be countered.\nTrample, ward {4}, haste\nToxic 4 (Players \
                      dealt combat damage by this creature also get four poison counters.)"
            .to_string(),
        power: Some(8),
        toughness: Some(8),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // CR 702.21a: Ward {4} — triggers whenever this permanent becomes the
            // target of a spell or ability an opponent controls; counter it unless
            // that player pays {4}.
            AbilityDefinition::Keyword(KeywordAbility::Ward(4)),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // CR 702.164c (702.164a only fixes the notation "toxic N"): a player dealt
            // combat damage by this creature also gets four poison counters.
            // CR 120.3g: combat damage to a PLAYER only.
            AbilityDefinition::Keyword(KeywordAbility::Toxic(4)),
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
        // CR 701.6a is Counter (701.5a is Cast): "this spell can't be countered" is a static
        // ability of the spell itself, so it lives on the definition, not in `abilities`.
        cant_be_countered: true,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    }
}
