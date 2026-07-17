// Frantic Search — {2}{U}, Instant
// Draw two cards, then discard two cards. Untap up to three lands.
//
// TODO: "Then discard two cards" + "untap up to three lands" not expressible.
//   Implementing draw only.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("frantic-search"),
        name: "Frantic Search".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Draw two cards, then discard two cards. Untap up to three lands.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(2),
            },
            // TODO: Discard + untap lands not expressible.
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        completeness: Completeness::partial(
            "needs-rewiring (blocker stale): Effect::DiscardCards (card_definition.rs:1361) and \
             Effect::UntapPermanent (:1449) + TargetRequirement::UpToN { count: 3, inner: \
             TargetLand } (:2798, idiom documented at :1405) all exist. Author as \
             Sequence([DrawCards(2), DiscardCards(Controller, 2), \
             UntapPermanent(DeclaredTarget{0..2})]). Until then the def draws 2 and skips the \
             discard — wrong game state in the caster's favor (known_wrong).",
        ),
        ..Default::default()
    }
}
