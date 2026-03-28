// Olivia Voldaren — {2}{B}{R}, Legendary Creature — Vampire 3/3
// Flying
// {1}{R}: Olivia Voldaren deals 1 damage to another target creature. That creature becomes
// a Vampire in addition to its other types. Put a +1/+1 counter on Olivia Voldaren.
// {3}{B}{B}: Gain control of target Vampire for as long as you control Olivia Voldaren.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("olivia-voldaren"),
        name: "Olivia Voldaren".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Vampire"],
        ),
        oracle_text: "Flying\n{1}{R}: Olivia Voldaren deals 1 damage to another target creature. That creature becomes a Vampire in addition to its other types. Put a +1/+1 counter on Olivia Voldaren.\n{3}{B}{B}: Gain control of target Vampire for as long as you control Olivia Voldaren.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — "{1}{R}: deal 1 damage + add Vampire subtype + +1/+1 counter on self"
            // Needs: DealDamage to target + AddSubtype continuous effect + AddCounter on Source
            // in a single activated ability. AddSubtype LayerModification may not exist.
            // CR 613.1b: {3}{B}{B}: Gain control of target Vampire for as long as you control
            // Olivia Voldaren (WhileSourceOnBattlefield approximation).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 3, black: 2, ..Default::default() }),
                effect: Effect::GainControl {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    has_subtype: Some(SubType("Vampire".to_string())),
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
