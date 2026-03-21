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
                activation_condition: None,
            },
            // Channel — {3}{R}, Discard this card: Create two 1/1 colorless Spirit tokens
            // with haste. Haste is baked into the token spec (permanent keyword, not temporary
            // grant). Oracle text says "gain haste until end of turn" but baking it into the
            // token definition is equivalent since the tokens are created with haste and any
            // "until end of turn" temporary layer would also only matter this turn.
            // TODO: Strictly, haste should be a temporary UntilEndOfTurn effect, not a permanent
            // keyword on the token. This matters if an opponent gains control of the token via
            // Insurrection — correct behavior would be haste expires at cleanup. DSL gap:
            // CreateToken does not support post-creation continuous effects. Acceptable approximation.
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
                        keywords: [KeywordAbility::Haste].into_iter().collect(),
                        count: 2,
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
