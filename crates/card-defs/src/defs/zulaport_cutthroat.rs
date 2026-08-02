// Zulaport Cutthroat — {1}{B}, Creature — Human Rogue Ally 1/1.
// "Whenever this creature or another creature you control dies, each opponent
// loses 1 life and you gain 1 life."
// CR 603.2/603.10a: WheneverCreatureDies trigger; controller_you because oracle says
// "this creature or another creature you control." Each opponent loses exactly 1 life
// (ForEach EachOpponent), then the controller gains exactly 1 life (flat — not
// total_lost). This is distinct from DrainLife, which gains total_lost across all
// opponents and is wrong in 3+ player games (see sanctum_seeker.rs for the same shape).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("zulaport-cutthroat"),
        name: "Zulaport Cutthroat".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            black: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Rogue", "Ally"]),
        oracle_text: "Whenever this creature or another creature you control dies, each opponent \
                      loses 1 life and you gain 1 life."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 603.10a: controller_you because oracle says "this creature or another creature
            // you control dies." Self is covered since Zulaport is your creature.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: false,
                    nontoken_only: false,
                    filter: None,
                },
                effect: Effect::Sequence(vec![
                    Effect::ForEach {
                        over: ForEachTarget::EachOpponent,
                        effect: Box::new(Effect::LoseLife {
                            player: PlayerTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(1),
                        }),
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                ]),
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
        completeness: Completeness::Complete,
    }
}
