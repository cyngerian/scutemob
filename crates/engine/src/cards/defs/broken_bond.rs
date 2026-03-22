// Broken Bond — {1}{G} Sorcery
// Destroy target artifact or enchantment. You may put a land card from your hand onto the
// battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("broken-bond"),
        name: "Broken Bond".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text:
            "Destroy target artifact or enchantment. You may put a land card from your hand onto the battlefield."
                .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 701.8: Destroy target artifact or enchantment.
            // TODO: The second effect ("you may put a land card from your hand onto the
            // battlefield") requires a PutCardFromHandOntoBattlefield effect with a Land filter.
            // No such primitive exists yet. When it is added, wrap in Effect::Sequence with
            // DestroyPermanent and the new land-play effect.
            effect: Effect::DestroyPermanent {
                target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
