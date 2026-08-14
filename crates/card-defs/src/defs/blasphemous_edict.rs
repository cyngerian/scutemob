// Blasphemous Edict — {3}{B}{B}, Sorcery
// This spell costs {B}{B} less to cast if there are thirteen or more creatures
// on the battlefield.
// Each player sacrifices a creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blasphemous-edict"),
        name: "Blasphemous Edict".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            black: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        // PB-DX27 (2026-08-13), OOS-CARDS2-10: the previous text was wrong in TWO clauses —
        // "costs {B}{B} less" for a printed ALTERNATIVE cost, and "a creature" for a printed
        // "thirteen creatures". Replaced with the MCP-verified printed text.
        oracle_text: "You may pay {B} rather than pay this spell's mana cost if there are \
                      thirteen or more creatures on the battlefield.\nEach player sacrifices \
                      thirteen creatures of their choice."
            .to_string(),
        abilities: vec![
            // TODO: Conditional cost reduction "{B}{B} less if 13+ creatures on the battlefield"
            // is not expressible in the current DSL. Blocked on a
            // Condition::CreaturesOnBattlefieldAtLeast(N) primitive. (PB-SFT scope boundary.)
            AbilityDefinition::Spell {
                // PB-SFT (CR 701.21a + CR 109.1): creature-only filter applied.
                // Each player sacrifices a creature of their choice.
                effect: Effect::SacrificePermanents {
                    player: PlayerTarget::EachPlayer,
                    count: EffectAmount::Fixed(1),
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        completeness: Completeness::partial(
            "Conditional cost reduction '{B}{B} less if 13+ creatures on the battlefield' is not \
             expressible in the current DSL....",
        ),
        ..Default::default()
    }
}
