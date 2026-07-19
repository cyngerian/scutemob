// Silumgar, the Drifting Death — {4}{U}{B}, Legendary Creature — Dragon 3/7
// Flying, hexproof
// Whenever a Dragon you control attacks, creatures defending player controls get -1/-1
// until end of turn.
//
// CR 508.4 (defending player), CR 611.2a (continuous effect locked in at resolution),
// CR 613.1d/613.4c (Layer 7c P/T modification), CR 514.2 (until-end-of-turn cleanup),
// CR 704.5f (0-toughness SBA), CR 205.3m (Dragon subtype). Ruling 2014-11-24: the
// affected set is relative to the SPECIFIC attacking Dragon — each attacking Dragon you
// control triggers separately, and the -1/-1 scope is that Dragon's defending player
// (per-attacker independence, PB-EF3 defending_player_id capture).
//
// PB-OS7 / OOS-EF3-1: uses the new EffectFilter::CreaturesControlledByDefendingPlayer
// DSL placeholder, substituted at ApplyContinuousEffect execution time into
// CreaturesControlledBy(ctx.defending_player).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("silumgar-the-drifting-death"),
        name: "Silumgar, the Drifting Death".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            blue: 1,
            black: 1,
            ..Default::default()
        }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Dragon"]),
        oracle_text: "Flying, hexproof\nWhenever a Dragon you control attacks, creatures \
                      defending player controls get -1/-1 until end of turn."
            .to_string(),
        power: Some(3),
        toughness: Some(7),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Hexproof),
            // CR 508.1m/508.4 / CR 205.3m: "Whenever a Dragon you control attacks,
            // creatures defending player controls get -1/-1 until end of turn." Fires
            // once per attacking Dragon (WheneverCreatureYouControlAttacks with a Dragon
            // subtype filter); the -1/-1 is scoped to the per-attacker defending player
            // via CreaturesControlledByDefendingPlayer, substituted at
            // ApplyContinuousEffect execution time (PB-OS7 / OOS-EF3-1).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Dragon".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBoth(-1),
                        filter: EffectFilter::CreaturesControlledByDefendingPlayer,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
