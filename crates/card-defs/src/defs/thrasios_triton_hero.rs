// Thrasios, Triton Hero — {G}{U}, Legendary Creature — Merfolk Wizard 1/3
// {4}: Scry 1, then reveal the top card of your library. If it's a land card, put it onto
// the battlefield tapped. Otherwise, draw a card.
// Partner (You can have two commanders if both have partner.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thrasios-triton-hero"),
        name: "Thrasios, Triton Hero".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            blue: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Merfolk", "Wizard"],
        ),
        oracle_text: "{4}: Scry 1, then reveal the top card of your library. If it's a land card, \
                      put it onto the battlefield tapped. Otherwise, draw a card.\nPartner (You \
                      can have two commanders if both have partner.)"
            .to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Partner),
            // {4}: Scry 1, then reveal top card — land goes to battlefield tapped, else draw.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 4,
                    ..Default::default()
                }),
                effect: Effect::Sequence(vec![
                    Effect::Scry {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::RevealAndRoute {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Land),
                            ..Default::default()
                        },
                        matched_dest: ZoneTarget::Battlefield { tapped: true },
                        unmatched_dest: ZoneTarget::Hand {
                            owner: PlayerTarget::Controller,
                        },
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        // PB-DX4 (2026-08-01, OOS-DP10-8): Complete (by the `#[default]` derive) -> partial.
        //
        // MCP-verified printed text: "{4}: Scry 1, then reveal the top card of your library. If
        // it's a land card, put it onto the battlefield tapped. Otherwise, DRAW A CARD."
        //
        // The `Effect::RevealAndRoute` above sends the non-land case to
        // `unmatched_dest: ZoneTarget::Hand` -- a ZONE MOVE, not a draw. The two are not
        // interchangeable (CR 121.1/704): a zone move emits no draw event, so draw triggers,
        // draw replacement effects (Notion Thief, Leovold, Hullbreacher), PB-DP5's
        // `WouldDraw`/dredge channel, and "can't draw" restrictions are all bypassed.
        //
        // Not expressible today: routing the non-match to a real draw needs a
        // "reveal top; if it matches -> zone, else -> draw" branch that no `Effect` variant
        // provides (`RevealAndRoute`'s destinations are both `ZoneTarget`s, and no `Condition`
        // inspects the revealed card). An engine change, out of scope for this
        // card-def-only batch.
        completeness: Completeness::partial(
            "Printed 'Otherwise, draw a card' is authored as RevealAndRoute's unmatched_dest = \
             ZoneTarget::Hand -- a zone move, not a draw, so no draw event fires and draw \
             triggers, draw replacements (Notion Thief/Leovold/Hullbreacher), the WouldDraw / \
             dredge channel and 'can't draw' restrictions are all bypassed. No Effect variant \
             branches a reveal between a zone destination and a real draw.",
        ),
        ..Default::default()
    }
}
