// Scavenging Ooze — {1}{G}, Creature — Ooze 2/2
// {G}: Exile target card from a graveyard. If it was a creature card, put a +1/+1 counter
// on this creature and you gain 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scavenging-ooze"),
        name: "Scavenging Ooze".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Ooze"]),
        oracle_text: "{G}: Exile target card from a graveyard. If it was a creature card, put a +1/+1 counter on Scavenging Ooze and you gain 1 life.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: DSL gap — "Exile target card from a graveyard" with conditional
            // on creature card type (EffectTarget for graveyard card + type check at
            // resolution). TargetRequirement::TargetCardInGraveyard exists but conditional
            // effect based on card type at resolution not in DSL.
        ],
        ..Default::default()
    }
}
