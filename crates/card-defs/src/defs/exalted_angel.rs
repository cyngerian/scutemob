// Exalted Angel — {4}{W}{W}, Creature — Angel 4/5
// Flying
// Whenever this creature deals damage, you gain that much life.
// Morph {2}{W}{W} (You may cast this card face down as a 2/2 creature for {3}.
// Turn it face up any time for its morph cost.)
//
// AbilityDefinition::Morph carries the turn-face-up cost {2}{W}{W}.
// KeywordAbility::Morph is the marker for quick presence-checking.
//
// CR 702.15a lifelink is a static keyword; this is NOT lifelink — the printed
// ability is a *triggered* ability, uses the stack, and can be responded to or
// countered (e.g. by Stifle). PB-DX36 (`OOS-CARDS2-6`) authored the missing
// primitives: `TriggerCondition::WhenDealsDamage { recipient: DamageRecipient::Any }`
// (CR 603.2, any damage — combat or noncombat — to any recipient) lowers to
// `TriggerEvent::SelfDealsDamage`, dispatched from both `GameEvent::CombatDamageDealt`
// and `GameEvent::DamageDealt` via `rules/abilities.rs::queue_damage_source_triggers`;
// `EffectAmount::DamageDealt` reads the CR 608.2h / CR 113.7a "that much" amount from
// `EffectContext::damage_dealt_amount`.
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
            // CR 603.2: "Whenever this creature deals damage, you gain that much life."
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenDealsDamage {
                    recipient: DamageRecipient::Any,
                },
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::DamageDealt,
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
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
        // Declared EXPLICITLY rather than left to the `#[default]` derive: `OOS-RR3-1`
        // measured 965 defs that never declare a marker and observed that nothing
        // reviews that population. A def this batch promotes says so out loud.
        completeness: Completeness::Complete,
    }
}
