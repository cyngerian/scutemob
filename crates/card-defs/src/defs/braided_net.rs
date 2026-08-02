// Braided Net // Braided Quipu — DFC with Craft (CR 702.167)
// Front: {2}{U} Artifact, when ETB tap target creature an opponent controls.
//        Craft with artifact {2}{U}
// Back:  Braided Quipu, Artifact, when ETB tap target creature an opponent controls,
//        whenever you cast a spell, draw a card.
//
// Both ETB tap triggers use Effect::TapPermanent (mirrors ravenous_chupacabra.rs /
// sharktocrab.rs). The back face's cast-spell trigger uses
// TriggerCondition::WheneverYouCastSpell with during_opponent_turn: false and
// spell_type_filter: None (mirrors murmuring_mystic.rs) — fires on any spell you cast.
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
        oracle_text: "When Braided Net enters the battlefield, tap target creature an opponent \
                      controls.\nCraft with artifact {2}{U}"
            .to_string(),
        power: None,
        toughness: None,
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Craft),
            AbilityDefinition::Craft {
                cost: ManaCost {
                    generic: 2,
                    blue: 1,
                    ..Default::default()
                },
                materials: CraftMaterials::Artifacts(1),
            },
            // CR 603.1: ETB trigger — tap target creature an opponent controls.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::TapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],
                modes: None,
                trigger_zone: None,
            },
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Braided Quipu".to_string(),
            mana_cost: None,
            types: types(&[CardType::Artifact]),
            oracle_text: "When Braided Quipu enters the battlefield, tap target creature an \
                          opponent controls.\nWhenever you cast a spell, draw a card."
                .to_string(),
            power: None,
            toughness: None,
            abilities: vec![
                // CR 603.1: ETB trigger — tap target creature an opponent controls.
                AbilityDefinition::Triggered {
                    once_per_turn: false,
                    trigger_condition: TriggerCondition::WhenEntersBattlefield,
                    effect: Effect::TapPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    intervening_if: None,
                    targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                        controller: TargetController::Opponent,
                        ..Default::default()
                    })],
                    modes: None,
                    trigger_zone: None,
                },
                // "Whenever you cast a spell, draw a card." — fires on any spell.
                AbilityDefinition::Triggered {
                    once_per_turn: false,
                    trigger_condition: TriggerCondition::WheneverYouCastSpell {
                        during_opponent_turn: false,
                        spell_type_filter: None,
                        noncreature_only: false,
                        chosen_subtype_filter: false,
                        spell_subtype_filter: None,
                    },
                    effect: Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    intervening_if: None,
                    targets: vec![],
                    modes: None,
                    trigger_zone: None,
                },
            ],
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
        completeness: Completeness::Complete,
    }
}
