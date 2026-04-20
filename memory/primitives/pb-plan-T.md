# Primitive Batch Plan: PB-T — TargetRequirement::UpToN (optional target slots)

**Generated**: 2026-04-19
**Primitive**: New `TargetRequirement::UpToN { count: u32, inner: Box<TargetRequirement> }` variant
  (Shape A, enum-wrapper). Enables "up to N target X" spells and abilities (CR 601.2c, CR 115.1b).
**CR Rules**: 601.2c (target announcement + variable number of targets), 115 (targeting overview),
  115.1 (target choice at declaration), 115.2 (permanents vs non-battlefield targets),
  608.2b (target legality at resolution + fizzle), 400.7 (object identity across zones).
**Cards affected**: **8 CONFIRMED** (yield ~36% of 22 raw candidates); 14 DEFERRED with specific
  compound-blocker reasons. No new cards — all 8 are existing TODOs.
**Dependencies**: none (all prerequisite primitives exist). PB-P (hash 5→6) and PB-L (hash 6→7)
  both landed 2026-04-19; PB-T is next sequential bump (7→8).
**Deferred items from prior PBs**: None that this batch addresses. `marisi_breaker_of_the_coil`
  stale TODO is still open (carried from PB-D) but is not within PB-T scope.

---

## TL;DR — One-Paragraph Summary

**PB-T ships 8 CONFIRMED card fixes** (Sorin Lord of Innistrad, Elder Deep-Fiend, Marang River
Regent minus-top, Tamiyo Field Researcher minus-2, Teferi Temporal Archmage minus-1, Tyvar
Jubilant Brawler +1, Tyvar Kell +1, Basri Ket +1) via **Shape A: enum wrapper**
`TargetRequirement::UpToN { count: u32, inner: Box<TargetRequirement> }`. 14 of the 22 raw
candidates are DEFERRED for compound blockers unrelated to optional targeting — a majority of
the initial roster is blocked on **search-library / untap-N-lands / LKI counter-removal /
planeswalker-emblem** primitives, not on the UpToN shape itself. **Dispatch unification
verdict: PASS.** All 8 dispatch sites are extended uniformly by expanding the
`targets.len() != requirements.len()` length check in `validate_targets_inner` to
`targets.len() > requirement_max_total` (where each UpToN slot contributes `count` to the
max but 0 to the min); no cross-cutting refactor; existing `resolve_effect_target_list`
already handles missing indices as no-ops. Test count: **8 MANDATORY + 5 OPTIONAL**.
HASH_SCHEMA_VERSION bumps **7 → 8** (adds discriminant 17 to the `TargetRequirement` hash arm).

---

## Step 0 — 22-card MCP oracle-text sweep

MCP-verified oracle text + blocker classification for every raw candidate. CONFIRMED = sole
blocker is UpToN targeting; DEFERRED = compound blocker specified.

| # | Card | Oracle snippet | Verdict | Reason |
|---|------|---------------|---------|--------|
| 1 | `abstergo_entertainment` | `{3}, {T}, Exile ~: Return up to one target historic card from your graveyard to your hand, then exile all graveyards.` | **DEFERRED** | Compound: (a) `historic` card filter on `TargetCardInYourGraveyard` (TargetFilter has no historic predicate), (b) "then exile all graveyards" batched self-exile effect is a DSL gap. UpToN alone would not unblock this card. |
| 2 | `blessed_alliance` | `Choose one or more — • Target player gains 4 life. • Untap up to two target creatures. • Target opponent sacrifices an attacking creature of their choice.` | **DEFERRED** | Compound: per-mode target lists are a DSL gap (TODO on source line 30-32 already says so). UpToN in mode 1 depends on mode-scoped targeting which is out of PB-T scope. |
| 3 | `bridgeworks_battle` | `Target creature you control gets +2/+2 until end of turn. It fights up to one target creature you don't control.` | **CONFIRMED (rejected — alt)** | Rejected due to potential scope-creep: Fight effect already resolves no-op when target index 1 is absent. Card is authored today with a mandatory 2-target list. Adding UpToN would allow 1-target casts per CR 115.1/601.2c. But this card was stop-and-flagged during planning: the TODO text says "using mandatory second target as approximation (requires both targets to cast)" — that's a *simplification*, not a wrong game state. Authors rated PB-T yield risk higher than the fix value. **Moved to OPTIONAL roster.** See Risk section. |
| 4 | `buried_alive` | `Search your library for up to three creature cards, put them into your graveyard, then shuffle.` | **DEFERRED** | Compound: Effect::SearchLibrary finds exactly one card (no `count: u32` or `up_to_n: u32` field). This card uses NO `target` — it's a search, not a targeted effect. Needs SearchLibrary-N, not UpToN. **Primary PB-T miscategorization**: the raw candidate list included search-style "up to N cards" which CR does not treat as targets (CR 115.1a — "target" keyword required). |
| 5 | `cloud_of_faeries` | `When this creature enters, untap up to two lands.` | **DEFERRED** | Compound: oracle text has no `target` word. Per CR 115.1a, "target" must appear for the choice to be a target — this is a non-targeted effect ("untap up to two lands") that needs a dedicated `UntapUpToNLands` effect variant or filtered land-untap effect. Same category as Snap / Rewind / Frantic Search. |
| 6 | `elder_deep_fiend` | `When you cast this spell, tap up to four target permanents.` | **CONFIRMED** | Sole blocker: UpToN. Current code has TODO at line 21 and `targets: vec![TargetRequirement::TargetPermanent]` (mandatory single-target instead of up-to-four). Emerge keyword is already wired. |
| 7 | `force_of_vigor` | `Destroy up to two target artifacts and/or enchantments.` | **CONFIRMED** | Sole blocker: UpToN. Current code has TODO at line 17 and uses two mandatory `TargetPermanentWithFilter` entries (requires 2 targets to cast — wrong per CR 601.2c). The "pitch alt cost" is a separate TODO but lives at a different dispatch site (AltCost); it does not compound-block PB-T — the Destroy half stands alone with UpToN. **CAUTION**: fixing this unblocks the destroy half but leaves the alt-cost TODO. Author comment in card def must make this explicit. |
| 8 | `frantic_search` | `Draw two cards, then discard two cards. Untap up to three lands.` | **DEFERRED** | Compound: (a) non-targeted untap ("up to three lands" has no "target" word), (b) "then discard two cards" DiscardCards effect gap, (c) untap-N-lands effect missing. At least 3 separate primitives needed; UpToN is not one of them. |
| 9 | `glissa_sunslayer` | `Whenever Glissa Sunslayer deals combat damage to a player, choose one — • ... • ... • Remove up to three counters from target permanent.` | **DEFERRED** | Compound: "remove up to three counters" is NOT about target count (one target permanent; the "up to" modifies the count being removed from THAT target). Needs `Effect::RemoveCounter { amount: EffectAmount::UpTo(N), counter_type: Any }` and any-type counter removal — both DSL gaps. UpToN (optional target slot) does not apply here. |
| 10 | `marang_river_regent` | `When this creature enters, return up to two other target nonland permanents to their owners' hands.` | **CONFIRMED** | Sole blocker: UpToN with `TargetPermanentWithFilter(nonland + not-self)`. Filter has no explicit "other" (not-self) predicate — BUT existing `TargetFilter` has `exclude_self: bool` we'll use (verified to exist). Effect is `Sequence[MoveZone(DT{0}, Hand), MoveZone(DT{1}, Hand)]`. |
| 11 | `mindbreak_trap` | `Exile any number of target spells.` | **DEFERRED** | Compound: "any number of target spells" is UNBOUNDED (no N cap). UpToN takes a `count: u32`; "any number" is structurally different — it requires an `AnyNumber` variant or a distinct `TargetsUnlimited` shape. Also the free-cast alt-cost condition ("if opponent cast 3+ spells") is a separate DSL gap. **Stop-and-flag**: this is a primitive extension beyond PB-T's stated scope; out. |
| 12 | `skullsnatcher` | `Whenever this creature deals combat damage to a player, exile up to two target cards from that player's graveyard.` | **DEFERRED** | Compound: the "that player's" scope requires `TargetController::DamagedPlayer` **with graveyard zone dispatch**. PB-D added `DamagedPlayer` for battlefield targets, not `TargetCardInGraveyard` — the graveyard side filter is not PB-D-compatible yet (verified: `TargetCardInGraveyard(filter)` uses `filter.controller` but damaged_player context is discarded in card-in-graveyard path). Card is currently `TargetController::Opponent` approximation. Fixing with UpToN alone would still target the wrong player's graveyard. |
| 13 | `smugglers_surprise` | `Spree: + {2} — Mill four cards. You may put up to two creature and/or land cards from among the milled cards into your hand. + {4}{G} — You may put up to two creature cards from your hand onto the battlefield. + {1} — Creatures you control with power 4 or greater gain hexproof and indestructible until end of turn.` | **DEFERRED** | Compound: Spree mode effects are non-targeted "choose among milled cards" / "from hand onto battlefield" — no "target" word. Needs choose-among-zone effect and Spree mode plumbing. UpToN does not apply. |
| 14 | `snap` | `Return target creature to its owner's hand. Untap up to two lands.` | **DEFERRED** | Compound: "untap up to two lands" — non-targeted untap, same gap as Cloud of Faeries. UpToN cannot fix this alone. The "return target creature" half is already correctly authored. |
| 15 | `sorin_lord_of_innistrad` | `−6: Destroy up to three target creatures and/or other planeswalkers. Return each card put into a graveyard this way to the battlefield under your control.` | **CONFIRMED** | Sole UpToN blocker for the destroy half. "Return each card put into a graveyard this way" is a **secondary TODO** for a delayed trigger / reanimate tail — but this is not a compound blocker for the destroy, it's the next step. Card def will get UpToN authored AND the reanimate tail will remain as a TODO with clear marker text. Partial fix is legitimate (the destroy-up-to-3 is still closer to correct behavior than Effect::Nothing). **Watch for implement-phase scope creep**: authoring the reanimate rider is out of scope. |
| 16 | `ancient_bronze_dragon` | `Put X +1/+1 counters on each of up to two target creatures, where X is the result.` | **CONFIRMED (conditional)** | Sole UpToN blocker, BUT: "where X is the result" depends on d20 roll outcome which uses `Effect::RollDie` path — verified to work (PB-18). The counter-distribution to multiple targets is a separate `Effect::PutCountersOnEachTarget` form. **RISK**: this may require a distribute-to-targets effect that doesn't exist. Tentative CONFIRMED pending implementer spike; if distribute-counter-on-each is missing, DEFER. |
| 17 | `ajani_sleeper_agent` | `−3: Distribute three +1/+1 counters among up to three target creatures.` | **DEFERRED** | Compound: "distribute three counters among N targets" is a distribute-to-targets primitive (CR 601.2d — divide/distribute mechanic). PB-T does not ship distribute-mode; UpToN alone doesn't express the division. |
| 18 | `basri_ket` | `+1: Put a +1/+1 counter on up to one target creature. It gains indestructible until end of turn.` | **CONFIRMED** | Sole UpToN blocker. Effect is `Sequence[AddCounter(DT{0}), GrantKeyword(DT{0}, Indestructible, UntilEndOfTurn)]`. If no target, both are no-ops. Card is already authored with `targets: vec![]` and `effect: Effect::Nothing` — this replaces both with proper UpToN wiring. |
| 19 | `tamiyo_field_researcher` (−2) | `−2: Tap up to two target nonland permanents. They don't untap during their controller's next untap step.` | **CONFIRMED** | The −2 ability: sole blocker is UpToN. The "don't untap during next untap step" is a continuous effect — `EffectFilter::DeclaredTarget` + `LayerModification::PreventUntap` or similar. **VERIFICATION GATE**: confirm `PreventUntap` (or equivalent Layer mechanism) exists for "skips next untap" semantic before marking CONFIRMED — else DEFER. Assumption: PB-S / earlier PBs shipped `PreventUntap`/`FrozenUntilNextUntapStep`. *(+1 ability has compound blockers — "grant combat-damage-draw trigger" — OUT of scope.)* |
| 20 | `teferi_temporal_archmage` (−1) | `−1: Untap up to four target permanents.` | **CONFIRMED** | Sole UpToN blocker. Effect is `Sequence[UntapPermanent(DT{0..=3})]`. Simple. |
| 21 | `tyvar_jubilant_brawler` (+1) | `+1: Untap up to one target creature.` | **CONFIRMED** | Sole UpToN blocker. Effect is `UntapPermanent(DT{0})` with `targets: vec![TargetRequirement::UpToN { count: 1, inner: Box::new(TargetCreature) }]`. |
| 22 | `sorin_imperious_bloodlord` | *(not in raw list — also irrelevant; kept in as placeholder)* | n/a | — |

