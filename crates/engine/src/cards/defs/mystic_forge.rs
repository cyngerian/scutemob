// Mystic Forge — {4}, Artifact
// You may look at the top card of your library any time.
// You may cast artifact spells and colorless spells from the top of your library.
// {T}, Pay 1 life: Exile the top card of your library.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mystic-forge"),
        name: "Mystic Forge".to_string(),
        mana_cost: Some(ManaCost { generic: 4, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "You may look at the top card of your library any time.\nYou may cast artifact spells and colorless spells from the top of your library.\n{T}, Pay 1 life: Exile the top card of your library.".to_string(),
        abilities: vec![
            // CR 601.3 (PB-A): "You may look at the top card of your library any time.
            // You may cast artifact spells and colorless spells from the top of your library."
            // ArtifactsAndColorless filter: artifact spells + spells with no colored mana in cost.
            // 2019-07-12 ruling: "cast" not "play" — artifact lands CANNOT be played from top.
            // Morph face-down spells are colorless and thus allowed (plan edge case).
            AbilityDefinition::StaticPlayFromTop {
                filter: PlayFromTopFilter::ArtifactsAndColorless,
                look_at_top: true,
                reveal_top: false,
                pay_life_instead: false,
                condition: None,
                on_cast_effect: None,
            },
            // TODO: "{T}, Pay 1 life: Exile the top card of your library."
            // Requires ExileTopOfLibrary effect variant — DSL gap. Deferred.
            // The play-from-top ability above is the primary implementation.
        ],
        ..Default::default()
    }
}
