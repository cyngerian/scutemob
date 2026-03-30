// Teferi's Protection — {2}{W}, Instant
// Until your next turn, your life total can't change and you gain protection from everything.
// All permanents you control phase out.
// Exile Teferi's Protection.
//
// CR 702.16j: "you gain protection from everything" — implemented via GrantPlayerProtection.
// CR 611.2b: UntilYourNextTurn duration applied to the protection grant.
//
// TODO: "your life total can't change" — needs a continuous prevention effect until next turn.
// TODO: "All permanents you control phase out" — Effect::PhaseOut for all controller permanents.
// TODO: "Exile Teferi's Protection" — self-exile on resolution.
// Note: PlayerId(0) is a placeholder; replaced with the actual controller at resolution.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("teferis-protection"),
        name: "Teferi's Protection".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Until your next turn, your life total can't change and you gain protection from everything. All permanents you control phase out. (While they're phased out, they're treated as though they don't exist. They phase in before you untap during your untap step.)\nExile Teferi's Protection.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                // CR 702.16j: "you gain protection from everything until your next turn."
                // CR 611.2b: UntilYourNextTurn duration — expires at start of your next turn.
                effect: Effect::GrantPlayerProtection {
                    player: PlayerTarget::Controller,
                    qualities: vec![ProtectionQuality::FromAll],
                    duration: Some(crate::state::EffectDuration::UntilYourNextTurn(
                        crate::state::player::PlayerId(0),
                    )),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
