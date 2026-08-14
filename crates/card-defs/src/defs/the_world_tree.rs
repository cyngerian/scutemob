// The World Tree — Land (not Legendary)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("the-world-tree"),
        name: "The World Tree".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\n{T}: Add {G}.\nAs long as you control six or more \
                      lands, lands you control have \"{T}: Add one mana of any \
                      color.\"\n{W}{W}{U}{U}{B}{B}{R}{R}{G}{G}, {T}, Sacrifice this land: Search \
                      your library for any number of God cards, put them onto the battlefield, \
                      then shuffle."
            .to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement — this land enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // CR 613.1f: Layer 6 static — "As long as you control six or more lands,
            // lands you control have '{T}: Add one mana of any color.'"
            // CR 605.1a: the granted ability IS a mana ability (no target, adds mana).
            // The intervening condition is evaluated continuously (CR 613.1d), so the
            // grant switches off the moment the sixth land leaves.
            //
            // PB-DX27 (2026-08-13): this clause was an inline `// TODO` claiming a DSL
            // gap ("count_threshold + grant-ability-to-permanents"). The claim was FALSE
            // and this file's own `Completeness` note said so two lines below — the note
            // carried the exact recipe used here. Precedent: cryptolith_rite.rs.
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
                    filter: EffectFilter::LandsYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: Some(Condition::YouControlNOrMoreWithFilter {
                        count: 6,
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Land),
                            ..Default::default()
                        },
                    }),
                },
            },
            // TODO: see the `Completeness` note — the God tutor is the one blocked clause.
        ],
        completeness: Completeness::partial(
            "Blocked only on multi-card search: Effect::SearchLibrary has no count field \
             (card_definition.rs:1701-1719 — player/filter/reveal/destination/ \
             shuffle_before_placing/also_search_graveyard, no count), so 'search for any number \
             of God cards' is inexpressible. PB-DX27 (2026-08-13) IMPLEMENTED the six-lands \
             static grant this note previously described as merely not-blocked; the stale inline \
             TODO claiming a `count_threshold` gap is deleted. Known simplification carried by \
             the grant: `ManaAbility.any_color` documents itself as defaulting to colourless \
             until interactive colour choice exists (game_object.rs:342-346) — that is a \
             pre-existing engine deviation shared with cryptolith_rite, not a new one here.",
        ),
        ..Default::default()
    }
}
