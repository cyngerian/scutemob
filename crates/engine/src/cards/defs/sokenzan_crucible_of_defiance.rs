// Sokenzan, Crucible of Defiance — Legendary Land; {T}: Add {R};
// Channel — {3}{R}, Discard: Create two 1/1 Spirit tokens with haste (cost reduction per legendary creature).
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
                targets: vec![],
            },
            // Channel — {3}{R}, Discard this card: Create two 1/1 colorless Spirit tokens.
            // TODO: Tokens should gain haste until end of turn (temporary keyword grant).
            // TODO: Cost reduction — {1} less per legendary creature you control.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 3, red: 1, ..Default::default() }),
                    Cost::DiscardSelf,
                ]),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Spirit".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: im::OrdSet::new(),
                        supertypes: im::OrdSet::new(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Spirit".to_string())].into_iter().collect(),
                        keywords: im::OrdSet::new(),
                        count: 2,
                        tapped: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                timing_restriction: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
