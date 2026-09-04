// Risen Reef — {1}{G}{U}, Creature — Elemental 1/1
// Whenever this or another Elemental you control enters, look at the top card of
// your library. If it's a land, you may put it onto the battlefield tapped.
// If you don't, put it into your hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("risen-reef"),
        name: "Risen Reef".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Elemental"]),
        oracle_text: "Whenever Risen Reef or another Elemental you control enters, look at the \
                      top card of your library. If it's a land card, you may put it onto the \
                      battlefield tapped. If you don't put the card onto the battlefield, put it \
                      into your hand."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                filter: Some(TargetFilter {
                    controller: TargetController::You,
                    has_subtype: Some(SubType("Elemental".to_string())),
                    ..Default::default()
                }),
                exclude_self: false,
            },
            // "**Look at** the top card of your library. If it's a land card, **you may** put
            // it onto the battlefield tapped. If you don't put the card onto the battlefield,
            // put it into your hand."
            //
            // PB-DX4 fix cycle (2026-08-01, `scutemob-168`, review Finding 1): was
            // `Effect::RevealAndRoute`, which has no optionality axis at all — so the printed
            // "you may" was dropped entirely at the DSL level and the controller was FORCED to
            // put the land onto the battlefield. Re-authored onto `LookAtTopThenPlace`, which
            // is also the primitive that matches the printed verb ("look at", not "reveal";
            // the 2019-07-12 ruling turns on exactly that — you need not reveal a card you
            // keep, nor say whether it was a land).
            //
            // **`optional` recorded the "may" structurally while it was INERT**
            // (`effects/mod.rs`'s `LookAtTopThenPlace` arm destructured `optional: _`;
            // pre-existing **OOS-DP10-5**). That was deliberate and is why this def stayed
            // `Complete` rather than joining the batch's demotions: four other `Complete`
            // defs shipped in exactly this position (`birthing_ritual`,
            // `growing_rites_of_itlimoc`, `grisly_salvage`, `satyr_wayfinder`). Filed as the
            // class it is — **OOS-DX4-5** — rather than settled per-card here.
            // **CLOSED by PB-DX35** (`scutemob-227`, 2026-09-04): `optional` is real now —
            // a nonempty candidate set asks
            // `EffectChoiceQuestion::ChooseObject { count: 1, up_to: true, .. }` on the
            // CR 608.2d suspend-and-replay channel, and a decline puts the card into your
            // hand (`rest_to`), the printed fallback verbatim.
            effect: Effect::LookAtTopThenPlace {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
                filter: TargetFilter {
                    has_card_type: Some(CardType::Land),
                    ..Default::default()
                },
                place_cost: None,
                destination: ZoneTarget::Battlefield { tapped: true },
                rest_to: ZoneTarget::Hand {
                    owner: PlayerTarget::Controller,
                },
                optional: true,
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    }
}
