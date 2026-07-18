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
            // TODO: "Creatures you control have '{T}: Add one mana of any color.'"
            // The grant primitive (LayerModification::AddManaAbility(ManaAbility{ any_color:
            // true, .. }) + EffectFilter::CreaturesYouControl) exists structurally, but
            // NOT implementing it here — empirically verified (rules/mana.rs:337-365,
            // handle_tap_for_mana) that `any_color: true` mana abilities are STUBBED:
            // "Simplified: colorless until interactive color choice is implemented" —
            // they always add ManaColor::Colorless, never a real chosen color. This is
            // the same class of defect as the gated Effect::AddManaAnyColor family
            // (SR-37). enduring_vitality.rs, despite carrying the identical grant, is
            // itself still `partial` (for its unrelated Enduring-cycle clause) and its
            // any-color grant has NOT been certified against this stub — so it is not
            // valid precedent for marking this Complete. Wiring the grant here would
            // silently make every creature you control tap for colorless instead of
            // any color, which is wrong game state (W5), not merely incomplete —
            // deliberately left unauthored rather than shipped known_wrong.
        ],
        completeness: Completeness::partial(
            "'Creatures you control have \"{T}: Add one mana of any color.\"' is blocked on a \
             real engine defect, not a missing DSL primitive: handle_tap_for_mana \
             (rules/mana.rs:337-365) stubs every any_color ManaAbility to ManaColor::Colorless, \
             so the grant would ship wrong game state (always colorless mana) if authored. Needs \
             interactive color choice at mana-ability resolution before this clause can be \
             Complete. The other two clauses (look-at-top + cast-from-top) are already \
             implemented.",
        ),
        ..Default::default()
    }
}
