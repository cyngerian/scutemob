// Backup Agent — {1}{W}, Creature — Human Citizen 1/1.
// When this creature enters, put a +1/+1 counter on target creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("backup-agent"),
        name: "Backup Agent".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Citizen"]),
        oracle_text: "When this creature enters, put a +1/+1 counter on target creature."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // ETB: put a +1/+1 counter on target creature (any controller).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::AddCounter {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreature],
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
        completeness: Completeness::Complete,
    }
}
