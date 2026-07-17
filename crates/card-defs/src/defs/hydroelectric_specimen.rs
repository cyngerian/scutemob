// Hydroelectric Specimen // Hydroelectric Laboratory — {2}{U} Creature — Weird 1/4
// Flash
// When this creature enters, you may change the target of target instant or sorcery
// spell with a single target to this creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hydroelectric-specimen"),
        name: "Hydroelectric Specimen // Hydroelectric Laboratory".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Weird"]),
        oracle_text: "Flash\nWhen this creature enters, you may change the target of target \
                      instant or sorcery spell with a single target to this creature."
            .to_string(),
        power: Some(1),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            // TODO: ETB trigger — redirect target of an instant/sorcery spell.
            // DSL gap: target redirection effect not expressible.
        ],
        completeness: Completeness::partial(
            "Blocked: Effect::ChangeTargets (PB-J) exists but has no 'new target' parameter — \
             must_change retargets to the effect's controller, not to a named object, so 'change \
             the target ... to this creature' would redirect to the player instead. Needs a \
             ChangeTargets new-target argument (EffectTarget::Source). Also 'you may' is not \
             expressible. MDFC back face (Hydroelectric Laboratory) is missing.",
        ),
        ..Default::default()
    }
}
