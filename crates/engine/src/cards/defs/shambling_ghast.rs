// Shambling Ghast — {B}, Creature — Zombie 1/1; Decayed.
// When Shambling Ghast dies, create a Treasure token or put a -1/-1 counter
// on target creature an opponent controls. Choose one.
//
// CR 700.2b / PB-35: Modal WhenDies triggered ability. Bot fallback: mode 0 (Treasure token).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shambling-ghast"),
        name: "Shambling Ghast".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Zombie"]),
        oracle_text:
            "Decayed (This creature can't block. When it attacks, sacrifice it at end of combat.)\nWhen Shambling Ghast enters, create a Treasure token or put a -1/-1 counter on target creature."
                .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Decayed),
            // CR 700.2b / PB-35: Modal WhenDies trigger.
            // Mode 0: Create a Treasure token.
            // Mode 1: Put a -1/-1 counter on target creature an opponent controls.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::Nothing,
                intervening_if: None,
                targets: vec![
                    // Mode 1 target: opponent's creature.
                    TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                        controller: TargetController::Opponent,
                        ..Default::default()
                    }),
                ],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    modes: vec![
                        // Mode 0: Create a Treasure token.
                        Effect::CreateToken {
                            spec: treasure_token_spec(1),
                        },
                        // Mode 1: Put a -1/-1 counter on target creature.
                        Effect::AddCounter {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            counter: CounterType::MinusOneMinusOne,
                            count: 1,
                        },
                    ],
                    allow_duplicate_modes: false,
                    mode_costs: None,
                }),
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
    }
}
