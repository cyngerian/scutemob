// Thousand-Year Elixir — {3}, Artifact; static haste-for-abilities grant + {1},{T} untap.
// TODO: DSL gap — static ability "creatures you control may activate abilities as though
// they had haste" not expressible (no ContinuousEffectDef for haste-bypass on abilities).
// {1},{T}: Untap target creature is implementable.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thousand-year-elixir"),
        name: "Thousand-Year Elixir".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "You may activate abilities of creatures you control as though those creatures had haste.\n{1}, {T}: Untap target creature.".to_string(),
        abilities: vec![
            // {1},{T}: Untap target creature (CR 701.17).
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::UntapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        // TODO: static ability — "activate creature abilities as though they had haste"
        ..Default::default()
    }
}
