// Vraska, Betrayal's Sting — {4}{B}{B/P} Legendary Planeswalker — Vraska
// Compleated ({B/P} can be paid with {B} or 2 life. If life was paid, enters
// with two fewer loyalty counters.)
// 0: You draw a card and lose 1 life. Proliferate.
// -2: Target creature becomes a Treasure artifact with "{T}, Sacrifice this
//     artifact: Add one mana of any color" and loses all other card types and abilities.
// -9: If target player has fewer than nine poison counters, they get a number of
//     poison counters equal to the difference.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vraska-betrayals-sting"),
        name: "Vraska, Betrayal's Sting".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            black: 1,
            phyrexian: vec![PhyrexianMana::Single(ManaColor::Black)],
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Vraska"],
        ),
        oracle_text: "Compleated ({B/P} can be paid with {B} or 2 life. If life was paid, this \
                      planeswalker enters with two fewer loyalty counters.)\n0: You draw a card \
                      and lose 1 life. Proliferate.\n\u{2212}2: Target creature becomes a \
                      Treasure artifact with \"{T}, Sacrifice this artifact: Add one mana of any \
                      color\" and loses all other card types and abilities.\n\u{2212}9: If target \
                      player has fewer than nine poison counters, they get a number of poison \
                      counters equal to the difference."
            .to_string(),
        starting_loyalty: Some(6),
        abilities: vec![
            // 0: Draw a card and lose 1 life. Proliferate.
            // NOTE: Compleated (entering with 4 loyalty when life paid) not modeled in DSL.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Zero,
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::LoseLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                    Effect::Proliferate,
                ]),
                targets: vec![],
            },
            // -2: Target creature becomes a Treasure artifact with "{T}, Sacrifice this
            // artifact: Add one mana of any color" and loses all other card types and
            // abilities. PB-AC7: unblocked. Per ruling: "The target of Vraska's second
            // loyalty ability will lose any other subtypes and card types it previously
            // had and will be only a Treasure artifact. It will retain any supertypes
            // it had." — so SetCardTypes (not SetTypeLine) preserves supertypes;
            // LoseAllSubtypes + AddSubtypes(Treasure) clears ALL subtypes (not just
            // creature types) per the ruling. RemoveAllAbilities is listed BEFORE the
            // AddManaAbility grant (CR 613.7 timestamp ordering — within a Sequence,
            // Effect::ApplyContinuousEffect does not advance the timestamp counter, so
            // push order = apply order for same-layer effects) so the granted mana
            // ability survives the "loses all other abilities" removal.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::TypeChange,
                            modification: LayerModification::SetCardTypes(
                                [CardType::Artifact].into_iter().collect(),
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::Indefinite,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::TypeChange,
                            modification: LayerModification::LoseAllSubtypes,
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::Indefinite,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::TypeChange,
                            modification: LayerModification::AddSubtypes(
                                [SubType("Treasure".to_string())].into_iter().collect(),
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::Indefinite,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::RemoveAllAbilities,
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::Indefinite,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddManaAbility(ManaAbility {
                                produces: Default::default(),
                                requires_tap: true,
                                sacrifice_self: true,
                                any_color: true,
                                damage_to_controller: 0,
                                ..Default::default()
                            }),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::Indefinite,
                            condition: None,
                        }),
                    },
                ]),
                targets: vec![TargetRequirement::TargetCreature],
            },
            // -9: If target player has fewer than nine poison counters, they get a
            // number of poison counters equal to the difference.
            // TODO: No "poison counters equal to difference" EffectAmount variant.
            // Needs EffectAmount::PoisonDifference or similar (OOS-AC7-1). Genuine
            // remaining gap — not addressed by PB-AC7.
        ],
        completeness: Completeness::partial(
            "Blocked on (a) the -9: no EffectAmount for 'poison counters equal to the difference \
             from nine', and Effect::AddCounter cannot target a player (effects/mod.rs:2321 \
             handles ResolvedTarget::Object only); (b) Compleated is not modeled — \
             starting_loyalty is fixed at 6, so paying {B/P} with 2 life wrongly yields 6 loyalty \
             instead of 4.",
        ),
        ..Default::default()
    }
}
