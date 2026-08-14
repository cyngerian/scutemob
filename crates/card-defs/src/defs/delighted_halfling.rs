// Delighted Halfling — {G}, Creature — Halfling Citizen 1/2.
// "{T}: Add {G}. If this mana is spent to cast a legendary spell, that spell
// can't be countered."
// TODO: DSL gap — "mana tracking" (conditional uncounterability based on mana
// source) is not expressible. Modeled as a plain {G} mana dork.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("delighted-halfling"),
        name: "Delighted Halfling".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Halfling", "Citizen"]),
        // PB-DX27 (2026-08-13), OOS-CARDS2-10: the previous text INVENTED "{T}: Add {G}".
        // The card prints TWO separate mana abilities — an unrestricted "{T}: Add {C}" and a
        // restricted any-colour one. This matters beyond text: colour identity is computed
        // from mana production, so a green source that should be colourless changes what
        // `random_deck` builds (OOS-CARDS2-3). The def is `partial`, and `random_deck`
        // filters on `is_complete()`, so no deal moves today — but the text was the trap.
        // Replaced with the MCP-verified printed text.
        oracle_text: "{T}: Add {C}.\n{T}: Add one mana of any color. Spend this mana only to cast \
                      a legendary spell, and that spell can't be countered."
            .to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddMana {
                player: PlayerTarget::Controller,
                mana: mana_pool(0, 0, 0, 0, 1, 0),
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
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
            "DSL gap — 'mana tracking' (conditional uncounterability based on mana source) is not \
             expressible. Modeled as a plain...",
        ),
    }
}
