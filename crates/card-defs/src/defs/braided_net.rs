// Braided Net // Braided Quipu — {2}{U} Artifact, DFC with Craft (CR 702.167)
//
// Front — Braided Net, {2}{U} Artifact:
//   "This artifact enters with three net counters on it.
//    {T}, Remove a net counter from this artifact: Tap another target nonland permanent.
//    Its activated abilities can't be activated for as long as it remains tapped.
//    Craft with artifact {1}{U}"
// Back — Braided Quipu, Artifact:
//   "{3}{U}, {T}: Draw a card for each artifact you control, then put this artifact into
//    its owner's library third from the top."
//
// CARDS-2 (scutemob-181), SECOND fix cycle. This file is the batch's own worst moment and
// is documented as such rather than quietly corrected.
//
// The first fix cycle *authored three abilities this card does not have* into a `Complete`
// def: an ETB "tap target creature an opponent controls" on each face, and a back-face
// "whenever you cast a spell, draw a card" — a free draw engine. None of it is printed.
// It happened because the repair was briefed from THIS FILE'S OWN STALE COMMENT rather
// than from the oracle: the header had described that imaginary card since the def was
// written, the brief copied it, and the agent flagged that it could not confirm the text
// through MCP (the DFC summary view returns no oracle text for either face) and proceeded
// on the brief's authority. Every link in that chain trusted the previous one and none
// touched the card.
//
// So the def went from "missing abilities" — a silent no-op — to **legal but wrong**,
// which `memory/project_legal_but_wrong_gap.md` names as the project's biggest pre-alpha
// risk. It was written in the same commit that recorded the rule it broke.
//
// TWO durable lessons, both paid for:
//   1. A def's own comment is not evidence about the card. It is the thing under audit.
//      Verify against Scryfall/MCP, and when the DFC summary view is empty, query
//      `card_faces` directly — `cards.sqlite` has per-face `oracle_text` and the fixture
//      generator already joins it.
//   2. "Not expressible" claims must name the primitive so they can be rechecked
//      (OOS-CARDS2-8). The stale claim here was not merely out of date, it was about a
//      card that did not exist.
//
// DEMOTED, not repaired. The printed front face needs: enters-with-N-counters on a
// non-creature artifact, a `Remove a counter` activation cost, "tap ANOTHER target NONLAND
// permanent" (`Effect::TapPermanent` exists, but the target filter needs nonland +
// exclude_self), and a static "its activated abilities can't be activated while it remains
// tapped" — a conditional, duration-bound activation lock keyed to another object's tapped
// state, which the DSL has no expression for. The back face needs
// "draw a card for each artifact you control" (a scaled draw) and "put this into its
// owner's library third from the top" (a positional library insert; `ZoneTarget::Library`
// offers top/bottom only). Craft is real and stays; its cost is corrected {2}{U} -> {1}{U}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("braided-net-braided-quipu"),
        name: "Braided Net".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "This artifact enters with three net counters on it.\n{T}, Remove a net \
                      counter from this artifact: Tap another target nonland permanent. Its \
                      activated abilities can't be activated for as long as it remains \
                      tapped.\nCraft with artifact {1}{U}"
            .to_string(),
        power: None,
        toughness: None,
        abilities: vec![
            // CR 702.167: Craft with artifact {1}{U}. Real, and the only printed ability
            // on this face the DSL can express.
            AbilityDefinition::Keyword(KeywordAbility::Craft),
            AbilityDefinition::Craft {
                cost: ManaCost {
                    generic: 1,
                    blue: 1,
                    ..Default::default()
                },
                materials: CraftMaterials::Artifacts(1),
            },
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Braided Quipu".to_string(),
            mana_cost: None,
            types: types(&[CardType::Artifact]),
            oracle_text: "{3}{U}, {T}: Draw a card for each artifact you control, then put this \
                          artifact into its owner's library third from the top."
                .to_string(),
            power: None,
            toughness: None,
            abilities: vec![],
            color_indicator: Some(vec![Color::Blue]),
        }),
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
        completeness: Completeness::known_wrong(
            "def was authored against text this card does not have, and a CARDS-2 repair pass \
             then IMPLEMENTED that invented text (ETB 'tap target creature an opponent controls' \
             on both faces, plus a back-face 'whenever you cast a spell, draw a card') because it \
             was briefed from this file's own stale header comment instead of the oracle. The \
             invented abilities are removed. Real front text: three net counters on ETB; '{T}, \
             Remove a net counter: Tap another target nonland permanent. Its activated abilities \
             can't be activated for as long as it remains tapped'; Craft with artifact {1}{U} \
             (the def had {2}{U}, now corrected). Real back text: '{3}{U}, {T}: Draw a card for \
             each artifact you control, then put this artifact into its owner's library third \
             from the top.' MISSING PRIMITIVES, named so they can be rechecked (OOS-CARDS2-8): \
             (a) enters-with-N-counters on a noncreature artifact; (b) a remove-a-counter \
             activation cost; (c) a target filter for 'another nonland permanent' (nonland + \
             exclude_self); (d) a static activation-lock on another object conditioned on its \
             tapped state, for any duration; (e) a draw scaled by a permanent count; (f) a \
             positional library insert -- ZoneTarget::Library is top/bottom only. Craft is \
             implemented and correct.",
        ),
    }
}
