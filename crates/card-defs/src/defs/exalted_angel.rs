// Exalted Angel — {4}{W}{W}, Creature — Angel 4/5
// Flying
// Whenever this creature deals damage, you gain that much life.
// Morph {2}{W}{W} (You may cast this card face down as a 2/2 creature for {3}.
// Turn it face up any time for its morph cost.)
//
// AbilityDefinition::Morph carries the turn-face-up cost {2}{W}{W}.
// KeywordAbility::Morph is the marker for quick presence-checking.
//
// TODO (CARDS-2, scutemob-181): DSL gap. The printed ability is a *triggered*
// ability (CR 702.15a lifelink is a static keyword; this is not lifelink — it uses
// the stack, can be responded to, and can be countered, e.g. by Stifle). No
// `TriggerCondition` exists for "whenever this permanent deals damage" in general
// (combat or noncombat, to any recipient). The closest variants are
// `TriggerCondition::WhenDealsCombatDamageToPlayer` (too narrow — misses combat
// damage to blocking creatures/planeswalkers and any noncombat damage) and
// `TriggerCondition::WhenDealtDamage` (wrong direction — that is CR 702.111a
// Enrage, "whenever THIS creature IS dealt damage", not "deals"). Needs a new
// general `TriggerCondition::WhenDealsDamage` plus a damage-dealt `EffectAmount`
// (an analogue of the existing `EffectAmount::CombatDamageDealt`, generalized to
// noncombat damage) for "gain that much life". Per W5 policy, the incorrect static
// Lifelink keyword is removed rather than left in place.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("exalted-angel"),
        name: "Exalted Angel".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            white: 2,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Creature], &["Angel"]),
        oracle_text: "Flying\nWhenever this creature deals damage, you gain that much \
                      life.\nMorph {2}{W}{W} (You may cast this card face down as a 2/2 creature \
                      for {3}. Turn it face up any time for its morph cost.)"
            .to_string(),
        power: Some(4),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Morph),
            AbilityDefinition::Morph {
                cost: ManaCost {
                    generic: 2,
                    white: 2,
                    ..Default::default()
                },
            },
            // TODO: "Whenever this creature deals damage, you gain that much life" — no
            // general damage-dealt trigger exists. See header TODO for the missing primitives.
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
            "def was authored against text this card does not have. Real oracle has NO lifelink \
             keyword — the printed ability is a triggered ability: 'Whenever this creature deals \
             damage, you gain that much life.' The def previously declared static \
             KeywordAbility::Lifelink, which is not functionally equivalent (CR 702.15a lifelink \
             cannot be responded to or countered; this trigger can, e.g. by Stifle). DSL gap: no \
             general 'whenever this deals damage' TriggerCondition (WhenDealsCombatDamageToPlayer \
             is too narrow; WhenDealtDamage is the Enrage direction, not this one) and no damage- \
             dealt EffectAmount generalized to noncombat damage. Flying and Morph are correct and \
             unaffected.",
        ),
    }
}
