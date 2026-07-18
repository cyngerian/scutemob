// Elven Chorus — {3}{G}, Enchantment
// You may look at the top card of your library any time.
// You may cast creature spells from the top of your library.
// Creatures you control have "{T}: Add one mana of any color."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elven-chorus"),
        name: "Elven Chorus".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "You may look at the top card of your library any time.\nYou may cast \
                      creature spells from the top of your library.\nCreatures you control have \
                      \"{T}: Add one mana of any color.\""
            .to_string(),
        abilities: vec![
            // CR 601.3 (PB-A): "You may look at the top card of your library any time.
            // You may cast creature spells from the top of your library."
            // look_at_top: true (controller sees top; not all players — distinguish from reveal_top).
            AbilityDefinition::StaticPlayFromTop {
                filter: PlayFromTopFilter::CreaturesOnly,
                look_at_top: true,
                reveal_top: false,
                pay_life_instead: false,
                condition: None,
                on_cast_effect: None,
            },
            // CR 613.1f: Layer 6 static ability — grants tap-for-any-color mana ability
            // to each creature you control while this permanent is on the battlefield.
            // PB-EF12 (EF-W-PB2-3): `any_color: true` ManaAbility grants now resolve to a
            // real chosen colour (CR 605.3b/111.10a — the colour is chosen on the
            // `Command::TapForMana` that activates the granted ability), not
            // ManaColor::Colorless. Same grant pattern as Cryptolith Rite / Enduring
            // Vitality.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddManaAbility(ManaAbility {
                        produces: Default::default(),
                        requires_tap: true,
                        sacrifice_self: false,
                        any_color: true,
                        damage_to_controller: 0,
                        ..Default::default()
                    }),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
