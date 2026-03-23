// Ezuri, Renegade Leader — {1}{G}{G}, Legendary Creature — Elf Warrior 2/2
// {G}: Regenerate another target Elf.
// {2}{G}{G}{G}: Elf creatures you control get +3/+3 and gain trample until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ezuri-renegade-leader"),
        name: "Ezuri, Renegade Leader".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elf", "Warrior"],
        ),
        oracle_text: "{G}: Regenerate another target Elf.\n{2}{G}{G}{G}: Elf creatures you control get +3/+3 and gain trample until end of turn.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // {G}: Regenerate another target Elf.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { green: 1, ..Default::default() }),
                effect: Effect::Regenerate {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: None,
                // NOTE: oracle says "another target Elf" — exclude_self not on TargetFilter.
                // TODO: DSL gap — "another" self-exclusion on TargetRequirement not in DSL.
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    has_subtype: Some(SubType("Elf".to_string())),
                    ..Default::default()
                })],
                activation_condition: None,
            },
            // {2}{G}{G}{G}: Elf creatures you control get +3/+3 and gain trample until EOT.
            // TODO: DSL gap — mass pump targeting "Elf creatures you control" (not "other").
            // EffectFilter::CreaturesYouControlWithSubtype does not exist (only Other variant).
            // Using empty abilities as placeholder to avoid wrong game state.
        ],
        ..Default::default()
    }
}
