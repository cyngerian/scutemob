// Cover of Darkness — {1}{B}, Enchantment
// As Cover of Darkness enters, choose a creature type.
// Creatures of the chosen type have fear.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cover-of-darkness"),
        name: "Cover of Darkness".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            black: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "As Cover of Darkness enters, choose a creature type.\nCreatures of the \
                      chosen type have fear. (They can't be blocked except by artifact creatures \
                      and/or black creatures.)"
            .to_string(),
        abilities: vec![
            // TODO: "As this enters, choose a creature type" — no ChooseCreatureType
            // effect or chosen_subtype field on GameObject. The static grant of Fear
            // depends on the chosen type (Layer 6).
        ],
        completeness: Completeness::partial(
            "'As this enters, choose a creature type' IS expressible \
             (ReplacementModification::ChooseCreatureType, as used by cavern_of_souls.rs:17-24; \
             chosen type is read back via EffectFilter's *OfChosenType variants). Real remaining \
             gap: the fear grant applies to creatures of the chosen type controlled by ANY \
             player, and EffectFilter has no AllCreaturesOfChosenType (only \
             CreaturesYouControl/OtherCreaturesYouControl OfChosenType, \
             continuous_effect.rs:214-218). Using the you-control variant would silently \
             under-apply the static in multiplayer.",
        ),
        ..Default::default()
    }
}
