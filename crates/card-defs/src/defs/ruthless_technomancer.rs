// Ruthless Technomancer — {3}{B}, Creature — Human Wizard 2/4
// When this creature enters, you may sacrifice another creature you control. If you do,
// create a number of Treasure tokens equal to that creature's power.
// {2}{B}, Sacrifice X artifacts: Return target creature card with power X or less from
// your graveyard to the battlefield. X can't be 0.
//
// ETB clause authored below (CR 118.12 / 109.1). The two blockers the header previously
// claimed are FALSE at HEAD: `can_pay_optional_cost` / `pay_optional_cost` DO thread a
// `source: Option<ObjectId>` (effects/mod.rs:9331-9337) into `sacrifice_permanents_for_player`
// (effects/mod.rs:9404-9445), and `TargetFilter.exclude_self` (card_definition.rs:3249) is
// enforced there — the identical `disciple_of_freyalise.rs` shape closed this exact class in
// PB-EF1/PB-OS2. `EffectAmount::PowerOfSacrificedCreature` (card_definition.rs:2792) already
// captures the sacrificed permanent's LKI power (effects/mod.rs:9442-9445), and
// `dockside_extortionist.rs` is precedent for a dynamic `TokenSpec.count`.
//
// ENGINE-BLOCKED (activated ability): "{2}{B}, Sacrifice X artifacts: Return target
// creature card with power X or less from your graveyard to the battlefield. X
// can't be 0." No `Cost` variant supports a player-chosen variable-X sacrifice count, and
// `TargetFilter.max_power` is a static `i32` (card_definition.rs:3049) with no
// `max_power_amount` sibling to tie it to the chosen X. Genuinely blocked.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ruthless-technomancer"),
        name: "Ruthless Technomancer".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            black: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "When this creature enters, you may sacrifice another creature you control. \
                      If you do, create a number of Treasure tokens equal to that creature's \
                      power.\n{2}{B}, Sacrifice X artifacts: Return target creature card with \
                      power X or less from your graveyard to the battlefield. X can't be 0."
            .to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            // CR 118.12 / 109.1: "When this creature enters, you may sacrifice another creature
            // you control. If you do, create a number of Treasure tokens equal to that
            // creature's power." exclude_self: true is "another creature"; the Treasure count
            // is the sacrificed creature's LKI power (CR 608.2h/i).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::MayPayThenEffect {
                    cost: Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::You,
                        exclude_self: true,
                        ..Default::default()
                    }),
                    payer: PlayerTarget::Controller,
                    then: Box::new(Effect::CreateToken {
                        spec: TokenSpec {
                            count: EffectAmount::PowerOfSacrificedCreature,
                            ..treasure_token_spec(1)
                        },
                    }),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // ENGINE-BLOCKED: see module comment -- the activated variable-X sacrifice ability
            // is blocked on real DSL gaps (no player-chosen variable-X Cost, no dynamic
            // max_power_amount graveyard filter).
        ],
        completeness: Completeness::partial(
            "ETB clause (CR 118.12, optional sacrifice -> Treasure count = sacrificed creature's \
             power) is authored. Still blocked on the activated ability: '{2}{B}, Sacrifice X \
             artifacts: return target creature card with power X or less...' has no Cost variant \
             for a player-chosen variable-X sacrifice count, and TargetFilter.max_power is a \
             static i32 with no max_power_amount sibling to tie it to the chosen X.",
        ),
        ..Default::default()
    }
}
