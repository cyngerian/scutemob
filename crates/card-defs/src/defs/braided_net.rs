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
// DEMOTED, not repaired — and the reason has itself been rechecked, because the first
// version of this note failed the very test lesson 2 above sets.
//
// The first draft listed SIX missing primitives. A reviewer rechecked all six, as that
// lesson invites, and **four of them exist**:
//
//   claim                                        verdict
//   enters-with-N-counters on a noncreature      FALSE. `ReplacementModification::
//                                                EntersWithCounters` is type-agnostic, and
//                                                even the "Net" counter is expressible as
//                                                `CounterType::Custom(..)`.
//   a remove-a-counter activation cost           FALSE. `Cost::RemoveCounter { .. }`, used
//                                                by 13 defs.
//   "another nonland permanent" target filter    FALSE. `TargetFilter.non_land` and
//                                                `.exclude_self` both exist and are enforced.
//   a draw scaled by a permanent count           FALSE. `Effect::DrawCards { count }` with
//                                                `EffectAmount::PermanentCount { .. }`.
//   an activation lock on another object,        TRUE. `LayerModification` can
//   conditioned on its tapped state              `RemoveAllAbilities` but cannot forbid
//                                                activation.
//   a positional library insert                  TRUE. `LibraryPosition` is Top / Bottom /
//                                                ShuffledIn — no Nth-from-top.
//
// So this card is *mostly* expressible: the front face lacks only "its activated abilities
// can't be activated for as long as it remains tapped", and the back face only "third from
// the top". It stays `known_wrong` and unauthored anyway, deliberately, on two grounds:
// the activation lock is the load-bearing half of the front face's ability (a Braided Net
// that taps without locking is a materially different card), and **this file has already
// demonstrated once what authoring into it on incomplete information costs**. Authoring it
// belongs to a pass that can verify the result against the card, clause by clause — which is
// what OOS-CARDS2-8 asks for and what the earlier pass did not do.
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
        completeness: Completeness::partial(
            "PARTIAL, not KnownWrong: only Craft is implemented -- the net counters, the whole \
             tap ability and the entire back face are absent, which is the Partial definition \
             (card_definition.rs). Originally authored against text this card does not have; a \
             CARDS-2 pass then IMPLEMENTED that invented text (see the header comment for the \
             full incident) because it was briefed from this file's own stale comment instead of \
             the oracle. Invented abilities removed; Craft cost corrected {2}{U} -> {1}{U}. TWO \
             missing primitives, RE-VERIFIED after a reviewer found four of the six first listed \
             here to exist: (1) a static activation-lock on another object keyed to its tapped \
             state (LayerModification can RemoveAllAbilities but cannot forbid activation); (2) a \
             positional library insert (LibraryPosition is Top/Bottom/ShuffledIn only). \
             Everything else this card needs exists, so it is mostly authorable and is left \
             unauthored on purpose -- the lock is the load-bearing half of the front ability. \
             Craft is implemented and correct.",
        ),
    }
}
