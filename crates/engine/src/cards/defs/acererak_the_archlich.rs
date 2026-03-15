// Acererak the Archlich — {2}{B}, Legendary Creature — Zombie Wizard 5/5
// When Acererak the Archlich enters the battlefield, if you haven't completed
// Tomb of Annihilation, return Acererak the Archlich to its owner's hand and
// venture into the dungeon.
// Whenever Acererak the Archlich attacks, for each opponent, that player creates
// a 2/2 black Zombie creature token unless that player sacrifices a creature.
//
// CR 309.7: "if you haven't completed Tomb of Annihilation" — Condition::Not wrapping
// CompletedSpecificDungeon(TombOfAnnihilation). Intervening-if checked at trigger
// time and at resolution (CR 603.4).
// CR 701.49a-c: Venture into the dungeon.
// CR 111.10: Token creation (2/2 black Zombie).
//
// Simplification: The attack trigger's "unless that player sacrifices a creature"
// is an interactive player choice. Simplified to always create zombie tokens for
// each opponent. Interactive "unless sacrifice" deferred to M10+ interactive choices.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("acererak-the-archlich"),
        name: "Acererak the Archlich".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Zombie", "Wizard"],
        ),
        oracle_text: "When Acererak the Archlich enters the battlefield, if you haven't completed Tomb of Annihilation, return Acererak the Archlich to its owner's hand and venture into the dungeon.\nWhenever Acererak the Archlich attacks, for each opponent, that player creates a 2/2 black Zombie creature token unless that player sacrifices a creature."
            .to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            // CR 603.4: ETB trigger with intervening-if — "if you haven't completed
            // Tomb of Annihilation, return Acererak to its owner's hand and venture."
            // Condition is re-evaluated at resolution (CR 603.4).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    // CR 400.7: Return to owner's hand — zone change creates new object identity.
                    Effect::MoveZone {
                        target: EffectTarget::Source,
                        to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    },
                    // CR 701.49a-c: Venture into the dungeon.
                    Effect::VentureIntoDungeon,
                ]),
                // CR 603.4: "if you haven't completed Tomb of Annihilation"
                intervening_if: Some(Condition::Not(Box::new(
                    Condition::CompletedSpecificDungeon(DungeonId::TombOfAnnihilation),
                ))),
                targets: vec![],
            },
            // CR 603.1: Attack trigger — for each opponent, create a 2/2 black Zombie token.
            // Simplification 1: "unless that player sacrifices a creature" is interactive;
            // deterministic fallback creates zombie tokens for all opponents. Deferred to M10+.
            // Simplification 2 (token controller): ForEach { EachOpponent, CreateToken } keeps
            // ctx.controller as the Acererak player, so tokens are created under the Acererak
            // controller's control. The oracle text says "that player creates" — tokens should be
            // under each opponent's control. Fixing this requires EffectTarget::CurrentIterationPlayer
            // or a CreateTokenUnderTargetControl effect variant.
            // TODO(M10+): tokens should be created under each opponent's control.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::CreateToken {
                        spec: TokenSpec {
                            name: "Zombie".to_string(),
                            power: 2,
                            toughness: 2,
                            colors: [Color::Black].iter().copied().collect(),
                            card_types: [CardType::Creature].iter().copied().collect(),
                            subtypes: [SubType("Zombie".to_string())].iter().cloned().collect(),
                            count: 1,
                            ..Default::default()
                        },
                    }),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
    }
}
