// Sokenzan, Crucible of Defiance — Legendary Land; {T}: Add {R};
// Channel — {3}{R}, Discard: Create two 1/1 Spirit tokens with haste (cost reduction per legendary creature).
// TODO: Channel ability (discard from hand, cost reduction) not expressible in current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sokenzan-crucible-of-defiance"),
        name: "Sokenzan, Crucible of Defiance".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {R}.\nChannel — {3}{R}, Discard this card: Create two 1/1 colorless Spirit creature tokens. They gain haste until end of turn. This ability costs {1} less to activate for each legendary creature you control.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 1, 0, 0),
                },
                timing_restriction: None,
            },
            // TODO: Channel ability — discard-from-hand cost, token creation with haste,
            // cost reduction per legendary creature. Requires Channel keyword enforcement + DSL gaps.
        ],
        ..Default::default()
    }
}
