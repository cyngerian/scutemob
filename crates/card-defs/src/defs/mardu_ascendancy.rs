// Mardu Ascendancy — {R}{W}{B}, Enchantment
// Whenever a nontoken creature you control attacks, create a 1/1 red Goblin creature
// token that's tapped and attacking.
// Sacrifice this enchantment: Creatures you control get +0/+3 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mardu-ascendancy"),
        name: "Mardu Ascendancy".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            white: 1,
            black: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a nontoken creature you control attacks, create a 1/1 red Goblin \
                      creature token that's tapped and attacking.\nSacrifice this enchantment: \
                      Creatures you control get +0/+3 until end of turn."
            .to_string(),
        abilities: vec![
            // CR 508.1m: "Whenever a nontoken creature you control attacks, create a 1/1 red
            // Goblin token tapped and attacking."
            // PB-23: WheneverCreatureYouControlAttacks.
            // TODO: Nontoken filter not yet in DSL for attack triggers — over-triggers on token
            // attackers (including Goblin tokens created by this ability itself).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks {
                    filter: None,
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Goblin".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: [Color::Red].into_iter().collect(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
                        count: EffectAmount::Fixed(1),
                        tapped: true,
                        enters_attacking: true,
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // Sacrifice Mardu Ascendancy: Creatures you control get +0/+3 until end of turn.
            AbilityDefinition::Activated {
                cost: Cost::SacrificeSelf,
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyToughness(3),
                        filter: EffectFilter::CreaturesYouControl,
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
        ],
        completeness: Completeness::partial(
            "SURVIVING blocker (CR 508.1m): nontoken filter not yet in DSL for attack triggers — \
             TriggerCondition::WheneverCreatureYouControlAttacks carries only `filter: \
             Option<TargetFilter>` and no nontoken predicate, so this over-triggers on token \
             attackers, including the Goblin tokens this very ability creates. CLOSED blocker, \
             recorded because this note named only the first one for four months while the second \
             was live in every game (PB-DX39, OOS-DX5-7 residual): the `Cost::SacrificeSelf` \
             ability below sacrifices its own source to pay for itself, so the source was ALWAYS \
             gone by the time the effect resolved, and `EffectFilter::CreaturesYouControl` \
             answered `false` for every creature — the +0/+3 applied to NOBODY, always. CR 611.2c \
             determines the set at resolution and CR 608.2h/113.7a say it must use the source's \
             last known information; it now does. This def stays `partial` on the nontoken filter \
             ALONE and must not be promoted until that is closed.",
        ),
        ..Default::default()
    }
}
