// Kaito Shizuki — {1}{U}{B}, Legendary Planeswalker — Kaito
// At the beginning of your end step, if Kaito entered this turn, he phases out.
// +1: Draw a card. Then discard a card unless you attacked this turn.
// −2: Create a 1/1 blue Ninja creature token with "This token can't be blocked."
// −7: You get an emblem with "Whenever a creature you control deals combat damage to a
//     player, search your library for a blue or black creature card, put it onto the
//     battlefield, then shuffle."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kaito-shizuki"),
        name: "Kaito Shizuki".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            black: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Kaito"],
        ),
        oracle_text: "At the beginning of your end step, if Kaito Shizuki entered this turn, he \
                      phases out.\n+1: Draw a card. Then discard a card unless you attacked this \
                      turn.\n\u{2212}2: Create a 1/1 blue Ninja creature token with \"This token \
                      can't be blocked.\"\n\u{2212}7: You get an emblem with \"Whenever a \
                      creature you control deals combat damage to a player, search your library \
                      for a blue or black creature card, put it onto the battlefield, then \
                      shuffle.\""
            .to_string(),
        starting_loyalty: Some(3),
        abilities: vec![
            // ENGINE-BLOCKED: "if Kaito Shizuki entered this turn, he phases out" — needs an
            // entered-this-turn condition on an end-step trigger plus a self-phase-out effect.
            //
            // +1 is fully expressible as of PB-AC6 (Condition::YouAttackedThisTurn, CR 508.1):
            // the discard is mandatory unless the raid condition is met.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::Conditional {
                        condition: Condition::YouAttackedThisTurn,
                        if_true: Box::new(Effect::Nothing),
                        if_false: Box::new(Effect::DiscardCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(1),
                        }),
                    },
                ]),
                targets: vec![],
            },
            // −2: "Create a 1/1 blue Ninja creature token with 'This token can't be blocked.'"
            // The claimed blocker is FALSE at HEAD: KeywordAbility::CantBeBlocked is a real
            // variant (types.rs:503-507, enforced in rules/combat.rs::handle_declare_blockers)
            // and TokenSpec.keywords: OrdSet<KeywordAbility> (card_definition.rs:3990) carries
            // it directly — no static-ability plumbing needed, the printed clause IS a
            // rules-text keyword on the token. Precedent: basri_ket.rs's +1/-6 LoyaltyAbility
            // -> CreateToken shape.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Ninja".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: [Color::Blue].into_iter().collect(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Ninja".to_string())].into_iter().collect(),
                        keywords: [KeywordAbility::CantBeBlocked].into_iter().collect(),
                        count: EffectAmount::Fixed(1),
                        ..Default::default()
                    },
                },
                targets: vec![],
            },
            // ENGINE-BLOCKED: −7 emblem with combat damage -> search library. `Effect::CreateEmblem`
            // and `TriggerEvent::AnyCreatureYouControlDealsCombatDamageToPlayer` both exist, BUT
            // `collect_emblem_triggers_for_event` (abilities.rs:7197) is called from exactly six
            // sites — turn_actions.rs:356/362/821/1981, abilities.rs:3754/3760 — and NONE is a
            // combat-damage dispatch site. Authoring this ships a 7-loyalty ability that silently
            // does nothing, worse than the honest omission. Genuinely blocked on emblem-trigger
            // dispatch at the combat-damage site, not on Effect::CreateEmblem itself.
        ],
        completeness: Completeness::partial(
            "-2 (CR 701.15a-style token grant, oracle-verified) is authored. Still blocked: the \
             end-step phase-out clause ('if Kaito entered this turn, he phases out') needs an \
             entered-this-turn Condition on an end-step trigger plus a self-phase-out effect — \
             KeywordAbility::Phasing is a static untap-step ability (types.rs:1303-1316), not \
             this. The -7 emblem is blocked on emblem-trigger dispatch not covering the \
             combat-damage site (collect_emblem_triggers_for_event, abilities.rs:7197 — six call \
             sites, none combat-damage).",
        ),
        ..Default::default()
    }
}