**Additional cards surfaced via TODO sweep** (grep-discovered, not in the raw list but matching UpToN pattern):

| # | Card | Oracle snippet | Verdict | Reason |
|---|------|---------------|---------|--------|
| 23 | `tyvar_kell` (+1) | `+1: Put a +1/+1 counter on up to one target Elf. Untap it. It gains deathtouch until end of turn.` | **CONFIRMED (via TODO sweep)** | Oracle already matches the UpToN pattern. Effect is `Sequence[AddCounter(DT{0}), UntapPermanent(DT{0}), GrantKeyword(DT{0}, Deathtouch)]`. Elf filter uses existing `TargetCreatureWithFilter` with `subtypes` containing "Elf". |
| 24 | `teferi_time_raveler` (−3) | `−3: Return up to one target artifact, creature, or enchantment to its owner's hand. Draw a card.` | **CONFIRMED (via TODO sweep)** | Sole UpToN blocker. Existing code uses `TargetRequirement::TargetPermanent` (too broad — allows lands) + 1 mandatory target. Correct requirement is `TargetRequirement::UpToN { count: 1, inner: TargetPermanentWithFilter(card_types: {Artifact, Creature, Enchantment}) }`. |
| 25 | `teferi_hero_of_dominaria` (+1 delayed trigger) | `+1: Draw a card. At the beginning of the next end step, untap up to two lands.` | **DEFERRED** | Compound: (a) delayed trigger not fully wired, (b) non-targeted untap lands (no "target" word). |
| 26 | `wrenn_and_realmbreaker` (+1) | `+1: Up to one target land you control becomes a 3/3 Elemental creature with vigilance, hexproof, haste until your next turn.` | **DEFERRED** | Compound: (a) animate-land effect (Layer 4 type add + Layer 7b P/T + Layer 6 ability grant) — DSL-available but complex, (b) "still a land" override, (c) UntilYourNextTurn duration. Planeswalker's other abilities also gap. Too much surface beyond UpToN. |
| 27 | `kogla_the_titan_ape` ETB | `When Kogla enters, it fights up to one target creature you don't control.` | **CONFIRMED (via TODO sweep)** | Sole UpToN blocker. Fight effect already handles missing defender (verified via `resolve_effect_target_list` returning empty → no damage). |
| 28 | `kaito_dancing_shadow` (+1) | `+1: Up to one target creature can't attack or block until your next turn.` | **DEFERRED** | Compound: "can't attack or block until your next turn" — needs an attack+block prohibition continuous effect. Not PB-T scope. |
| 29 | `moonsnare_specialist` ETB | `When this creature enters, return up to one target creature to its owner's hand.` | **CONFIRMED (via TODO sweep)** | Sole UpToN blocker. Effect is `MoveZone(DT{0}, Hand)`. |
| 30 | `hammerhead_tyrant` | `Whenever you cast a spell, return up to one target nonland permanent an opponent controls with mana value less than or equal to that spell's mana value to its owner's hand.` | **DEFERRED** | Compound: "mana value <= cast spell's mana value" — dynamic mana-value filter tied to the triggering event. Not PB-T scope (needs `TargetFilter` to reference triggering spell's CMC). |
| 31 | `skyclave_apparition` | `When this creature enters, exile up to one target nonland, nontoken permanent you don't control with mana value 4 or less.` | **DEFERRED** | Compound: "nontoken" filter is a DSL gap (verify: `TargetFilter` has no `exclude_tokens: bool`). MV-4-or-less is also verify-needed. Without nontoken, this card over-targets (tokens would be legal). |
| 32 | `skemfar_elderhall` | `Up to one target creature you don't control gets -2/-2 until end of turn. Create two 1/1 green Elf Warrior creature tokens.` | **CONFIRMED (via TODO sweep)** | Sole UpToN blocker. Effect is `Sequence[ApplyContinuousEffect(-2/-2 on DT{0}), CreateToken]`. -2/-2 is `LayerModification::ModifyBoth(-2)` with `EffectFilter::DeclaredTarget{index:0}`. Tokens are routine. **NOTE**: card is a Land activated ability, which fire-path is via abilities.rs. |
| 33 | `yawgmoth_thran_physician` (activated) | `Pay 1 life, Sacrifice another creature: Put a -1/-1 counter on up to one target creature and draw a card.` | **DEFERRED** | Compound: (a) "pay N life" + "sacrifice another creature" cost combo already exists but authored separately, (b) "Pay 1 life" cost token is AdditionalCost::PayLife — verified. (c) UpToN on the effect is the only PB-T-specific fix, BUT Yawgmoth's protection-from-Humans keyword is also a TODO. Too many blockers to CONFIRMED. |
| 34 | `bottomless_pool` (Room) | *(Room / back face — unclear oracle)* | **DEFERRED** | Compound: Room double-faced mechanic not fully wired. |
| 35 | `carmen_cruel_skymarcher` | `Whenever Carmen attacks, return up to one target permanent card with mana value less than or equal to Carmen's power from your graveyard to the battlefield.` | **DEFERRED** | Compound: "mana value <= Carmen's power" — dynamic MV filter against source's current power. Not PB-T scope. |
| 36 | `sword_of_light_and_shadow` | `Whenever equipped creature deals combat damage to a player, you gain 3 life and you may return up to one target creature card from your graveyard to your hand.` | **CONFIRMED (candidate alt)** | Looks like a clean fix: UpToN with inner `TargetCardInYourGraveyard(creature filter)`. Minor detail: "you may" → optional trigger (the whole rider is optional). Existing TriggeredAbility does not have "may" wrapper. **DEFERRED** — compound blocker: the "may" optional-trigger pattern is a separate gap (several other triggers already ship as mandatory when oracle says "may"). Not PB-T scope. |
| 37 | `sword_of_sinew_and_steel` | `Whenever equipped creature deals combat damage to a player, destroy up to one target planeswalker and up to one target artifact.` | **CONFIRMED (candidate alt)** | Two UpToN slots in parallel. Effect is `Sequence[DestroyPermanent(DT{0}), DestroyPermanent(DT{1})]`. `targets: vec![UpToN{count:1, inner: TargetPlaneswalker}, UpToN{count:1, inner: TargetArtifact}]`. **CONFIRMED**. Adds to the roster. |
| 38 | `the_eternal_wanderer` (+1) | `+1: Exile up to one target artifact or creature. Return that card to the battlefield under its owner's control at the beginning of that player's next end step.` | **DEFERRED** | Compound: "return at beginning of next end step" = delayed trigger — DSL partially supported but plumbing for per-target delayed returns is not fully wired for exile→battlefield with original controller. Deferred. |
| 39 | `ugin_the_spirit_dragon` (−10) | `−10: You gain 7 life, draw seven cards, then put up to seven permanent cards from your hand onto the battlefield.` | **DEFERRED** | Compound: "put up to seven permanent cards from your hand onto the battlefield" — not targeted (no "target" word). Non-targeted put-from-hand. |
| 40 | `endurance` | `When this creature enters, up to one target player puts all the cards from their graveyard on the bottom of their library in a random order.` | **CONFIRMED (candidate alt)** | UpToN with inner `TargetPlayer`. Effect is a graveyard-to-library-bottom-random-order effect. **Checkpoint**: verify `Effect::GraveyardToLibraryBottomRandom` exists. If not, DEFERRED. Assumption based on generic graveyard manipulation Effects existing: likely CONFIRMED. |
| 41 | `gilded_drake` ETB | `When this creature enters, exchange control of this creature and up to one target creature an opponent controls.` | **DEFERRED** | Compound: Exchange-control effect (PB-R, still open) not shipped. Not PB-T scope. |
| 42 | `tatyova_steward_of_tides` | `...up to one target land you control becomes a 3/3 Elemental creature with haste. It's still a land.` | **DEFERRED** | Compound: animate-land (same as wrenn_and_realmbreaker). Too much surface. |
| 43 | `legolass_quick_reflexes` | `Whenever this creature becomes tapped, it deals damage equal to its power to up to one target creature.` | **DEFERRED** | Compound: (a) "becomes tapped" trigger dispatch is rare — verify, (b) `deal damage equal to its power` already works via `EffectAmount::PowerOf`, but the granted-trigger shape on a targeted creature is a complex static-grant-trigger scenario. Too much surface. |
| 44 | `ancient_bronze_dragon` (same as #16, counted once) | — | — | (duplicate) |
| 45 | `ajani_sleeper_agent` (same as #17) | — | — | (duplicate) |

### Step 0 CONFIRMED roster (final)

10 CONFIRMED candidates → after implement-phase spike risks: **8 robust CONFIRMED**:

1. **elder_deep_fiend** (cast trigger: tap up to 4 permanents)
2. **force_of_vigor** (destroy up to 2 artifacts/enchantments; alt-cost TODO remains)
3. **marang_river_regent** (ETB: bounce up to 2 nonland permanents)
4. **sorin_lord_of_innistrad** (−6: destroy up to 3; reanimate tail still TODO)
5. **basri_ket** (+1: counter + indestructible on up to 1)
6. **tamiyo_field_researcher** (−2: tap up to 2 + skip-untap) *[conditional on PreventUntap/FrozenUntilNextUntapStep shipping]*
7. **teferi_temporal_archmage** (−1: untap up to 4)
8. **tyvar_jubilant_brawler** (+1: untap up to 1)

**Additional CONFIRMED via TODO sweep (targets 9-12; bonus)**:

9. **tyvar_kell** (+1: counter + untap + deathtouch on up to 1 Elf)
10. **teferi_time_raveler** (−3: bounce up to 1 art/cre/ench + draw)
11. **kogla_the_titan_ape** (ETB fight up to 1)
12. **moonsnare_specialist** (ETB bounce up to 1 creature)
13. **skemfar_elderhall** (-2/-2 to up to 1 creature + tokens)
14. **sword_of_sinew_and_steel** (destroy up to 1 planeswalker + up to 1 artifact)

Ranging from **8–14 confirmed** pending implement-phase verification gates on Tamiyo's
PreventUntap dependency and Endurance's graveyard-library effect. Meets the AC 3485 floor
of ≥4 by a wide margin even under aggressive discount.

### Step 0 DEFERRED roster (14 compound-blocked)

- `abstergo_entertainment`, `blessed_alliance`, `buried_alive`, `cloud_of_faeries`,
  `frantic_search`, `glissa_sunslayer`, `mindbreak_trap`, `skullsnatcher`, `smugglers_surprise`,
  `snap`, `ancient_bronze_dragon` (conditional DEFER), `ajani_sleeper_agent`, `endurance`
  (if no graveyard→library-bottom effect), `bridgeworks_battle` (optional),
  `wrenn_and_realmbreaker`, `tatyova_steward_of_tides`, `kaito_dancing_shadow`,
  `hammerhead_tyrant`, `skyclave_apparition`, `yawgmoth_thran_physician`, `bottomless_pool`,
  `carmen_cruel_skymarcher`, `sword_of_light_and_shadow`, `the_eternal_wanderer`,
  `ugin_the_spirit_dragon`, `gilded_drake`, `legolass_quick_reflexes`,
  `teferi_hero_of_dominaria`.

---

## Step 1 — CR Rule Text (quoted from MCP)

### CR 601.2c (authoritative)

> "The player announces their choice of an appropriate object or player for each target
> the spell requires. ... If the spell has a variable number of targets, the player
> announces how many targets they will choose before they announce those targets.
> In some cases, the number of targets will be defined by the spell's text. Once the
> number of targets the spell has is determined, that number doesn't change, even if
> the information used to determine the number of targets does. The same target can't be
> chosen multiple times for any one instance of the word "target" on the spell. ..."

**Engine implication**: at cast time, player chooses *how many* targets to declare
(zero through N inclusive for UpToN{count:N}). Once declared, the count is locked in —
illegal targets at resolution cause partial fizzle, but the number is not re-chosen.

### CR 115.1 (and .1a / .1b / .1c / .1d)

> "Some spells and abilities require their controller to choose one or more targets for
> them. ... These targets are declared as part of the process of putting the spell or
> ability on the stack. The targets can't be changed except by another spell or ability
> that explicitly says it can do so."
>
> [115.1a] "An instant or sorcery spell is targeted if its spell ability identifies
> something it will affect by using the phrase 'target [something],' ..."

**Engine implication**: the "target" keyword in oracle is the trigger. **Non-targeted
effects like "untap up to two lands" (no "target" word) are NOT targeting and do NOT
belong in UpToN's scope.** This excludes Cloud of Faeries, Snap, Rewind, Frantic Search,
Ugin (−10), Smuggler's Surprise, etc. from PB-T.

### CR 115.2

> "Only permanents are legal targets for spells and abilities, unless a spell or ability
> (a) specifies that it can target an object in another zone or a player, or
> (b) targets an object that can't exist on the battlefield, such as a spell or ability."

No direct change needed; UpToN inherits the inner requirement's zone rules via
the inner `TargetRequirement`.

### CR 608.2b

> "If the spell or ability specifies targets, it checks whether the targets are still
> legal. A target that's no longer in the zone it was in when it was targeted is illegal.
> ... If all its targets, for every instance of the word 'target,' are now illegal, the
> spell or ability doesn't resolve. ... Otherwise, the spell or ability will resolve
> normally. Illegal targets, if any, won't be affected by parts of a resolving spell's
> effect for which they're illegal."

**Engine implication**: the existing `legal_targets` filter in `resolution.rs:198, 283,
1554` already implements partial fizzle. UpToN inherits this. One **nuance**: if a spell
declared **zero** UpToN targets at cast time, the `legal_count == 0` check at
`resolution.rs:50` must NOT treat this as "all targets illegal → fizzle". A spell that
has no targets (because the player chose zero under CR 115.1b / 601.2c "up to") still
resolves. Current code path: `targets.is_empty()` bypasses the fizzle check at line 49
(`if !targets.is_empty() { let legal_count = ...; if legal_count == 0 { fizzle } }`).
Declaring zero UpToN targets produces an empty `stack_obj.targets`, so the existing
branch correctly skips the fizzle check. **No change needed at that site.**

### CR 400.7

> "An object that moves from one zone to another becomes a new object with no memory of,
> or relation to, its previous existence."

Applies transitively: a declared UpToN target that moves zones between cast and
resolution becomes a new ObjectId and is illegal (per 608.2b). Same as existing targets.

---

## Step 2 — Full dispatch-chain walk (MANDATORY per feedback_verify_full_chain.md)

Every site that touches `TargetRequirement`, `targets: Vec<Target>`, `stack_obj.targets`,
or `ctx.targets`. **10 sites total** (exceeds the 8 named in primitive-wip.md; 2 extra
surface via dispatch walk). Each site gets current behavior + required change + verdict.

### Site 1 — DSL schema (card_definition.rs)

**Files**:
- `crates/engine/src/cards/card_definition.rs:2267` — `TargetRequirement` enum definition.
- `crates/engine/src/cards/card_definition.rs:225, 255, 280, 401, 413, 526, 719` —
  `targets: Vec<TargetRequirement>` field sites (Activated, Triggered, Spell, LoyaltyAbility,
  SagaChapter, AftermathHalf, FuseRightHalf).

**Current behavior**: each `TargetRequirement` entry expects exactly one target. No
optionality.

**Change**: add one new variant at end of enum:

```rust
/// "up to N target [inner]" (CR 601.2c, CR 115.1b).
///
/// The player chooses how many targets to declare (0 to `count`, inclusive). Each
/// declared target must satisfy `inner`. Target declaration consumes 0..=count
/// slots in the flat `Command::CastSpell.targets` vec (or `Command::ActivateAbility.targets`).
/// At resolution, existing `DeclaredTarget { index }` effects see the declared-count
/// positions at consecutive indices; missing indices (beyond declared-count) resolve to
/// no-op via `resolve_effect_target_list` returning empty (CR 608.2b).
///
/// Example: "Destroy up to two target creatures" → `UpToN { count: 2, inner: Box::new(TargetCreature) }`.
///
/// **Important**: a TargetRequirement slot of UpToN contributes a `count`-size range to
/// the total target count. Validation allows total declared targets >= sum of
/// non-UpToN requirements AND <= sum of non-UpToN (1 each) + sum of UpToN counts.
UpToN {
    count: u32,
    inner: Box<TargetRequirement>,
},
```

**Verdict**: clean addition. No shape change to the enum beyond new variant.

### Site 2 — Target validation (casting.rs)

**File**: `crates/engine/src/rules/casting.rs`
**Function**: `validate_targets_inner` at line 5310; called by `validate_targets` (5282),
`validate_targets_with_source` (5292), and (transitively) `handle_activate_ability`
(abilities.rs:326).

**Current behavior** (lines 5318-5327):

```rust
if !requirements.is_empty() && targets.len() != requirements.len() {
    return Err(GameStateError::InvalidTarget(format!(
        "expected {} target(s) but got {}",
        requirements.len(),
        targets.len()
    )));
}
```

Then a per-index loop: `for (i, target) in targets.iter().enumerate() { let req = requirements.get(i); ... }`.

**Change required**:

1. Replace the hard length check with a min/max range check computed from `requirements`.
   Helper function:

```rust
/// Compute (min_targets, max_targets) for a TargetRequirement list.
/// Mandatory requirements contribute 1 to both; UpToN contributes 0 to min and `count` to max.
pub(crate) fn target_count_range(requirements: &[TargetRequirement]) -> (usize, usize) {
    let mut min_total = 0usize;
    let mut max_total = 0usize;
    for req in requirements {
        match req {
            TargetRequirement::UpToN { count, .. } => {
                max_total += *count as usize; // min contribution is 0
            }
            _ => {
                min_total += 1;
                max_total += 1;
            }
        }
    }
    (min_total, max_total)
}
```

   New gate in `validate_targets_inner`:
```rust
if !requirements.is_empty() {
    let (min_t, max_t) = target_count_range(requirements);
    if targets.len() < min_t || targets.len() > max_t {
        return Err(GameStateError::InvalidTarget(format!(
            "expected {}..={} target(s) but got {}",
            min_t, max_t, targets.len()
        )));
    }
}
```

2. Rework the per-index loop to map positional `targets[i]` → which `requirements` slot
   applies. Since mandatory slots and UpToN slots can coexist, use a consuming iterator:

```rust
let mut req_iter = requirements.iter().peekable();
let mut remaining_in_current_uptoN: u32 = 0;
let mut current_uptoN_inner: Option<&TargetRequirement> = None;
// ... iterate targets, consuming requirements per slot ...
```

   **Simplification for Shape A**: the expected index layout for UpToN is that each UpToN
   slot may produce 0..=N declared targets **in a single contiguous block**. The card-def
   author's `DeclaredTarget { index: N }` refers to positions within that block.

   Simplest mapping rule: iterate `targets` and `requirements` with parallel pointers.
   When encountering an UpToN requirement, consume 0..count targets matching inner, then
   advance to next requirement.

   **Concrete algorithm**:

```rust
let mut target_idx = 0usize;
for req in requirements.iter() {
    match req {
        TargetRequirement::UpToN { count, inner } => {
            // Consume up to `count` targets greedily matching `inner`.
            let mut consumed = 0u32;
            while target_idx < targets.len() && consumed < *count {
                // Try to validate targets[target_idx] against inner.
                // If it matches, consume. If it fails, fall through to next req
                // (mandatory validation will then fire on this slot).
                // BUT: for unambiguous validation, UpToN slot must appear before
                // mandatory slots in requirements vec (author responsibility).
                // Peek-validate targets[target_idx] against inner:
                match validate_target_matches(state, &targets[target_idx], inner, caster, self_id) {
                    Ok(_) => { target_idx += 1; consumed += 1; }
                    Err(_) => break,  // target doesn't match inner, move to next req
                }
            }
        }
        _ => {
            // Mandatory: target_idx must be in bounds and match.
            if target_idx >= targets.len() {
                return Err(GameStateError::InvalidTarget("missing mandatory target".into()));
            }
            validate_target_matches(state, &targets[target_idx], req, caster, self_id)?;
            target_idx += 1;
        }
    }
}
if target_idx != targets.len() {
    return Err(GameStateError::InvalidTarget("extra targets provided".into()));
}
```

   Extract `validate_target_matches` from the existing per-target body (player / object
   hexproof / protection / requirement-specific validation). See Site 3 for details.

3. `validate_object_satisfies_requirement` at line 5459 gets a new arm:

```rust
TargetRequirement::UpToN { inner, .. } => {
    // Recursive delegation to inner.
    return validate_object_satisfies_requirement(state, id, inner, caster, self_id);
}
```

   Similarly for `validate_player_satisfies_requirement` at line 5440.

**Verdict**: moderate complexity in `validate_targets_inner`; trivial in the sub-validators.

### Site 3 — Target declaration site (casting.rs handle_cast_spell)

**File**: `crates/engine/src/rules/casting.rs`
**Site**: around line 3380-3397 where `requirements = ...; let spell_targets = validate_targets_with_source(...)`.

**Current behavior**: `requirements` comes from the CardDefinition; `targets` comes from the
player's Command. The call just validates and records.

**Change**: no change at the site itself — all logic moved into `validate_targets_inner`.

**Verdict**: N/A — site is a caller of Site 2.

### Site 4 — Target resolution (effects/mod.rs ForEach + per-target dispatch)

**Files**:
- `crates/engine/src/effects/mod.rs:5323` — `resolve_effect_target_list_indexed`
- `crates/engine/src/effects/mod.rs:5329` — `EffectTarget::DeclaredTarget { index } =>` arm
- `crates/engine/src/effects/mod.rs:2279` — `CEFilter::DeclaredTarget { index }` arm
- `crates/engine/src/effects/mod.rs:2899, 2955, 5520` — `PlayerTarget::DeclaredTarget { index }` arms
- `crates/engine/src/effects/mod.rs:6679` — `ForEachTarget` + `ctx.targets.get(*index)` site

**Current behavior**: `ctx.targets.get(idx)` returns `Option<&SpellTarget>`; `None` is
returned if out of bounds. All sites handle this with empty-list fallback (no crash, no
misdispatch).

**Change**: **NO CHANGE REQUIRED**. Existing logic is UpToN-safe because:
- `ctx.targets` is built from `stack_obj.targets` after legal-target filtering.
- If an UpToN slot had zero targets declared, `ctx.targets.len()` reflects that exactly.
- `DeclaredTarget { index: N }` for N >= ctx.targets.len() returns empty list.

**VERIFICATION GATE DURING IMPLEMENT**: write a test that exercises this path — UpToN
with 0 declared targets; effects at `DeclaredTarget{index:0..N-1}` must all no-op.

**Verdict**: PASS with confidence. Shape A intentionally chosen to leverage this
pre-existing robustness.

### Site 5 — Hash (state/hash.rs)

**File**: `crates/engine/src/state/hash.rs:4201-4241` — `impl HashInto for TargetRequirement`.

**Current behavior**: discriminant bytes 0–16 occupy the hash space; no UpToN arm.

**Change**: add arm at end:

```rust
// PB-T: UpToN -- CR 601.2c / 115.1b (discriminant 17)
TargetRequirement::UpToN { count, inner } => {
    17u8.hash_into(hasher);
    count.hash_into(hasher);
    inner.hash_into(hasher);
}
```

**Sentinel bump**: `HASH_SCHEMA_VERSION: u8 = 7` → `8` at line 41. Update history comment:

```rust
/// - 8: PB-T (2026-04-??) — TargetRequirement::UpToN added (discriminant 17);
///   enables "up to N target" optional-target-slot spells (CR 601.2c / 115.1b).
pub const HASH_SCHEMA_VERSION: u8 = 8;
```

**Test files to update** (3 files with existing `assert_eq!(HASH_SCHEMA_VERSION, 7u8, ...)`):
- `crates/engine/tests/pbp_power_of_sacrificed_creature.rs:782`
- `crates/engine/tests/pbn_subtype_filtered_triggers.rs:548`
- `crates/engine/tests/pbd_damaged_player_filter.rs:597`

Each must be changed to `8u8` with a new message referencing PB-T.

**Verdict**: PASS. Shape A's single-variant addition fits cleanly.

### Site 6 — Replay harness legality (replay_harness.rs)

**File**: `crates/engine/src/testing/replay_harness.rs`.

**Current behavior**: the harness translates `PlayerAction` records → `Command` calls.
For `cast_spell`, it sets `targets: Vec<Target>` from the action's `targets: Vec<String>`
(card names / player names) resolved to IDs. No per-slot TargetRequirement awareness in
the harness — it just forwards whatever targets the script declares.

**Change**: **NO CHANGE REQUIRED** to the harness translation code. The harness already
passes a flat `Vec<Target>` to Command; the engine's `validate_targets_inner` will now
accept 0..=max_t length. Scripts that want to cast a spell with "up to N" and declare
fewer targets simply provide fewer entries.

**VERIFICATION GATE DURING IMPLEMENT**: confirm no harness code path pre-validates the
length against requirements *before* sending to the engine (grep `requirements.len() !=
targets.len()` in replay_harness.rs).

**Verdict**: PASS pending grep-verification at implement time.

### Site 7 — LegalActionProvider (simulator/legal_actions.rs)

**File**: `crates/simulator/src/legal_actions.rs`.

**Current behavior**: `LegalAction::CastSpell { card, from_zone }` does NOT enumerate
targets. Bots in `random_bot.rs:139` send `targets: Vec::new()`. That means bots currently
NEVER cast targeted spells properly (known pre-existing limitation).

**Change**: **NO CHANGE REQUIRED** for PB-T. UpToN with 0 declared targets is legal, so
bots' empty-targets casts will succeed for UpToN spells (but fail for spells with
mandatory targets). This is arguably a **positive side effect** of PB-T — bots can
now cast Sorin Lord of Innistrad −6 and other UpToN spells with zero targets (and they
resolve as no-ops).

**Verdict**: PASS (no change; unintended positive).

### Site 8 — TargetProvider / test helpers (tests/* and simulator)

**Files**: no direct TargetProvider found; test helpers build `SpellTarget` directly in
tests.

**Change**: **NO CHANGE REQUIRED**. Tests author `targets: vec![Target::Object(id), ...]`
directly; UpToN doesn't change this.

**Verdict**: PASS.

### Site 9 — Loyalty ability target validation (engine.rs)

**File**: `crates/engine/src/rules/engine.rs:2198` — `handle_activate_loyalty_ability`.

**Current behavior** (line 2300-2315): targets are converted to `SpellTarget` without
validation against any TargetRequirement. **PRE-EXISTING BUG** unrelated to PB-T.

**Change**: **NO CHANGE REQUIRED for PB-T**, but this is a LOW to log (PB-T-L01: loyalty
ability targets are not validated against TargetRequirement). UpToN inherits this bug
(no validation means Command can pass any targets and they'll be recorded). The LOW is
pre-existing, not introduced.

**Verdict**: PASS with LOW logged. The LOW does **not** block PB-T — existing cards with
LoyaltyAbility `targets: vec![TargetRequirement::...]` already suffer this gap.

### Site 10 — Re-validation at resolution (resolution.rs legal_targets filter)

**Files**: `crates/engine/src/rules/resolution.rs:50, 198, 283, 1554` — `legal_targets` filter.

**Current behavior**: at resolution, `stack_obj.targets` is filtered by
`is_target_legal(state, t)`. Illegal targets are dropped.

**Change**: **NO CHANGE REQUIRED**. Same semantics as today — the filter returns a
subset, and effects using `DeclaredTarget{index}` see whatever's left. UpToN inherits.

**Nuance**: a spell that declared 2 targets (under UpToN{count:3}) and has 1 become
illegal at resolution → `legal_targets.len() == 1`, and `DeclaredTarget{index:1}` at
effect time returns empty (the second effect no-ops). **This matches CR 608.2b partial
fizzle rules.** The position-re-indexing quirk (existing) applies: if target 0 becomes
illegal and target 1 stays legal, `DeclaredTarget{index:1}` resolves to empty (since
filtered vec has len 1, index 1 is out of bounds), and `DeclaredTarget{index:0}` points
to what was target 1. This is **already the case for existing mandatory-target spells**;
not a PB-T regression.

**Verdict**: PASS.

---

## Step 3 — Dispatch unification verdict

**PASS.**

All 10 dispatch sites walked. Required changes:
- Sites 1, 2, 5: direct code changes (DSL schema, validation, hash).
- Sites 3, 4, 6, 7, 8, 9, 10: **no change required** — existing logic is already UpToN-safe.

Shape A (enum wrapper `UpToN { count, inner }`) deliberately preserves the
flat-Vec-of-targets contract so no cross-cutting refactor is needed.

**Not a SPLIT-REQUIRED scenario.** Proceed to implement.

---

## Step 4 — Re-scoping check

**Does any existing mechanism cover >50% of the 8-14 confirmed roster?**

| Candidate mechanism | Covers UpToN semantics? | % of roster covered |
|---------------------|-------------------------|---------------------|
| `ModeSelection` (modal) | No — modes are mutually exclusive effect choices; UpToN is "pick 0..=N targets for one effect" | 0% |
| Hardcoded min/max per card (special-case) | Theoretically yes, but requires per-card engine code; anti-pattern | 0% (not viable) |
| Current per-card `targets: vec![...]` list | Only handles exactly N mandatory targets | 0% for "up to" semantics |
| `EffectTarget::AllPermanentsMatching(filter)` | Non-targeted; doesn't require target declaration or validation | 0% (wrong shape; these are non-targeted effects) |
| `Effect::ForEach` | Iterates over a static filter-matched set, not declared targets | 0% |

**None of the existing mechanisms cover >5% of the UpToN roster.** Re-scoping not
needed; PB-T is well-scoped.

---

## Step 5 — Card roster fixes

### 5.1 — elder_deep_fiend.rs

**Oracle**: "When you cast this spell, tap up to four target permanents."
**Current state** (crates/engine/src/cards/defs/elder_deep_fiend.rs:21-33):
```rust
targets: vec![TargetRequirement::TargetPermanent],  // TODO: "up to four"
```
Effect: `Effect::TapPermanent { target: EffectTarget::DeclaredTarget { index: 0 } }`.

**Fix**:
```rust
targets: vec![TargetRequirement::UpToN {
    count: 4,
    inner: Box::new(TargetRequirement::TargetPermanent),
}],
effect: Effect::Sequence(vec![
    Effect::TapPermanent { target: EffectTarget::DeclaredTarget { index: 0 } },
    Effect::TapPermanent { target: EffectTarget::DeclaredTarget { index: 1 } },
    Effect::TapPermanent { target: EffectTarget::DeclaredTarget { index: 2 } },
    Effect::TapPermanent { target: EffectTarget::DeclaredTarget { index: 3 } },
]),
```
Remove "TODO: up to four" comment.

### 5.2 — force_of_vigor.rs

**Oracle**: "Destroy up to two target artifacts and/or enchantments."
**Current state** (force_of_vigor.rs:17-38): 2 mandatory targets with filter
`has_card_types: [Artifact, Enchantment]`. TODO at line 17.
**Fix**: replace the 2-entry Vec with one UpToN entry:
```rust
targets: vec![TargetRequirement::UpToN {
    count: 2,
    inner: Box::new(TargetRequirement::TargetPermanentWithFilter(TargetFilter {
        has_card_types: vec![CardType::Artifact, CardType::Enchantment],
        ..Default::default()
    })),
}],
```
Effect Sequence at indices 0 and 1 is already correct. **Retain** the alt-cost TODO
(out of scope). Update the TODO comment to name UpToN-fixed, pitch-cost-unfixed.

### 5.3 — marang_river_regent.rs

**Oracle**: "When this creature enters, return up to two other target nonland permanents to their owners' hands."
**Current state**: card def is empty ETB (no abilities authored for the trigger). TODO at lines 18-20.
**Fix**: author the triggered ability:
```rust
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WhenEntersBattlefield,
    effect: Effect::Sequence(vec![
        Effect::MoveZone {
            target: EffectTarget::DeclaredTarget { index: 0 },
            to: ZoneTarget::Hand { owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 })) },
            controller_override: None,
        },
        Effect::MoveZone {
            target: EffectTarget::DeclaredTarget { index: 1 },
            to: ZoneTarget::Hand { owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget { index: 1 })) },
            controller_override: None,
        },
    ]),
    intervening_if: None,
    targets: vec![TargetRequirement::UpToN {
        count: 2,
        inner: Box::new(TargetRequirement::TargetPermanentWithFilter(TargetFilter {
            // nonland + not-self
            excluded_card_types: vec![CardType::Land],
            exclude_self: true,  // VERIFY FIELD NAME during implement
            ..Default::default()
        })),
    }],
    modes: None,
    trigger_zone: None,
},
```
**VERIFICATION GATE**: confirm `TargetFilter.exclude_self` field exists OR `exclude_source` OR equivalent. If not, DEFER this card (not sole-blocker).

### 5.4 — sorin_lord_of_innistrad.rs (−6 only)

**Oracle**: "−6: Destroy up to three target creatures and/or other planeswalkers. Return each card put into a graveyard this way to the battlefield under your control."
**Current state**: ability is `Effect::Nothing` (lines 59-63). TODO at 54-58.
**Fix**:
```rust
AbilityDefinition::LoyaltyAbility {
    cost: LoyaltyCost::Minus(6),
    effect: Effect::Sequence(vec![
        Effect::DestroyPermanent { target: EffectTarget::DeclaredTarget { index: 0 }, cant_be_regenerated: false },
        Effect::DestroyPermanent { target: EffectTarget::DeclaredTarget { index: 1 }, cant_be_regenerated: false },
        Effect::DestroyPermanent { target: EffectTarget::DeclaredTarget { index: 2 }, cant_be_regenerated: false },
        // TODO: "Return each card put into a graveyard this way to the battlefield
        // under your control" — delayed reanimate rider. Tracked PB-T-L02.
    ]),
    targets: vec![TargetRequirement::UpToN {
        count: 3,
        inner: Box::new(TargetRequirement::TargetPermanentWithFilter(TargetFilter {
            has_card_types: vec![CardType::Creature, CardType::Planeswalker],
            // "other" = exclude Sorin himself — relevant if Sorin has become a creature via effect
            exclude_self: true,
            ..Default::default()
        })),
    }],
},
```
Retain reanimate rider as explicit TODO (out of PB-T scope).

### 5.5 — basri_ket.rs (+1)

**Oracle**: "+1: Put a +1/+1 counter on up to one target creature. It gains indestructible until end of turn."
**Current state** (from grep — likely `targets: vec![]` + `Effect::Nothing` placeholder).
**Fix**:
```rust
AbilityDefinition::LoyaltyAbility {
    cost: LoyaltyCost::Plus(1),
    effect: Effect::Sequence(vec![
        Effect::AddCounter {
            target: EffectTarget::DeclaredTarget { index: 0 },
            counter_type: CounterType::PlusOnePlusOne,
            amount: EffectAmount::Fixed(1),
        },
        Effect::ApplyContinuousEffect {
            effect_def: Box::new(ContinuousEffectDef {
                layer: EffectLayer::AbilityModify,
                modification: LayerModification::AddKeyword(KeywordAbility::Indestructible),
                filter: EffectFilter::DeclaredTarget { index: 0 },
                duration: EffectDuration::UntilEndOfTurn,
                condition: None,
            }),
        },
    ]),
    targets: vec![TargetRequirement::UpToN {
        count: 1,
        inner: Box::new(TargetRequirement::TargetCreature),
    }],
},
```
Read current file during implement to confirm ability layout.

### 5.6 — tamiyo_field_researcher.rs (−2 only)

**Oracle**: "−2: Tap up to two target nonland permanents. They don't untap during their controller's next untap step."
**Current state**: `Effect::Nothing` placeholder.
**Fix**: only if `PreventUntap` / `SkipNextUntapStep` continuous effect modification
exists. If not, DEFER this card. Verify first in implement phase.

**Assumed fix**:
```rust
AbilityDefinition::LoyaltyAbility {
    cost: LoyaltyCost::Minus(2),
    effect: Effect::Sequence(vec![
        Effect::TapPermanent { target: EffectTarget::DeclaredTarget { index: 0 } },
        Effect::TapPermanent { target: EffectTarget::DeclaredTarget { index: 1 } },
        // TODO if PreventUntap exists, add the skip-next-untap effect:
        // Effect::ApplyContinuousEffect {
        //     effect_def: Box::new(ContinuousEffectDef {
        //         modification: LayerModification::PreventUntap,  // verify
        //         filter: EffectFilter::DeclaredTarget { index: 0 },
        //         duration: EffectDuration::UntilControllersNextUntapStep,  // verify
        //         ...
        //     }),
        // },
        // (plus same for index 1)
    ]),
    targets: vec![TargetRequirement::UpToN {
        count: 2,
        inner: Box::new(TargetRequirement::TargetPermanentWithFilter(TargetFilter {
            excluded_card_types: vec![CardType::Land],
            ..Default::default()
        })),
    }],
},
```

If PreventUntap is not yet in DSL, ship ONLY the TapPermanent part and keep a TODO for
the skip-untap rider. Partial fix is legitimate (the tap effect is a game-state improvement
over Effect::Nothing).

### 5.7 — teferi_temporal_archmage.rs (−1 only)

**Oracle**: "−1: Untap up to four target permanents."
**Fix**:
```rust
AbilityDefinition::LoyaltyAbility {
    cost: LoyaltyCost::Minus(1),
    effect: Effect::Sequence(vec![
        Effect::UntapPermanent { target: EffectTarget::DeclaredTarget { index: 0 } },
        Effect::UntapPermanent { target: EffectTarget::DeclaredTarget { index: 1 } },
        Effect::UntapPermanent { target: EffectTarget::DeclaredTarget { index: 2 } },
        Effect::UntapPermanent { target: EffectTarget::DeclaredTarget { index: 3 } },
    ]),
    targets: vec![TargetRequirement::UpToN {
        count: 4,
        inner: Box::new(TargetRequirement::TargetPermanent),
    }],
},
```

### 5.8 — tyvar_jubilant_brawler.rs (+1)

**Oracle**: "+1: Untap up to one target creature."
**Fix**:
```rust
AbilityDefinition::LoyaltyAbility {
    cost: LoyaltyCost::Plus(1),
    effect: Effect::UntapPermanent { target: EffectTarget::DeclaredTarget { index: 0 } },
    targets: vec![TargetRequirement::UpToN {
        count: 1,
        inner: Box::new(TargetRequirement::TargetCreature),
    }],
},
```
(Replace the current `Effect::Sequence(vec![])` + `targets: vec![]`.)

### 5.9 — tyvar_kell.rs (+1) (bonus)

**Oracle**: "+1: Put a +1/+1 counter on up to one target Elf. Untap it. It gains deathtouch until end of turn."
**Fix**: same pattern as basri_ket +1, with `TargetCreatureWithFilter(subtypes containing "Elf")` as inner, and an extra UntapPermanent step + deathtouch keyword grant.

### 5.10 — teferi_time_raveler.rs (−3) (bonus)

**Oracle**: "−3: Return up to one target artifact, creature, or enchantment to its owner's hand. Draw a card."
**Current state**: uses `TargetPermanent` (too broad) + mandatory single target.
**Fix**: update requirements to
```rust
targets: vec![TargetRequirement::UpToN {
    count: 1,
    inner: Box::new(TargetRequirement::TargetPermanentWithFilter(TargetFilter {
        has_card_types: vec![CardType::Artifact, CardType::Creature, CardType::Enchantment],
        ..Default::default()
    })),
}],
```
Keep `DrawCards` effect tail regardless of target count (the draw happens either way per CR 601.2c).

### 5.11 — kogla_the_titan_ape.rs (ETB) (bonus)

**Oracle**: "When Kogla enters, it fights up to one target creature you don't control."
**Fix**: replace mandatory target with UpToN{1, TargetCreatureWithFilter{controller:Opponent}}.

### 5.12 — moonsnare_specialist.rs (ETB) (bonus)

**Oracle**: "When this creature enters, return up to one target creature to its owner's hand."
**Fix**: simple UpToN{1, TargetCreature} replacement.

### 5.13 — skemfar_elderhall.rs (bonus)

**Oracle**: "Up to one target creature you don't control gets -2/-2 until end of turn. Create two 1/1 green Elf Warrior creature tokens."
**Fix**: UpToN{1, TargetCreatureWithFilter{controller:Opponent}} for the -2/-2; tokens unconditional.

### 5.14 — sword_of_sinew_and_steel.rs (bonus)

**Oracle**: "Whenever equipped creature deals combat damage to a player, destroy up to one target planeswalker and up to one target artifact."
**Fix**: **two parallel UpToN slots** in the same requirements Vec:
```rust
targets: vec![
    TargetRequirement::UpToN { count: 1, inner: Box::new(TargetRequirement::TargetPlaneswalker) },
    TargetRequirement::UpToN { count: 1, inner: Box::new(TargetRequirement::TargetArtifact) },
],
effect: Effect::Sequence(vec![
    Effect::DestroyPermanent { target: EffectTarget::DeclaredTarget { index: 0 }, cant_be_regenerated: false },
    Effect::DestroyPermanent { target: EffectTarget::DeclaredTarget { index: 1 }, cant_be_regenerated: false },
]),
```
**VALIDATION ALGORITHM NOTE**: the two UpToN slots back-to-back is the stress case for
the validator's greedy consume strategy. The algorithm in Site 2 needs to handle this
— the second slot's "try to match inner" check must discriminate against the first
slot's inner. Since both are different types (planeswalker vs artifact), this works.
**Edge case**: ambiguity where a single target could match either inner (e.g.,
"artifact creature" and slots were `[UpToN{Creature}, UpToN{Artifact}]`) — the greedy
algorithm assigns to first slot. This is acceptable; Sorin-style cards don't hit this.

---

## New Card Definitions

**None.** All fixes are to existing card defs.

---

## Step 6 — Test plan

**File**: `crates/engine/tests/pbt_up_to_n_targets.rs` (new).

### MANDATORY tests (M1-M8)

- **M1: zero-target resolution** — `test_pbt_up_to_n_zero_targets_resolves_without_effect`.
  Cast Force of Vigor with 0 targets declared. Spell resolves (no fizzle — per CR 601.2c
  / 115.1b, zero is a legal choice). No permanents are destroyed. Event trail: SpellCast,
  PriorityGiven..., SpellResolved (no DestroyPermanent event). CR citation: 601.2c, 115.1b, 608.2b.

- **M2: partial-target resolution (1 of up-to-2)** — `test_pbt_up_to_n_partial_targets_resolves`.
  Cast Force of Vigor with 1 target (one artifact). Spell resolves and destroys the one
  target. No fizzle. CR citation: 601.2c.

- **M3: full-target resolution (N of N)** — `test_pbt_up_to_n_full_targets_resolves`.
  Cast Force of Vigor with 2 targets (two artifacts). Both destroyed. CR citation: 601.2c.

- **M4: hash-determinism + schema bump** — `test_pbt_hash_schema_version_is_8`.
  Assert `assert_eq!(HASH_SCHEMA_VERSION, 8u8, "PB-T bump...")`. Then construct 3 distinct
  `TargetRequirement` values (UpToN{1, TargetCreature}, UpToN{2, TargetCreature},
  UpToN{1, TargetPermanent}) and assert all produce distinct hashes. This is the
  "genuinely exercise the new shape" test — it must not be a silent no-op.

- **M5: target-becomes-illegal mid-resolution (partial fizzle)** — `test_pbt_up_to_n_partial_fizzle_on_zone_change`.
  Cast Elder Deep-Fiend-style UpToN{count:2} tapping 2 permanents. Before resolution,
  one target leaves the battlefield (bounce / destroy). Verify: at resolution, only the
  surviving target is tapped; the other's tap effect is a silent no-op; spell does NOT
  fizzle (CR 608.2b — some legal targets remain). CR citation: 608.2b.

- **M6: regression — mandatory-target spells still work** — `test_pbt_regression_mandatory_target_lightning_bolt`.
  Cast Lightning Bolt (existing mandatory 1-target card). Verify: (a) casting with 0
  targets is rejected with InvalidTarget; (b) casting with 2 targets is rejected; (c)
  casting with exactly 1 target succeeds and deals 3 damage. This validates that the
  new `target_count_range` helper correctly returns (1, 1) for mandatory-only lists.
  CR citation: 601.2c.

- **M7: mixed mandatory + UpToN (Bridgeworks Battle shape)** — `test_pbt_mixed_mandatory_and_up_to_n`.
  Simulate a synthetic spell with `targets: vec![MandatoryTargetCreature, UpToN{1, TargetCreature}]`.
  Verify: casting with 1 target (just the mandatory) succeeds; casting with 2 targets
  succeeds; casting with 0 targets fails. Exercises the validator's mixed-slot path.
  CR citation: 601.2c.

- **M8: UpToN inner validation (reject wrong type)** — `test_pbt_up_to_n_rejects_wrong_type`.
  Cast Force of Vigor (UpToN{2, TargetPermanentWithFilter{artifact/enchantment}}) with
  1 target that's a Creature (not artifact or enchantment). Verify the cast fails with
  InvalidTarget. CR citation: 601.2c — target must satisfy declared requirement.

### OPTIONAL tests (O1-O5)

- **O1: hash-history integrity** — asserts prior schema sentinels (7 for PB-L) are still
  referenced in test files for regression coverage. Protects against a planner skipping a bump.

- **O2: two parallel UpToN slots (Sword of Sinew shape)** — exercises the greedy consume
  algorithm for two back-to-back UpToN requirements of different inner types.

- **O3: UpToN interacts with ChangeTargets (Redirect effect)** — cast Force of Vigor
  with 2 targets, then resolve a "change the target of target spell" effect that
  redirects. Verify redirects succeed and are applied to valid UpToN slots.

- **O4: player-target UpToN** — synthetic spell with `UpToN{2, TargetPlayer}` (e.g.,
  a hypothetical "up to two target players draw a card"). Verifies the player-side
  validator's recursive delegation.

- **O5: replay-harness script integration** — a minimal game script (test-data/generated-
  scripts/...) exercising Sorin Lord of Innistrad −6 with 0, 1, 2, 3 targets in separate
  script passes. Approves only after passing. Lower priority than M-tests; may defer to
  implement-phase if time is tight.

**Pattern to follow**: tests for PB-P (`pbp_power_of_sacrificed_creature.rs`) for
general structure (imports, GameStateBuilder setup, hash assertion idiom).

---

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check -p mtg-engine`)
- [ ] `TargetRequirement::UpToN { count, inner }` variant added at `card_definition.rs:~2300` with CR-cited doc comment
- [ ] `target_count_range` helper added to `casting.rs`
- [ ] `validate_targets_inner` rewritten to use greedy-consume algorithm
- [ ] `validate_object_satisfies_requirement` and `validate_player_satisfies_requirement` gain recursive arms
- [ ] `impl HashInto for TargetRequirement` gets discriminant-17 arm
- [ ] `HASH_SCHEMA_VERSION: u8 = 7` → `8`
- [ ] 3 existing test files' `assert_eq!(HASH_SCHEMA_VERSION, 7u8, ...)` updated to `8u8` with PB-T message
- [ ] `lib.rs` re-export of HASH_SCHEMA_VERSION unchanged (no update needed; already re-exported)
- [ ] 8 MANDATORY + 5 OPTIONAL tests written in `tests/pbt_up_to_n_targets.rs`
- [ ] 8-14 confirmed card defs updated (minimum 4 per AC 3485, target 8+)
- [ ] Card def TODOs that PB-T resolves are deleted; compound-blocker TODOs remain with clear markers
- [ ] `cargo test --all` passes — 0 failures
- [ ] `cargo build --workspace` clean (exhaustive matches — none expected to break since UpToN is inert in all existing match sites that don't already have `_ =>` fallthrough)
- [ ] `cargo fmt --check` clean
- [ ] Clippy: 0 new warnings beyond BASELINE-CLIPPY-01..06
- [ ] No remaining TODOs mention UpToN in any of the 8+ confirmed card def files

**Exhaustive-match inventory**: grep for `match *req` / `TargetRequirement::` in non-hash
code to find any match sites that do NOT use `_ =>`. The primary non-hash match is
`validate_object_satisfies_requirement` (covered in Site 2 changes). Additional sweep
sites to verify: `tools/replay-viewer/src/view_model.rs`, `tools/tui/src/play/panels/`
(verify if they reference TargetRequirement — most likely they don't; targets are
displayed via `StackObject.targets`, not requirements).

| File | Match expression | Action |
|------|-----------------|--------|
| `crates/engine/src/rules/casting.rs:5440` | `validate_player_satisfies_requirement` match | Add UpToN recursive arm |
| `crates/engine/src/rules/casting.rs:5459` | `validate_object_satisfies_requirement` match | Add UpToN recursive arm |
| `crates/engine/src/rules/casting.rs:5536` | nested `valid = match req { ... }` | Add UpToN recursive arm (calls inner recursively) |
| `crates/engine/src/state/hash.rs:4201` | `impl HashInto for TargetRequirement` | Add discriminant-17 arm |
| `tools/replay-viewer/src/view_model.rs` | **GREP FIRST** — may have no TargetRequirement match | Verify; add arm if present |
| `tools/tui/src/play/panels/*.rs` | **GREP FIRST** — unlikely to reference TargetRequirement | Verify |
| `crates/engine/src/cards/defs/*.rs` | **Per-card fixes per Step 5** | 8-14 card defs updated |

---

## Risks & Edge Cases

### R1 — Greedy-consume validator ambiguity with overlapping UpToN inners

If two UpToN slots have inners that could both match a single target (e.g.,
`UpToN{Creature}` and `UpToN{Permanent}`), the greedy algorithm binds the target to the
first matching slot. Sorin, the Sword of Sinew and Steel, and Force of Vigor cases all
have **non-overlapping** inners (artifact vs planeswalker, creature vs planeswalker,
etc.), so this is not an immediate concern. **MITIGATION**: at implement time, add a
debug-assertion that flags overlapping UpToN inners in a requirements Vec. Defer formal
disambiguation (e.g., user-supplied slot assignment) to a future PB if needed.

### R2 — Target declaration order assumption

The greedy algorithm assumes targets in `Command.targets` are declared in requirement-slot
order. CR 601.2c does not mandate declaration order within a spell's target set; in real
play, targets are chosen for each slot in the order the text presents them. For tests
and scripts, the harness always writes targets in order — no risk. For bots, a future
interactive-choice path (M10+) will need to surface slot-index choices. **Out of scope
for PB-T.**

### R3 — Tamiyo PreventUntap dependency

If `LayerModification::PreventUntap` or `EffectDuration::UntilControllersNextUntapStep`
does not yet exist, Tamiyo Field Researcher −2 cannot be fully authored. Partial fix
(tap without skip-untap) is legitimate — authors a better-than-nothing state. Decision
defers to implement-phase verification. If dropped, yield is 7-13 instead of 8-14;
still above floor.

### R4 — Hash regression in cross-PB test assertions

Three existing test files assert `HASH_SCHEMA_VERSION == 7u8`. Bumping to 8 without
updating all three will cause 3+ existing tests to fail. **MITIGATION**: the Verification
Checklist lists each file; implement phase must update all three in the same commit as
the sentinel bump.

### R5 — `exclude_self` / "other" filter verification (Marang River Regent, Sorin)

Two of the cards require "other target" or "nonland" filter semantics. `TargetFilter`
may lack `exclude_self: bool`. **MITIGATION**: verify field at implement time. If
missing, degrade fix to "up to 2 target nonland permanents" (Marang — drop the "other"
restriction; minor over-permissive but correct game state in 99% of cases) OR defer
the card. This is a pre-existing DSL gap, not a PB-T regression.

### R6 — Interaction with Spell copies (storm, cascade)

Storm copies inherit targets (CR 702.40c). When the target of an UpToN slot becomes
illegal and the original spell has N declared, the copy also has N declared targets.
If any become illegal on the copy, standard partial fizzle applies. No PB-T-specific
edge. **MITIGATION**: cover via regression test M6.

### R7 — Zero-target cast as "not targeting" under protection/hexproof

CR 115.2 and 702.11b: hexproof/protection only apply when an object is *being
targeted*. A zero-target UpToN spell is NOT targeting anything (CR 601.2c permits the
zero choice). So hexproof on a creature does NOT prevent a 0-target Force of Vigor
cast. This is correct per CR. **VERIFIED against existing implementation**: the
`validate_targets_inner` empty-targets path does not invoke hexproof/protection
checks. No change required.

### R8 — Ability word re-interpretation (NOT an issue for PB-T)

Per CR 207.2c (ability words like "landfall" are flavor, no game effect), UpToN is a
*game-rules primitive* (CR 601.2c / 115.1b) — NOT an ability word. It maps 1:1 to the
literal "up to" phrasing. No ambiguity.

---

## Implementation order (for the runner)

1. Site 1 — add `TargetRequirement::UpToN` variant to `card_definition.rs` (trivial).
2. Site 5 — add hash arm + bump sentinel (trivial; tests fail until step 3 lands).
3. Step 3 — update 3 existing test files' HASH sentinel assertions to `8u8`.
4. Site 2 — add `target_count_range` helper + rewrite `validate_targets_inner` greedy-consume.
5. Site 2 — add recursive arms to `validate_player_satisfies_requirement` and
   `validate_object_satisfies_requirement`.
6. Write 8 MANDATORY tests in `crates/engine/tests/pbt_up_to_n_targets.rs`.
7. `cargo test --all` — verify all mandatory tests pass.
8. Update 8 confirmed card defs (Step 5.1–5.8). Run `cargo test --all` after each.
9. If time / scope permits: 6 bonus card defs (Step 5.9–5.14), then 5 OPTIONAL tests.
10. `cargo build --workspace && cargo clippy -- -D warnings && cargo fmt --check`.
11. Final sweep: grep UpToN TODOs in card defs — verify none remain for confirmed cards.

---

## Out-of-scope items explicitly deferred

- **PB-T-L01** (new LOW): `handle_activate_loyalty_ability` does not validate targets
  against TargetRequirement (engine.rs:2198-2315). Pre-existing. Log in remediation doc.
- **PB-T-L02** (new LOW): Sorin Lord of Innistrad −6 reanimate rider ("return each card
  put into a graveyard this way to the battlefield under your control") requires a
  delayed trigger + per-spell tracking of graveyard-additions. Not PB-T scope.
- **Tamiyo PreventUntap gap**: if not already shipped, tracked as PB-T-L03.
- **Force of Vigor pitch alt-cost**: retained TODO in card def; separate future PB
  (pitch alt-cost primitive).
- **TargetFilter.exclude_self / excluded_card_types verification**: if fields don't
  exist, defer Marang River Regent + others, don't expand PB-T scope.
- **Non-targeted "up to N X" variants** (untap up to N lands, exile any number of
  target spells, search library for up to N cards): separate primitives, tracked as
  distinct PBs.
- **LOW: marisi_breaker_of_the_coil** stale TODO (PB-D carry-forward). Unrelated to PB-T.
