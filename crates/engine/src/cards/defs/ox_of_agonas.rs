// 71. Ox of Agonas — {3}{R}{R}, Creature — Ox 4/2; ETB: discard hand, draw 3.
// Escape — {R}{R}, Exile eight other cards from your graveyard.
//
// ETB effect approximated: DiscardCards up to 7 then DrawCards 3 (interactive
// hand discard deferred). Escape cost and exile count accurate per oracle text.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ox-of-agonas"),
        name: "Ox of Agonas".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 2, ..Default::default() }),
        types: creature_types(&["Ox"]),
        oracle_text: "When Ox of Agonas enters the battlefield, discard your hand, then draw three cards.\nEscape — {R}{R}, Exile eight other cards from your graveyard. (You may cast this card from your graveyard for its escape cost.)".to_string(),
        power: Some(4),
        toughness: Some(2),
        abilities: vec![
            // CR 702.138a: Escape keyword marker for quick presence-check.
            AbilityDefinition::Keyword(KeywordAbility::Escape),
            // CR 702.138a: Escape cost ({R}{R}) and exile count (8).
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Escape,
                cost: ManaCost { red: 2, ..Default::default() },
                details: Some(AltCastDetails::Escape { exile_count: 8 }),
            },
            // ETB: discard hand then draw 3.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                intervening_if: None,
                targets: vec![],
                effect: Effect::Sequence(vec![
                    Effect::DiscardCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(7),
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(3),
                    },
                ]),
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
    }
}
