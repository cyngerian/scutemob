// Nissa, Resurgent Animist — {2}{G}, Legendary Creature — Elf Scout 3/3
// Landfall — Whenever a land you control enters, add one mana of any color. Then if
// this is the second time this ability has resolved this turn, reveal cards from the
// top of your library until you reveal an Elf or Elemental card. Put that card into
// your hand and the rest on the bottom of your library in a random order.
//
// TODO: "second time this ability has resolved this turn" conditional — no per-ability
// resolution counter exists. Only the mana production part is implemented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nissa-resurgent-animist"),
        name: "Nissa, Resurgent Animist".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elf", "Scout"],
        ),
        oracle_text: "Landfall — Whenever a land you control enters, add one mana of any color. Then if this is the second time this ability has resolved this turn, reveal cards from the top of your library until you reveal an Elf or Elemental card. Put that card into your hand and the rest on the bottom of your library in a random order.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // Landfall — add one mana of any color.
            // TODO: "second resolution" conditional — reveal Elf/Elemental from library.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
