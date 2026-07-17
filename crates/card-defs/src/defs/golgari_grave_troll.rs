// Golgari Grave-Troll — {4}{G}, Creature — Troll Skeleton 0/0
// This creature enters with a +1/+1 counter on it for each creature card in your graveyard.
// {1}, Remove a +1/+1 counter from this creature: Regenerate this creature.
// Dredge 6
//
// OOS-EWC-2 (2026-05-15): self-ETB EntersWithCounters authored using
// `EffectAmount::CardCount { zone: Graveyard(Controller), player: Controller,
// filter: TargetFilter { has_card_type: Creature } }`. Mirrors PB-EWC's
// Ingenious Prodigy / Master Biomancer pattern. CR 614.1c — counters are
// placed at ETB time by `apply_self_etb_from_definition`, which builds an
// `EffectContext` pinned to the entering object and calls `resolve_amount`.
//
// **Engine ordering caveat — ruling 2018-12-07 NOT honored today**:
// Ruling 2018-12-07 ("If you return Golgari Grave-Troll from your graveyard
// directly to the battlefield, its first ability counts itself") would
// require the EWC count to evaluate while the Troll is still in the
// graveyard. The current engine in `rules/resolution.rs:519-520` runs
// `move_object_to_zone(source_object, ZoneId::Battlefield)` BEFORE the
// `apply_self_etb_from_definition` call at line 1606; by then, the Troll
// is in `ZoneId::Battlefield`, not its origin zone. For casts from hand
// (this PR's tested path) the result is correct — the Troll wasn't in
// the graveyard to begin with. For reanimate-from-graveyard the engine
// will under-count by one. Filed as an OOS follow-up (engine ordering
// fix is out of scope for this card-authoring task; see PB-EWC handoff
// in `memory/workstream-state.md`).
//
// CR 702.52a: Dredge 6 — if you would draw a card, you may instead mill 6
// cards and return this card from your graveyard to your hand. Functions only
// while this card is in the graveyard. Requires >= 6 cards in library
// (CR 702.52b). Engine machinery already exists (rules/replacement.rs
// `DredgeAvailable` + Command::ChooseDredge).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("golgari-grave-troll"),
        name: "Golgari Grave-Troll".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Troll", "Skeleton"]),
        oracle_text: "This creature enters with a +1/+1 counter on it for each creature card in \
                      your graveyard.\n{1}, Remove a +1/+1 counter from this creature: Regenerate \
                      this creature.\nDredge 6 (If you would draw a card, you may mill six cards \
                      instead. If you do, return this card from your graveyard to your hand.)"
            .to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![
            // CR 614.1c: self-replacement — "This creature enters with a +1/+1
            // counter on it for each creature card in your graveyard."
            //
            // `is_self: true` + `ObjectFilter::Any` together restrict the
            // replacement to the entering permanent that owns this ability
            // (CR 614.15). `EffectAmount::CardCount` counts creature cards in
            // the controller's graveyard at the moment ETB is processed.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersWithCounters {
                    counter: CounterType::PlusOnePlusOne,
                    count: Box::new(EffectAmount::CardCount {
                        zone: ZoneTarget::Graveyard {
                            owner: PlayerTarget::Controller,
                        },
                        player: PlayerTarget::Controller,
                        filter: Some(TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            ..Default::default()
                        }),
                    }),
                },
                is_self: true,
                unless_condition: None,
            },
            // CR 702.52a: Dredge 6 marker.
            AbilityDefinition::Keyword(KeywordAbility::Dredge(6)),
            // CR 602.2: {1}, Remove a +1/+1 counter from this creature: Regenerate.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 1,
                        ..Default::default()
                    }),
                    Cost::RemoveCounter {
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    },
                ]),
                effect: Effect::Regenerate {
                    target: EffectTarget::Source,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
