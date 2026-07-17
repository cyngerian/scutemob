// Timeline Culler — {B}{B}, Creature — Drix Warlock 2/2
// Haste
// You may cast this card from your graveyard using its warp ability.
// Warp—{B}, Pay 2 life. (PB-AC5: Warp implemented via AltCostKind::Warp.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("timeline-culler"),
        name: "Timeline Culler".to_string(),
        mana_cost: Some(ManaCost {
            black: 2,
            ..Default::default()
        }),
        types: creature_types(&["Drix", "Warlock"]),
        oracle_text: "Haste\nYou may cast this card from your graveyard using its warp \
                      ability.\nWarp\u{2014}{B}, Pay 2 life. (You may cast this card from your \
                      hand or graveyard for its warp cost. If you do, exile this creature at the \
                      beginning of the next end step, then you may cast it from exile on a later \
                      turn.)"
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            AbilityDefinition::Keyword(KeywordAbility::Warp),
            // CR 702.185a: Warp—{B}, Pay 2 life. `from_graveyard: true` grants the
            // "You may cast this card from your graveyard using its warp ability" permission.
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Warp,
                cost: ManaCost {
                    black: 1,
                    ..Default::default()
                },
                details: Some(AltCastDetails::Warp {
                    costs: vec![Cost::PayLife(2)],
                    from_graveyard: true,
                }),
            },
        ],
        ..Default::default()
    }
}
