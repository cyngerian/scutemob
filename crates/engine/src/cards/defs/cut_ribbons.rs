// Cut // Ribbons — Aftermath split card (Amonkhet)
// Cut: {1}{R} Sorcery — Target creature gets -2/-2 until end of turn.
// Ribbons: {X}{B}{B} Sorcery — Aftermath. Each opponent loses X life.
//
// CR 702.127: Aftermath — the second half can only be cast from the graveyard,
// then the card is exiled when it leaves the stack.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cut-ribbons"),
        name: "Cut // Ribbons".to_string(),
        // Cut half: {1}{R}
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Cut — Target creature gets -2/-2 until end of turn.\nRibbons — Aftermath (Cast this spell only from your graveyard. Then exile it.) Each opponent loses X life.".to_string(),
        abilities: vec![
            // CR 702.127a: Aftermath keyword marker — enables graveyard casting of Ribbons half.
            AbilityDefinition::Keyword(KeywordAbility::Aftermath),

            // Cut half: target creature gets -2/-2 until end of turn.
            // CR 613.4c: P/T-modifying effect in layer 7c with UntilEndOfTurn duration.
            AbilityDefinition::Spell {
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBoth(-2),
                        filter: EffectFilter::DeclaredTarget { index: 0 },
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                cant_be_countered: false,
            },

            // Ribbons half: {X}{B}{B}, Aftermath — each opponent loses X life.
            // CR 702.127: cast from graveyard only; exiled when it leaves the stack.
            AbilityDefinition::Aftermath {
                name: "Ribbons".to_string(),
                cost: ManaCost { black: 2, x_count: 1, ..Default::default() },
                card_type: CardType::Sorcery,
                effect: Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::LoseLife {
                        player: PlayerTarget::EachOpponent,
                        amount: EffectAmount::XValue,
                    }),
                },
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
