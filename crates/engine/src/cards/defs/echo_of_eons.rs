// Echo of Eons — {4}{U}{U}, Sorcery
// Each player shuffles their hand and graveyard into their library, then draws
// seven cards.
// Flashback {2}{U}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("echo-of-eons"),
        name: "Echo of Eons".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Each player shuffles their hand and graveyard into their library, then draws seven cards.\nFlashback {2}{U} (You may cast this card from your graveyard for its flashback cost. Then exile it.)".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flashback),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Flashback,
                details: None,
                cost: ManaCost { generic: 2, blue: 1, ..Default::default() },
            },
            AbilityDefinition::Spell {
                // TODO: "shuffle hand and graveyard into library" — no Effect for this.
                // Using discard-all + draw-7 as wheel approximation; the shuffle-in of
                // graveyard is not modeled. Full Timetwister effect needs dedicated variant.
                effect: Effect::Sequence(vec![
                    Effect::DiscardCards {
                        player: PlayerTarget::EachPlayer,
                        count: EffectAmount::Fixed(7),
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::EachPlayer,
                        count: EffectAmount::Fixed(7),
                    },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
