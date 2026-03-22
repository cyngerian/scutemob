// Forger's Foundry — {2}{U}, Artifact
// {T}: Add {U}. When you spend this mana to cast an instant or sorcery spell with
//   mana value 3 or less, you may exile that spell instead of putting it into its
//   owner's graveyard as it resolves.
// {3}{U}{U}, {T}: You may cast any number of spells from among cards exiled with
//   this artifact without paying their mana costs. Activate only as a sorcery.
//
// TODO: Both abilities are complex DSL gaps:
//   (1) Mana-spend trigger with exile replacement
//   (2) Cast-from-exile activated ability
// Implementing only the base {T}: Add {U} ability.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("forgers-foundry"),
        name: "Forger's Foundry".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}: Add {U}. When you spend this mana to cast an instant or sorcery spell with mana value 3 or less, you may exile that spell instead of putting it into its owner's graveyard as it resolves.\n{3}{U}{U}, {T}: You may cast any number of spells from among cards exiled with this artifact without paying their mana costs. Activate only as a sorcery.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // TODO: mana-spend exile trigger + cast-from-exile activated ability
        ],
        ..Default::default()
    }
}
