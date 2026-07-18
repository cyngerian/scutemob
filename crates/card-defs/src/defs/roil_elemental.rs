// Roil Elemental — {3}{U}{U}{U}, Creature — Elemental 3/2
// Flying
// Landfall — Whenever a land you control enters, you may gain control of target creature
// for as long as you control this creature.
//
// Flying is implemented.
//
// TODO: Blocker — the "you may" optional wrapper around a costless GainControl effect is
// not expressible. PB-EF9 added EffectDuration::WhileYouControlSource, which resolves the
// DURATION half of this ability (the Landfall trigger itself is also coverable via
// TriggerCondition::WheneverPermanentEntersBattlefield { Land + You } + Effect::GainControl,
// CR 207.2c). But the ability is optional ("you MAY gain control..."), and there is no
// costless "you may [effect]" primitive: Effect::MayPayThenEffect requires a real Cost to
// pay (it is not a bare optional-effect wrapper), and Effect::MayPayOrElse is a gated stub
// (SR-33, never offers the choice). Authoring this as a mandatory GainControl would be
// legal-but-wrong (a mandatory steal where the printed card lets the controller decline).
// The Landfall ability remains omitted until a costless optional-effect primitive exists.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("roil-elemental"),
        name: "Roil Elemental".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            blue: 3,
            ..Default::default()
        }),
        types: creature_types(&["Elemental"]),
        oracle_text: "Flying\nLandfall \u{2014} Whenever a land you control enters, you may gain \
                      control of target creature for as long as you control this creature."
            .to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Flying)],
        completeness: Completeness::partial(
            "Landfall ('Whenever a land you control enters, you may gain control of target \
             creature for as long as you control this creature.') is NOT implemented. PB-EF9 \
             shipped EffectDuration::WhileYouControlSource, resolving the DURATION half of this \
             ability. The SURVIVING blocker: the ability is optional ('you MAY'), and there is no \
             costless 'you may [effect]' primitive — Effect::MayPayThenEffect requires a real \
             Cost, and Effect::MayPayOrElse is a gated stub (SR-33) that never offers the choice. \
             Authoring a mandatory GainControl here would be legal-but-wrong (W5 policy: wrong \
             game state, not a missing clause).",
        ),
        ..Default::default()
    }
}
