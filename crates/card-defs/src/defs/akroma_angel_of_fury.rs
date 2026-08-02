// Akroma, Angel of Fury — {5}{R}{R}{R}, Legendary Creature — Angel 6/6
// This spell can't be countered.
// Flying, trample, protection from white and from blue.
// {R}: Akroma, Angel of Fury gets +1/+0 until end of turn.
// Morph {3}{R}{R}{R} (You may cast this card face down as a 2/2 creature for {3}.
// Turn it face up any time for its morph cost.)
//
// CARDS-2 (scutemob-181) second fix cycle: `cant_be_countered` was left false behind a
// stale "DSL gap" comment even though the field exists (see tyrranax_rex.rs), the {R}
// self-pump was omitted behind the same stale-gap claim even though the primitive exists
// (see scourge_of_valkas.rs for the identical shape), and the Morph cost was missing its
// {3} generic component. All three repaired; no remaining gap.
// Protection from white and blue expressed as two Protection entries.
// AbilityDefinition::Morph carries the turn-face-up cost {3}{R}{R}{R}.
// KeywordAbility::Morph is the marker for quick presence-checking.
use crate::cards::helpers::*;
use crate::state::types::ProtectionQuality;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("akroma-angel-of-fury"),
        name: "Akroma, Angel of Fury".to_string(),
        mana_cost: Some(ManaCost {
            generic: 5,
            red: 3,
            ..Default::default()
        }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Angel"]),
        oracle_text: "This spell can't be countered.\nFlying, trample, protection from white and \
                      from blue\n{R}: Akroma, Angel of Fury gets +1/+0 until end of turn.\nMorph \
                      {3}{R}{R}{R} (You may cast this card face down as a 2/2 creature for {3}. \
                      Turn it face up any time for its morph cost.)"
            .to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(
                ProtectionQuality::FromColor(Color::White),
            )),
            AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(
                ProtectionQuality::FromColor(Color::Blue),
            )),
            // CR 613.4c: "{R}: Akroma, Angel of Fury gets +1/+0 until end of turn."
            // EffectFilter::Source resolves to SingleObject(ctx.source) at execution time.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    red: 1,
                    ..Default::default()
                }),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyPower(1),
                        filter: EffectFilter::Source,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Morph),
            AbilityDefinition::Morph {
                cost: ManaCost {
                    generic: 3,
                    red: 3,
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
        cant_be_countered: true,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    }
}
