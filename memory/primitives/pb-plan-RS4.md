# Primitive Batch Plan: PB-RS4 — Face-Aware Residuals (close the 3 surviving CR 712.8d/e deviations)

**Generated**: 2026-07-26
**Task**: `scutemob-146` · Branch `feat/pb-rs4-face-aware-residuals-close-the-3-surviving-cr-7128de-`
**Primitive**: not a new DSL surface — a *correctness* batch. Three existing gathering /
deregistration sites in the transform machinery are made face-aware, closing OOS-RS-3 and
making OOS-OS4-2 fully closed.
**CR Rules**: 712.8d, 712.8e, 712.8a, 712.18, 701.28a/e, 614.1c, 614.12, 604.1, 604.3, 613.6,
714.2b, 714.3a, 714.3b
**Cards affected**: **0 coverage flips** (verified below by `back_face: Some` roster enumeration).
2 integrity notes; 0 new card defs; 0 card-def edits.
**Dependencies**: PB-OS4 (`ExileSourceAndReturnTransformed`), PB-OS4b (`rules/face.rs`,
`CardDefinition::effective_abilities`). Both shipped.
**Deferred items from prior PBs**: OOS-RS3-1 (queue-time intervening-if) and OOS-RS2-1
(`TurnFaceUp` unflattened cost) are **explicitly OUT of scope** (task-description fence).

---

## 0. Headline answers to the six planning questions

| # | Question | Answer |
|---|---|---|
| **A** | Explicit `is_transformed` param vs live read? | **Live read inside each function**, via `state.fizzle_object(id).map(\|o\| o.is_transformed).unwrap_or(false)`. Justified per-call-site in §4 — the engine writes `is_transformed` in exactly **two** places (`resolution.rs:665`, `face.rs:80`), **both strictly before** the calls at the two enter-transformed sites. No call site sets it later. |
| **B** | Index-parity hazard at either replacement site? | **No** for both — `ReplacementEffect` (`card-types/src/state/replacement_effect.rs:335-352`) carries no ability index; consumers key on `ReplacementId`. **But a real index-parity defect was found *inside* `apply_self_etb_from_definition`'s body**: its Saga path (`replacement.rs:1248-1261` → `fire_saga_chapter_triggers:1293`) produces `PendingTrigger.ability_index` against `def.abilities` while the consumer (`resolution.rs:1991-2011` / `:2023-2030`) resolves it against `def.effective_abilities(obj.is_transformed)`. See §3.4 / §6 Step 3. |
| **C** | Nine families in `deregister_face_statics` | **Full symmetric coverage of all nine — recommended and specified** (§5). No subset, no new seed. All nine collections have public read accessors and all needed comparison types derive `PartialEq`. A single private inverse-of-registration helper + a **source-scan parity gate** replace nine ad-hoc removals. |
| **D** | Reachability + test design | Deviation #1 and #2 are **genuinely reachable via the disturb cast path** — `crates/engine/tests/mechanics_a_d/disturb.rs:199-296` is a working end-to-end template. All nine deregistration families are **genuinely reachable** via `Effect::TransformSelf` on synthetic DFCs (pattern: `pb_os4b_face_aware_abilities.rs:684-776`). **Every RS4 defect is probe-able fail-before/pass-after.** Details + the three regression-guard-only tests in §7. |
| **E** | Wire neutrality | **CONFIRMED. PROTOCOL_VERSION stays 27** (`rules/protocol.rs:260`), **HASH_SCHEMA_VERSION stays 63** (`state/hash.rs:578`), `PROTOCOL_SCHEMA_FINGERPRINT` unchanged. No `Command`/`GameEvent`/`Effect`/`AbilityDefinition`/struct-field change; only *which* entries are pushed to / removed from existing collections. **No bump is forced — no re-scope signal.** |
| **F** | Scope fence | Held. Three new seeds to file (§10), none widened into. |

### Roster-recall gate (MANDATORY sweep result)

```
Grep TODO.*(back face|back-face|transform|is_transformed|712.8|face-aware|OOS-OS4-2|OOS-RS-3)
     in crates/card-defs/src/defs/  (case-insensitive)
```
**TODO sweep: 0 cards with matching comments.** Positive assertion — the gate was run and
produced no forced adds. (The only `OOS-OS4-2` mentions in `crates/card-defs/` are narrative
comments in `fable_of_the_mirror_breaker.rs:18/24/119/184` recording that PB-OS4b *fixed*
something; they do not request this primitive.)

### Yield discipline

`feedback_pb_yield_calibration` honored: **0 flips claimed, 0 flips expected.** Enumerated the
full DFC roster (`back_face: Some` → 15 files) and cross-matched every one against the affected
`AbilityDefinition` families. Nothing flips. Report the three numbers separately: **roster 15
DFCs / flips 0 / integrity repairs 2**.

---

## 1. CR Rule Text (verbatim from `mcp__mtg-rules__get_rule`)

**712.8** — Each face of a double-faced card that isn't a meld card has its own set of
characteristics. […]

- **712.8a** While a double-faced card is outside the game or in a zone other than the
  battlefield or stack, it has only the characteristics of its front face.
- **712.8d** While a double-faced permanent has its front face up, it has only the
  characteristics of its front face.
- **712.8e** While a nonmodal double-faced permanent has its back face up, it has only the
  characteristics of its back face. However, its mana value is calculated using the mana cost
  of its front face. […]
- **712.8f** While a modal double-faced spell is on the stack or a modal double-faced permanent
  is on the battlefield, it has only the characteristics of the face that's up.

**712.18** — When a double-faced permanent transforms or converts, it doesn't become a new
object. Any effects that applied to that permanent will continue to apply to it.

**701.28a** — To convert a permanent, turn it so that its other face is up. This follows rules
701.27a–f, 712.9–10, and 712.18. Those rules apply to converting a permanent just as they apply
to transforming a permanent.

**701.28e** — If an activated or triggered ability of a permanent that isn't a delayed triggered
ability of that permanent tries to convert it, the permanent does so only if it hasn't converted
or transformed since the ability was put onto the stack. […]

**614.1c** — Effects that read "[This permanent] enters with . . . ," "As [this permanent]
enters . . . ," or "[This permanent] enters as . . . " are replacement effects.

**614.12** — Some replacement effects modify how a permanent enters the battlefield. (See rules
614.1c–d.) Such effects may come from the permanent itself if they affect only that permanent
(as opposed to a general subset of permanents that includes it). They may also come from other
sources. **To determine which replacement effects apply and how they apply, check the
characteristics of the permanent as it would exist on the battlefield**, taking into account
replacement effects that have already modified how it enters the battlefield (see rule 616.1),
continuous effects from the permanent's own static abilities that would apply to it once it's on
the battlefield, and continuous effects that already exist and would apply to the permanent.

**604.1** — Static abilities do something all the time rather than being activated or triggered.
They are written as statements, and they're simply true.

**604.3** — Some static abilities are characteristic-defining abilities. […] Characteristic-defining
abilities function in all zones. They also function outside the game and before the game begins.

**613.6** — If an effect should be applied in different layers and/or sublayers, the parts of the
effect each apply in their appropriate ones. […] even if the ability generating the effect is
removed during this process.

**714.2b** — "{rN}—[Effect]" means "When one or more lore counters are put onto this Saga, if the
number of lore counters on it was less than N and became at least N, [effect]."

**714.3a** — As a Saga without the read ahead ability enters the battlefield, its controller puts
a lore counter on it. […]

**714.3b** — As a player's precombat main phase begins, that player puts a lore counter on **each
Saga they control with one or more chapter abilities**. This turn-based action doesn't use the
stack.

### How the rules bind to the code

CR 614.12's "check the characteristics of the permanent **as it would exist on the battlefield**"
+ CR 712.8e ("has only the characteristics of its back face") means: a permanent entering
back-face-up must gather its self-ETB replacements (CR 614.1c) and its permanent replacement
abilities (CR 614) **from the back face only**. CR 604.1 + 712.8e means: on an in-place flip
(CR 712.18 — same object, so nothing else cleans up), the old face's static-ability
registrations must cease. CR 714.3b's "**each Saga … with one or more chapter abilities**"
means a permanent showing a non-Saga back face must not receive lore counters.

---

## 2. Verified current-state inventory (do not re-derive; re-verify only if surprised)

| Fact | Evidence |
|---|---|
| `is_transformed` is written by the engine in exactly **two** places | `rules/resolution.rs:665` (disturb, inside the `state.objects.get_mut(&new_id)` block opened at `:589`) and `rules/face.rs:80` (inside `apply_face_change`). Grep over `crates/engine/src` for `is_transformed = ` / `is_transformed: true` returns nothing else. `crates/engine/src/testing/` writes it never. |
| Zone change resets the flag | `state/mod.rs:1397-1399` — `// CR 712.8a / CR 400.7: DFC transform state is reset on zone change.` `is_transformed: false`. Also `builder.rs:1073`, `state/mod.rs:1244/1518/1740`. |
| `effective_abilities` accessor | `crates/card-types/src/cards/card_definition.rs:180-185`. |
| `register_static_continuous_effects` already threads an explicit `is_transformed: bool` | `rules/replacement.rs:2074-2080`; `face.rs:96-102` and `resolution.rs:1711-1722` pass a computed value; all other call sites pass literal `false`. |
| `ReplacementEffect` has no ability index | `crates/card-types/src/state/replacement_effect.rs:335-352`. |
| PROTOCOL / HASH | `rules/protocol.rs:260` = 27; `state/hash.rs:578` = 63. |
| DFC roster | 15 files with `back_face: Some` (see §8). |

---

## 3. The four deviations

### 3.1 Deviation #1 — `apply_self_etb_from_definition` reads the FRONT face

**File**: `crates/engine/src/rules/replacement.rs`
**Function**: `apply_self_etb_from_definition` (`:1160-1275`)
**Defect**: `:1191` `for ability in &def.abilities {` — gathers the FRONT face's
`Replacement { trigger: WouldEnterBattlefield, is_self: true }` abilities regardless of which
face is up. Self-labelled at `:1180-1190` ("PB-OS4b limitation (OOS-OS4-2)").
**CR violated**: 614.12 + 712.8e.
**Reachable via**: `resolution.rs:1673` (disturb cast, `is_transformed` already `true` at the
call) and `resolution.rs:7279` (stack craft path, `apply_face_change` ran at `:7276`).

### 3.2 Deviation #2 — `register_permanent_replacement_abilities` reads the FRONT face

**File**: `crates/engine/src/rules/replacement.rs`
**Function**: `register_permanent_replacement_abilities` (`:1892-2051`)
**Defect**: `:1913` `for ability in &def.abilities {`. Comment at `:1907-1912`.
**CR violated**: 614 + 712.8e.
**Reachable via**: `resolution.rs:1688` (disturb) **only** — the craft path (`resolution.rs:7241`)
and `Effect::ExileSourceAndReturnTransformed` (`effects/mod.rs:4273`) never call this function at
all. That absence is a *separate* pre-existing gap → seed, not scope (§10, OOS-RS4-1).

### 3.3 Deviation #3 — `deregister_face_statics` handles only `AbilityDefinition::Static`

**File**: `crates/engine/src/rules/face.rs`
**Function**: `deregister_face_statics` (`:149-171`), doc `:104-148`.
**Defect**: nine other families registered by `register_static_continuous_effects` are never
removed when the permanent transforms away from the face that declared them. On an in-place flip
(CR 712.18: *same object*), nothing else ever cleans them up while the permanent stays on the
battlefield, so the old face's Panharmonicon / Torpor Orb / Rule of Law / CDA P/T / extra land
drop / flash grant / play-from-graveyard / play-from-top effect keeps applying.
**CR violated**: 604.1 + 712.8e (the permanent no longer has that static ability, so it cannot
still "do something all the time").
**Reachable via**: `Effect::TransformSelf` → `rules/engine.rs:1225` `apply_face_change` → `:73`
`deregister_face_statics`, on any DFC whose *front* face declares one of the nine.

### 3.4 Deviation #4 (FOUND DURING PLANNING — not in the brief): Saga lore-counter + chapter-index face-blindness

Two coupled front-face reads with the same root cause and the same CR:

1. **`rules/turn_actions.rs:377-389`** — the CR 714.3b precombat-main lore-counter sweep filters
   on `def.abilities.iter().any(|a| matches!(a, AbilityDefinition::SagaChapter { .. }))`
   (`:380-383`, FRONT face). A transformed Fable of the Mirror-Breaker (i.e. *Reflection of
   Kiki-Jiki* — an Enchantment Creature with **no** chapter abilities) is therefore still swept:
   it accrues a lore counter each precombat main and re-fires chapters I/II/III off the back
   face. CR 714.3b says "each Saga they control **with one or more chapter abilities**"; per
   CR 712.8e the transformed permanent has neither. `rules/sba.rs:843-850` already got this right
   in PB-OS4b (`effective_abilities`), so the sweep and the SBA currently disagree.
2. **`rules/replacement.rs:1293`** — `fire_saga_chapter_triggers` enumerates `def.abilities` to
   produce `PendingTrigger.ability_index`, while **both** consumers resolve that index against
   `def.effective_abilities(obj.is_transformed)` (`resolution.rs:1996-1998` and `:2028-2030`) and
   the CR 714.4 SBA guard does the same (`sba.rs:889-891`). Producer and consumer index
   namespaces disagree exactly when `is_transformed` is true. This is **the PB-OS4b central
   hazard, still live**, and it sits inside deviation #1's own function body
   (`replacement.rs:1248-1261`).

**Recommendation: INCLUDE deviation #4 in PB-RS4.** Rationale: (a) same seed family (OOS-OS4-2 /
CR 712.8d-e face awareness), (b) same one-line mechanism (`effective_abilities`), (c) it is the
*index-parity* question the brief's item B asks about and leaving it open would make "OOS-OS4-2
fully closed" false again, (d) it is currently *reachable in-tree* using the shipped
`fable_of_the_mirror_breaker` def, which makes it the cleanest fail-before/pass-after probe in
the batch. Blast radius: two `def.abilities` → `def.effective_abilities(..)` swaps.

**Not a coverage flip**: `fable_of_the_mirror_breaker` is `Completeness::partial`, so
`validate_deck` rejects it — the bug cannot reach a real game today. It stays `partial` (its two
stated blockers are untouched). Report as an **integrity repair**, not a flip.

---

## 4. Face-signal threading — per-call-site analysis (brief item A)

### 4.1 The decision

**Read `is_transformed` live inside each function; do not add a parameter.**

Chosen form (single snapshot at the top of the function, reused for every scan in that function):

```rust
// PB-RS4 (CR 614.12 / 712.8e): gather from the face that is actually showing.
// CR 400.7: an earlier same-batch ETB replacement (or a harness call) may leave no
// live object for this id -- a departed permanent has no face, which is a legal
// fizzle, not an engine bug. Mirrors `queue_carddef_etb_triggers`'s guard
// (`replacement.rs:1341`).
let entering_is_transformed = state
    .fizzle_object(new_id)
    .map(|o| o.is_transformed)
    .unwrap_or(false);
```

### 4.2 Why live-read and not an explicit parameter

1. **The object always already carries the right value at every call site.** The engine writes
   `is_transformed` in exactly two places, and both are strictly *before* the calls (§2). No call
   site sets it afterwards. Verified per site in the tables below — not assumed uniform.
2. **The two closest siblings already do it this way.** `queue_carddef_etb_triggers`
   (`replacement.rs:1429-1436`) — the OS4b-fixed twin of these two functions, called from the
   same ETB blocks — reads the live flag; and the *caller* at `resolution.rs:1711-1715` computes
   `register_static_continuous_effects`'s explicit argument by doing exactly this live read.
   Adding a parameter here would make three sibling functions use three different conventions.
3. **`register_static_continuous_effects`'s explicit parameter is not a precedent to mirror.** It
   needs the parameter because it has one caller — `face.rs:96-102` — that must register the
   *new* face at a flip boundary; the explicit form documents that contract. Neither of RS4's two
   functions is ever called from a flip boundary.
4. **Churn/risk**: a parameter forces edits at 13 + 13 engine call sites **and ~35 test call
   sites** (`tests/rules/replacement_effects.rs` alone calls `apply_self_etb_from_definition`
   16 times), every one of which would pass `false`. That is a large mechanical diff whose only
   effect is to re-encode a value the object already holds — high error surface, zero behavioral
   gain.
5. **SR-25 bare-lookup ratchet stays green**: `fizzle_object` is diagnostics vocabulary, not a
   bare `.objects.get(`, so the pinned counts for `src/rules/replacement.rs` (24),
   `src/rules/turn_actions.rs` (7) and the unlisted `src/rules/face.rs` are unchanged
   (`tests/core/bare_lookup_ratchet.rs:52-159`).

**SR-4 side-taking**: absence is classified as a **fizzle** (CR 400.7 / 614 — the permanent never
finished entering), matching `replacement.rs:1341`'s already-argued position for the same
`new_id` in the same ETB batch. Do **not** use `expect_object` here: at
`register_permanent_replacement_abilities`'s call sites the preceding `apply_etb_replacements`
can legitimately have moved the object, and `expect_object`'s `debug_assert!` would panic
debug-mode tests.

### 4.3 `apply_self_etb_from_definition` — every engine (non-test) call site: 13

| # | Site | Context | `is_transformed` at the call | Written where |
|---|---|---|---|---|
| 1 | `effects/mod.rs:1628` | mass reanimate loop | `false` | `move_object_to_zone` reset |
| 2 | `effects/mod.rs:5948` | land from hand to battlefield (search-a-land effect) | `false` | reset |
| 3 | `effects/mod.rs:6188` | `ReturnAllFromGraveyardToBattlefield` | `false` | reset |
| 4 | `effects/mod.rs:6476` | exile-then-return-all | `false` | reset |
| 5 | `rules/lands.rs:113` | `handle_play_land` | `false` | reset (MDFC back-face play is unimplemented — §8) |
| 6 | **`rules/resolution.rs:1673`** | **main cast ETB — disturb enters transformed** | **`true` for a disturb cast** | **`resolution.rs:665`, ~1000 lines earlier in the same match arm** |
| 7 | `rules/resolution.rs:3162` | unearth | `false` | reset |
| 8 | `rules/resolution.rs:4413` | champion / exile-return (CR 702.72a) | `false` | reset |
| 9 | `rules/resolution.rs:6146` | ninjutsu | `false` | reset |
| 10 | `rules/resolution.rs:6368` | token creation | `false` | fresh `add_object` |
| 11 | `rules/resolution.rs:6594` | token creation | `false` | fresh `add_object` |
| 12 | `rules/resolution.rs:6835` | token creation | `false` | fresh `add_object` |
| 13 | **`rules/resolution.rs:7279`** | **stack craft return (CR 702.167a)** | **`true`** | **`face.rs:80` via `apply_face_change` at `resolution.rs:7276`, three lines earlier** |

Sites 6 and 13 are the only ones where the read differs from `false`; both already hold `true`.
**No call site edits are required.**

### 4.4 `register_permanent_replacement_abilities` — every engine (non-test) call site: 13

| # | Site | Context | `is_transformed` at the call |
|---|---|---|---|
| 1 | `effects/mod.rs:1642` | mass reanimate | `false` |
| 2 | `effects/mod.rs:5962` | land-to-battlefield | `false` |
| 3 | `effects/mod.rs:6202` | return-all-from-graveyard | `false` |
| 4 | `effects/mod.rs:6486` | exile-then-return-all | `false` |
| 5 | `rules/lands.rs:375` | `handle_play_land` | `false` |
| 6 | **`rules/resolution.rs:1688`** | **main cast ETB — disturb** | **`true` for a disturb cast** (set at `:665`; `apply_etb_replacements` at `:1682` does not touch the flag) |
| 7 | `rules/resolution.rs:3174` | unearth | `false` |
| 8 | `rules/resolution.rs:4423` | champion / exile-return | `false` |
| 9 | `rules/resolution.rs:6158` | ninjutsu | `false` |
| 10 | `rules/resolution.rs:6379` | token creation | `false` |
| 11 | `rules/resolution.rs:6605` | token creation | `false` |
| 12 | `rules/resolution.rs:6846` | token creation | `false` |
| 13 | `rules/engine.rs:2003` | `place_opening_hand_permanents` (Leyline) | `false` — CR 712.8a, a card in the opening hand is front-face |

**No call site edits are required.** Note the two *missing* call sites (stack craft at
`resolution.rs:7279+`, `ExileSourceAndReturnTransformed` at `effects/mod.rs:4337+`, plus
`engine.rs handle_craft` at `:1432-1453` which calls none of the ETB chain) — seed, §10.

---

## 5. Deregistration design for all nine families (brief item C)

### 5.1 Registration inventory — read from `register_static_continuous_effects` (`replacement.rs:2074-2306`) in full

| # | `AbilityDefinition` variant | Reg. line | Target collection | Entries per ability | Source field type | Identifying fields (all `PartialEq`) |
|---|---|---|---|---|---|---|
| 0 | `Static` *(already handled)* | `:2097-2120` | `state.continuous_effects` | 1 | `Option<ObjectId>` | `layer`, `duration`, `modification`, resolved `filter` |
| 1 | `TriggerDoubling` | `:2122-2134` | `state.trigger_doublers` | 1 | `ObjectId` | `filter: TriggerDoublerFilter` (PartialEq @ `stubs.rs:477`), `additional_triggers: u32` |
| 2 | `SuppressCreatureETBTriggers` | `:2136-2143` | `state.etb_suppressors` | 1 | `ObjectId` | `filter: ETBSuppressFilter` (PartialEq @ `stubs.rs:526`) |
| 3 | `StaticRestriction` | `:2145-2153` | `state.restrictions` | 1 | `ObjectId` | `restriction: GameRestriction` (PartialEq @ `stubs.rs:556`) |
| 4 | `CdaPowerToughness` | `:2157-2179` | `state.continuous_effects` | 1 | `Option<ObjectId>` | `layer == PtCda`, `is_cda == true`, `modification == SetPtDynamic { power, toughness }`, `filter == SingleObject(obj_id)`, `duration == WhileSourceOnBattlefield` |
| 5 | `CdaModifyPowerToughness` | `:2195-2244` | `state.continuous_effects` | **1 or 2** (one per `Some(power)` / `Some(toughness)`; **0** if both `None`) | `Option<ObjectId>` | `layer == PtModify`, `is_cda == true`, `modification == ModifyPowerDynamic{amount,negate:false}` and/or `ModifyToughnessDynamic{..}`, `filter == SingleObject(obj_id)` |
| 6 | `AdditionalLandPlays` | `:2248-2256` | `state.additional_land_play_sources` | 1 | `ObjectId` | `count: u32` (whole struct derives `PartialEq`, `stubs.rs:735`) |
| 7 | `StaticFlashGrant` | `:2259-2269` | `state.flash_grants` | 1 | **`Option<ObjectId>`** | `filter: FlashGrantFilter` (PartialEq @ `stubs.rs:632`), `duration == WhileSourceOnBattlefield` |
| 8 | `StaticPlayFromGraveyard` | `:2271-2280` | `state.play_from_graveyard_permissions` | 1 | `ObjectId` | `filter: PlayFromTopFilter` (PartialEq @ `stubs.rs:661`), `condition: Option<Condition>` (PartialEq @ `card_definition.rs:3675`) |
| 9 | `StaticPlayFromTop` | `:2282-2302` | `state.play_from_top_permissions` | 1 | `ObjectId` | `filter`, `look_at_top`, `reveal_top`, `pay_life_instead`, `condition` |

**Observability**: every collection has a public read accessor — `continuous_effects()`
(`state/mod.rs:407`), `trigger_doublers()` (`:442`), `etb_suppressors()` (`:447`),
`restrictions()` (`:452`), `flash_grants()` (`:457`), `play_from_top_permissions()` (`:462`),
`play_from_graveyard_permissions()` (`:467`), `additional_land_play_sources()` (`:563`).

**Nothing is infeasible.** The OS4b doc comment's stated reasons for deferring
("heterogeneous collection shapes, 1-or-2 entries per ability, no shared tuple") are real but do
not block a precise removal: each family has a small, fully-`PartialEq` identifying tuple. The
`Option<ObjectId>` vs `ObjectId` split affects families 0/4/5/7 and is handled by writing
`Some(obj_id)` vs `obj_id` in those arms. **Recommend full symmetric coverage. No subset, no
residual seed.**

### 5.2 Shape: one inverse-of-registration helper, not nine ad-hoc removals

Replace `deregister_face_statics`'s body with a loop over the old face's abilities that delegates
to a new private helper whose `match` is a **1:1 structural mirror of
`register_static_continuous_effects`'s `match`, arm for arm, in the same order**:

```rust
/// The exact inverse of one `register_static_continuous_effects` match arm.
/// Removes AT MOST the number of entries that arm would have registered
/// (one, or two for `CdaModifyPowerToughness`), matching structurally on
/// `source == obj_id` plus that family's identifying fields.
fn remove_one_registration(state: &mut GameState, obj_id: ObjectId, ability: &AbilityDefinition)
```

Rules the runner must follow inside it:

- **Remove at most the registered count**, never `retain`-purge by source. Other code registers
  into these collections with the same source id: `resolution.rs:7447-7470` (Class level-up pushes
  both a `ContinuousEffect` and an `AdditionalLandPlaySource` with `source: <the Class
  permanent>`), `effects/mod.rs:5574` (emblem `PlayFromGraveyardPermission`, different
  `ObjectId`), `effects/mod.rs:6084-6091` (`Effect::GrantFlash`, `source: None` — cannot collide).
  A bulk purge would delete those.
- **First-match `position(..)` + `remove(pos)`**, exactly like the existing `Static` arm
  (`face.rs:160-168`). Where a same-source duplicate could exist (Class level-up), the two
  entries are field-identical, so removing either is observationally identical — note this in the
  doc comment rather than adding tie-breaking machinery.
- **Resolve `EffectFilter::Source -> SingleObject(obj_id)`** before comparing, as the existing arm
  does (`face.rs:156-159`); the two CDA arms register `SingleObject(new_id)` directly.
- **`CdaModifyPowerToughness` removes up to two entries** — build the same
  `modifications: Vec<LayerModification>` the registration builds (`replacement.rs:2209-2226`)
  and remove one entry per element. Both-`None` removes nothing.
- Use **fully-qualified `AbilityDefinition::X`** patterns (no `use AbilityDefinition as A;`) —
  `tests/core/ability_definition_registry.rs:643` (`use_imports_do_not_bypass_the_scanner`)
  forbids aliasing, and the site scan at `:201` needs the literal `AbilityDefinition::X` text.

### 5.3 Drift guard: registration/deregistration parity gate (REQUIRED)

The `_ => {}` catch-all on both matches means a family added to registration later would silently
re-open this hole. Add a source-scan gate in the SR-5/SR-8 style:

**File**: `crates/engine/tests/core/face_dereg_parity.rs` (+ `mod face_dereg_parity;` in
`crates/engine/tests/core/main.rs`).
**Test**: `registration_and_deregistration_cover_the_same_ability_families`.
**Method**: brace-match the body of `register_static_continuous_effects` out of
`crates/engine/src/rules/replacement.rs` and the body of `remove_one_registration` out of
`crates/engine/src/rules/face.rs`; strip `//`-to-EOL comments (reuse the technique documented at
`tests/core/bare_lookup_ratchet.rs:185-200`); collect every `AbilityDefinition::<Name>` token
(applying the same alphanumeric/`_` boundary check as
`tests/core/ability_definition_registry.rs:207-210`, so `Static` does not match inside
`StaticRestriction`); assert the two `BTreeSet<String>`s are equal, with a failure message naming
the missing family and pointing at this plan. Add a non-vacuity assertion (`>= 10` names found in
each) so a broken extractor cannot pass silently.

---

## 6. Engine Changes — step-numbered, with file:line targets

> Order matters: **Step 1 (probes) must land and be observed FAILING before Steps 2-6.**
> AC 5458 requires fail-before/pass-after where the defect is reachable, and every RS4 defect is
> reachable (§7).

### Step 0 — orientation probe (no code shipped)

Confirm the two "already `true` at the call" claims empirically before changing anything. Add a
temporary `eprintln!` of `state.objects.get(&new_id).map(|o| o.is_transformed)` at the top of
`apply_self_etb_from_definition`, run
`cargo test -p mtg-engine --test mechanics_a_d disturb::test_disturb_enters_transformed -- --nocapture`,
and observe `Some(true)`. Then revert the probe. If it prints `Some(false)`, **STOP** — the
threading decision in §4 is wrong and the plan must be re-scoped to an explicit parameter.

### Step 1 — write the probe tests FIRST and verify them RED

**File (new)**: `crates/engine/tests/primitives/pb_rs4_face_aware_residuals.rs`
**Registration**: add `mod pb_rs4_face_aware_residuals;` to
`crates/engine/tests/primitives/main.rs` in alphabetical position (after
`mod pb_rs3_rabblemaster_mustattack_probe;` at `:48`). **SR-9a: never create a top-level
`tests/*.rs`.**

Write the full test list from §7. Run `cargo test -p mtg-engine --test primitives pb_rs4` and
**record, per test, whether it fails**. Any test that passes against unmodified HEAD is a
test-validity bug (`memory/conventions.md` — "test-validity MEDIUMs are fix-phase HIGHs"): fix
the test until it discriminates, or move it to the explicitly-labelled "pass-after regression
guard" list in §7.3 with a written reason.

### Step 2 — `apply_self_etb_from_definition` becomes face-aware

**File**: `crates/engine/src/rules/replacement.rs`

2a. Insert the live-read snapshot immediately after the `let Some(def) = registry.get(...)` guard
    (currently `:1171-1173`), before `let mut evts = Vec::new();` at `:1175`. Use the exact
    `fizzle_object` form from §4.1, binding `let entering_is_transformed: bool`.

2b. **Replace** the limitation comment at `:1180-1190` with an accurate one:
    ```
    // PB-RS4 (CR 614.12 / 712.8d/e, OOS-RS-3): gather from the face that is
    // actually showing. A permanent entering back-face-up (disturb -- resolution.rs
    // :665; stack craft -- resolution.rs:7276) has only its back face's
    // characteristics, so only that face's self-ETB replacements apply.
    ```
    **Do not leave the old text** (`memory/conventions.md` — aspirationally-wrong comments are
    correctness hazards).

2c. `:1191` — `for ability in &def.abilities {` → `for ability in def.effective_abilities(entering_is_transformed) {`.
    The item type changes from `&&AbilityDefinition` to `&AbilityDefinition`; the
    `if let AbilityDefinition::Replacement { .. } = ability` pattern at `:1192-1197` compiles
    unchanged (the existing `register_static_continuous_effects` loop at `:2095` proves the shape).

2d. `:1248-1251` `has_saga_chapters` — `def.abilities.iter()` → `def.effective_abilities(entering_is_transformed).iter()`.
    **CR 714.3a**: only a permanent that is *actually* a Saga on the visible face gets the entry
    lore counter.

2e. `:1264-1267` `has_class_levels` — same swap. (Classes are not DFCs today; the swap is free and
    keeps the function internally consistent.)

2f. `:1233-1245` `def.starting_loyalty` — **leave front-face**. Add a one-line pointer comment:
    `// CR 306.5b: back-face starting loyalty is OOS-OS4-1 / queue item R10 -- deliberately front-only here.`
    Do not widen.

### Step 3 — `fire_saga_chapter_triggers` index parity (deviation #4, part 2)

**File**: `crates/engine/src/rules/replacement.rs`, fn at `:1282-1305`.

3a. Add a live read at the top of the function body (same `fizzle_object` form, on `saga_id`).
    Do **not** add a parameter — the fn is `pub` and is called from `turn_actions.rs:411` and
    `tests/mechanics_m_z/saga_class.rs:220`; a live read keeps both callers untouched.

3b. `:1293` — `for (ability_index, ability) in def.abilities.iter().enumerate()` →
    `... def.effective_abilities(is_transformed).iter().enumerate()`.

3c. Add a doc line to the fn's doc comment (`:1276-1281`) recording the contract:
    *"CR 712.8d/e: `ability_index` is a dense index into the currently-visible face's effective
    list — the same namespace the consumers use (`resolution.rs:1996`/`:2028`, `sba.rs:889`)."*

### Step 4 — `register_permanent_replacement_abilities` becomes face-aware

**File**: `crates/engine/src/rules/replacement.rs`

4a. Insert the live-read snapshot after `let Some(def) = registry.get(...)` (`:1904-1906`).
4b. Replace the limitation comment at `:1907-1912` with an accurate CR 614 / 712.8e note.
4c. `:1913` — `for ability in &def.abilities {` → `for ability in def.effective_abilities(is_transformed) {`.

### Step 5 — CR 714.3b precombat-main Saga sweep (deviation #4, part 1)

**File**: `crates/engine/src/rules/turn_actions.rs`, `:377-389`.

5a. `:380-383` — `def.abilities.iter().any(..)` → `def.effective_abilities(obj.is_transformed).iter().any(..)`.
    `obj` is already the closure's bound `&GameObject` (`:377`), so **no new state lookup** — the
    SR-25 ceiling of 7 for this file is unchanged.
5b. Add the CR citation inline: `// CR 714.3b / 712.8e: "each Saga they control with one or more chapter abilities" -- a permanent showing a non-Saga back face is not a Saga (matches rules/sba.rs:843).`

### Step 6 — `deregister_face_statics` extended to all nine families

**File**: `crates/engine/src/rules/face.rs`

6a. **Rewrite the doc comment `:104-148` completely.** The current text argues a deferral that no
    longer exists; leaving any of it standing would be an aspirationally-wrong comment in reverse.
    New doc must: cite CR 604.1 / 613 / 712.8e / 712.18; state that the function is the structural
    inverse of `register_static_continuous_effects` for **all ten** registration families; describe
    the "remove at most the registered count, first structural match" rule; name the three
    same-source competing registrants from §5.2 and why first-match is safe for them; point at the
    parity gate (`tests/core/face_dereg_parity.rs`).
6b. Keep the function **name** `deregister_face_statics` (traceability to OOS-OS4-2 / OS4b docs
    and to `face.rs:73`'s call). Note in the doc that "statics" now means "everything
    `register_static_continuous_effects` registers".
6c. Rewrite the body as: `for ability in old_face_abilities { remove_one_registration(state, obj_id, ability); }`.
6d. Add the private `remove_one_registration` per §5.2, arms in registration order:
    `Static`, `TriggerDoubling`, `SuppressCreatureETBTriggers`, `StaticRestriction`,
    `CdaPowerToughness`, `CdaModifyPowerToughness`, `AdditionalLandPlays`, `StaticFlashGrant`,
    `StaticPlayFromGraveyard`, `StaticPlayFromTop`, `_ => {}`.
    Each arm cites its CR (604.1 for the statics / permissions / restrictions, 604.3 + 613.4a for
    `CdaPowerToughness`, 604.3 + 613.4c for `CdaModifyPowerToughness`, 603.2d for
    `TriggerDoubling`, 614.16a for `SuppressCreatureETBTriggers`, 305.2 for `AdditionalLandPlays`,
    601.3b for `StaticFlashGrant`, 601.3 + 305.1 for the two play-permissions).
6e. Note the ordering invariant preserved by `apply_face_change` (`face.rs:71-102`): deregister
    OLD → flip → rebuild Channel-A → register NEW. Do not reorder.

### Step 7 — exhaustive-match / gate updates (THE #1 SOURCE OF FAILURES — do not skip)

| File | What | Line | Action |
|---|---|---|---|
| `crates/engine/src/state/ability_definition_registry.rs` | `A::TriggerDoubling` sites | `:122-124` | **add** `"crates/engine/src/rules/face.rs"` |
| ″ | `A::SuppressCreatureETBTriggers` | `:125-127` | **add** `face.rs` |
| ″ | `A::StaticRestriction` | `:351-353` | **add** `face.rs` |
| ″ | `A::CdaPowerToughness` | `:354-356` | **add** `face.rs` |
| ″ | `A::CdaModifyPowerToughness` | `:357-359` | **add** `face.rs` |
| ″ | `A::AdditionalLandPlays` | `:360-365` | **add** `face.rs` (keep `replacement.rs`, `resolution.rs`) |
| ″ | `A::StaticFlashGrant` | `:366-368` | **add** `face.rs` |
| ″ | `A::StaticPlayFromTop` | `:369-371` | **add** `face.rs` |
| ″ | `A::StaticPlayFromGraveyard` | `:372-374` | **add** `face.rs` |
| ″ | `A::Static` | `:92-98` | **no change** — `face.rs` already listed |
| ″ | `A::SagaChapter` | `:154-161` | **no change** — `replacement.rs` + `turn_actions.rs` already listed |
| `crates/engine/tests/core/main.rs` | test module list | after `:10` | **add** `mod face_dereg_parity;` (alphabetical) |
| `crates/engine/tests/primitives/main.rs` | test module list | after `:48` | **add** `mod pb_rs4_face_aware_residuals;` |
| `crates/engine/tests/core/bare_lookup_ratchet.rs` | pinned ceilings | `:94-158` | **no change expected** (all new reads use `fizzle_object`). If `cargo test --test core bare_lookup` fails, a bare `.objects.get(` slipped in — convert it, do not raise the ceiling. |
| `crates/engine/src/state/hash.rs` | `HASH_SCHEMA_VERSION` | `:578` | **no change** (stays 63) |
| `crates/engine/src/rules/protocol.rs` | `PROTOCOL_VERSION` / fingerprint | `:260` / `:277` | **no change** (stays 27) |

**No `Effect` / `Command` / `GameEvent` / `StackObjectKind` / `KeywordAbility` variant is added**,
so the usual display-arm sites (`state/hash.rs`, `tools/replay-viewer/src/view_model.rs`,
`tools/tui/src/play/panels/stack_view.rs`) need **no** new match arms. Still run
`cargo build --workspace` — those two exhaustive matches are the project's most-missed compile
sites.

### Step 8 — full gates

```
cargo build --workspace
cargo test --all
cargo clippy --all-targets -- -D warnings
cargo fmt --check
tools/check-defs-fmt.sh          # SR-35
```

---

## 7. Test design (brief item D)

### 7.1 Fixture strategy — no speculative card defs

Two established in-tree patterns; **use both, add nothing to `crates/card-defs/`**:

- **Test-local `CardDefinition` + injected registry** —
  `crates/engine/tests/mechanics_m_z/pb_os4b_face_aware_abilities.rs:84-86` (`registry_with`),
  `:684-724` (`mock_static_dfc_def`), `:726-734` (`mock_static_dfc_on_battlefield`),
  `:740-776` (the full "register front statics manually via
  `register_static_continuous_effects(&mut state, id, card_id.as_ref(), &registry, false)`, run
  `Effect::TransformSelf`, assert removed" recipe). Copy this shape verbatim for all nine
  deregistration tests. Note `GameStateBuilder` does not replay ETB, hence the manual
  registration call.
- **Working disturb end-to-end harness** —
  `crates/engine/tests/mechanics_a_d/disturb.rs:54-103` (`beloved_beggar_def`, front carries
  `AbilityDefinition::Keyword(KeywordAbility::Disturb)` **and** `AbilityDefinition::Disturb { cost }`),
  `:105-118` (`beggar_in_graveyard`), `:199-296` (`test_disturb_enters_transformed`: build state,
  add `{1}{W}` to the pool, `Command::CastSpell(Box::new(CastSpellData { alt_cost: Some(AltCostKind::Disturb), .. }))`,
  `pass_all`, then find the battlefield object by `card_id` + `is_transformed`). Copy this for the
  four replacement probes.

Give every RS4 mock a distinct `card_id` prefix (`"mock-rs4-*"`) to avoid registry collisions.

### 7.2 Genuine fail-before / pass-after probes

| # | Test | Deviation | Fixture | Pre-fix behavior (RED) | Post-fix (GREEN) | CR |
|---|---|---|---|---|---|---|
| 1 | `test_disturb_back_face_self_etb_replacement_applies` | #1 | disturb DFC; **back** face has `Replacement { WouldEnterBattlefield{Any}, EntersTapped, is_self: true }`, front has none | permanent enters **untapped** (back face's replacement never gathered) | enters **tapped** | 614.1c, 614.12, 712.8e |
| 2 | `test_disturb_front_face_self_etb_replacement_does_not_apply` | #1 | disturb DFC; **front** face has `EntersWithCounters { PlusOnePlusOne, Box::new(Fixed(2)) }`, back has none | permanent enters with **2 +1/+1 counters** it should not have | enters with **0** | 712.8e |
| 3 | `test_disturb_back_face_permanent_replacement_is_registered` | #2 | disturb DFC; **back** face has a non-self `Replacement` (e.g. `WouldPlaceCounters` doubler) | `state.replacement_effects()` contains **no** entry with `source == Some(id)` | contains exactly one, with the back face's `modification` | 614, 712.8e |
| 4 | `test_disturb_front_face_permanent_replacement_is_not_registered` | #2 | disturb DFC; **front** face has a non-self `Replacement`, back has none | one wrong entry registered | zero entries | 712.8e |
| 5 | `test_transformed_saga_stops_accruing_lore_counters` | #4.1 | **shipped `fable_of_the_mirror_breaker`** via `all_cards()` + the `real_card_spec` helper (`pb_os4b_..:95-108`); put it on the battlefield already `is_transformed` (reach that state with `Effect::TransformSelf` or by casting chapter III's `ExileSourceAndReturnTransformed`), advance to the controller's `PreCombatMain` | a lore counter is added to Reflection of Kiki-Jiki and chapter I queues | no lore counter, no chapter trigger | 714.3b, 712.8e |
| 6 | `test_saga_chapter_trigger_index_matches_effective_face` | #4.2 | synthetic DFC whose **front** has 3 `SagaChapter` abilities and whose **back** has a single `Triggered` ability at index 0; enter/flip it transformed, then drive `fire_saga_chapter_triggers` directly (pattern: `saga_class.rs:217-222`) | producer emits an index into the front list that the consumer resolves against the back list → wrong effect fires | no chapter trigger produced at all (back face has no `SagaChapter`) | 714.2b, 712.8d/e |
| 7 | `test_transform_deregisters_trigger_doubling` | #3 | front `TriggerDoubling { ArtifactOrCreatureETB, 1 }` | `state.trigger_doublers()` still contains the entry after `TransformSelf` | empty | 603.2d, 604.1 |
| 8 | `test_transform_deregisters_etb_suppressor` | #3 | front `SuppressCreatureETBTriggers { CreaturesOnly }` | entry survives | `etb_suppressors()` empty | 614.16a |
| 9 | `test_transform_deregisters_static_restriction` | #3 | front `StaticRestriction { MaxSpellsPerTurn { max: 1 } }` | entry survives | `restrictions()` empty | 604.1 |
| 10 | `test_transform_deregisters_cda_power_toughness` | #3 | front `CdaPowerToughness { Fixed(5), Fixed(5) }`, back printed 2/2 | `calculate_characteristics` still reports 5/5 | reports 2/2 | 604.3, 613.4a |
| 11 | `test_transform_deregisters_cda_modify_both_entries` | #3 | front `CdaModifyPowerToughness { Some(Fixed(3)), Some(Fixed(2)) }` (**two** entries) | both survive | **both** removed — assert `continuous_effects()` has zero `is_cda` entries sourced by the object (this is the two-entry case the OS4b comment specifically worried about) | 604.3, 613.4c |
| 12a | `test_transform_deregisters_additional_land_plays` | #3 | front `AdditionalLandPlays { count: 1 }` | entry survives (and the controller keeps the extra land drop at their next untap) | `additional_land_play_sources()` empty of `source == obj_id` | 305.2 |
| 12b | `test_transform_deregisters_static_flash_grant` | #3 | front `StaticFlashGrant { AllSpells }` | survives | `flash_grants()` has no `source == Some(obj_id)` entry | 601.3b |
| 12c | `test_transform_deregisters_play_from_graveyard` | #3 | front `StaticPlayFromGraveyard { All, None }` | survives | `play_from_graveyard_permissions()` empty of `source == obj_id` | 601.3, 305.1 |
| 12d | `test_transform_deregisters_play_from_top` | #3 | front `StaticPlayFromTop { All, .. }` | survives | `play_from_top_permissions()` empty of `source == obj_id` | 601.3 |
| 13 | `test_transform_there_and_back_restores_all_nine_families` | #3 | one front face declaring all nine; `TransformSelf` ×2 | after the round trip the counts are doubled (nothing was removed on the way out) | every collection back to exactly its pre-transform contents | 712.18 |

Where cheap, prefer a **behavioral** assertion over a collection-membership one (per
`memory/conventions.md`'s full-dispatch rule) — e.g. test 9 can additionally assert a second spell
is castable again, test 10 asserts through `calculate_characteristics`. Keep at least one
collection-level assertion per family so the removal itself is pinned.

### 7.3 Pass-after regression guards only (explicitly labelled, with reasons)

| Test | Why it cannot be a fail-before probe |
|---|---|
| `test_transform_does_not_remove_other_sources_registrations` | Pre-fix the function removes nothing at all, so "does not over-remove" trivially passes. Its value is **post-fix**: it pins the "remove at most the registered count, first structural match" rule against a future bulk-purge refactor. Set up a *second* permanent registering into the same collections plus a Class-level-up `AdditionalLandPlaySource` on the same object (mirroring `resolution.rs:7460-7470`) and assert those survive. Label the doc comment "regression guard, not a probe". |
| `registration_and_deregistration_cover_the_same_ability_families` (§5.3) | A source-scan gate — it passes both before and after by construction once written against the finished code. Its value is future drift. Include the non-vacuity assertion so it cannot pass on a broken extractor. |
| `test_mdfc_back_face_self_replacement_still_unreachable` (optional) | Documents that MDFC back-face *play* is unimplemented (§8) so the four `Complete` MDFC lands' back-face replacements cannot fire either before or after. Pure documentation; keep only if cheap, and cite seed OOS-RS4-2. |

### 7.4 Existing tests that must stay green (watch these)

- `crates/engine/tests/mechanics_m_z/pb_os4b_face_aware_abilities.rs` — the whole file, especially
  `test_front_static_removed_on_transform` (`:740`) and
  `test_transform_there_and_back_restores_front_ability_set` (`:781`).
- `crates/engine/tests/mechanics_a_d/disturb.rs` — all tests.
- `crates/engine/tests/mechanics_m_z/saga_class.rs` — notably `:217-222` (direct
  `fire_saga_chapter_triggers` call) and `:111` (ETB lore counter). Step 3's live read must not
  change behavior for a non-transformed Saga.
- `crates/engine/tests/rules/replacement_effects.rs` — ~16 direct
  `apply_self_etb_from_definition` calls (`:3309`-`:4257`) and one
  `register_permanent_replacement_abilities` (`:4354`). All use non-transformed objects, so
  `effective_abilities(false) == abilities` and behavior is byte-identical. If any turn red, the
  `fizzle_object` guard is the suspect (a test that never added the object) — fix by ensuring the
  `unwrap_or(false)` path is taken, not by reverting the change.
- `crates/engine/tests/rules/trigger_doubling.rs`, `tests/rules/grant_flash.rs`,
  `tests/rules/etb_trigger_suppression.rs`, `tests/primitives/pb_k_land_drops.rs` — the four
  families with existing registration coverage.
- `cargo test --test core` — `ability_definition_registry`, `bare_lookup_ratchet`,
  `card_defs_fmt`, plus the new `face_dereg_parity`.

---

## 8. Card Definition Fixes: NONE (with the evidence)

**No card-def file is edited by PB-RS4.** Full DFC roster (`Grep "back_face: Some"` over
`crates/card-defs/src/defs/` → 15 files) cross-matched against the affected families:

| Card | Back-face families relevant to RS4 | Flip? |
|---|---|---|
| `bridgeworks_battle` (`Complete`) | back face `Replacement { WouldEnterBattlefield, EntersTapped, is_self }` (`:77-84`) | **No** — MDFC back-face *play* is unimplemented (below) |
| `sea_gate_restoration` (`Complete`) | back face `Replacement { .., EntersTappedUnlessPayLife(3), is_self }` (`:63-70`) | No — same |
| `revitalizing_repast` (`Complete`) | back face same shape (`:65-72`) | No — same |
| `disciple_of_freyalise` (`Complete`) | back face same shape (`:72-79`) | No — same |
| `fable_of_the_mirror_breaker` (`partial`) | front `SagaChapter` ×3; back activated only | **No flip** — integrity repair only (deviation #4); stays `partial` on its two unrelated blockers |
| `beloved_beggar` (`Complete`) | disturb DFC; back face is keywords only — no `Replacement`, none of the nine | No |
| `brutal_cathar`, `braided_net`, `hanweir_the_writhing_township`, `docent_of_perfection`, `bloodline_keeper`, `delver_of_secrets`, `legions_landing`, `growing_rites_of_itlimoc`, `thaumatic_compass` | none of `Replacement` / the nine on either face (grep over all 15 files for the ten variant names returns only the five rows above) | No |

### Correction to the record (worth writing down)

The PB-OS4b doc comment at `replacement.rs:1185-1187` asserts *"No roster DFC/craft/disturb back
face declares a `WouldEnterBattlefield` self-replacement."* **That is false as written** — four
`Complete` MDFC lands do. It is true only in the narrower sense that **MDFC back-face play is
unimplemented**: there is no `Mdfc`/face-selection path anywhere in `crates/engine/src` (grep for
`ModalDfc|modal_dfc|MDFC` hits only `crates/card-defs/`, and `rules/lands.rs` has no face concept
at all; e.g. `sejiri_shelter.rs:22-23` records "MDFC back face … is not authored — `back_face` is
None" for the un-authored cases). Step 2b's replacement comment must state the *narrow* truth,
not repeat the wide false one.

---

## 9. New Card Definitions

**None.** PB-RS4 authors no cards. All test DFCs are test-local mocks (§7.1).

---

## 10. Seeds to file (do NOT widen into these)

| ID | Finding | Class | Evidence |
|---|---|---|---|
| **OOS-RS4-1** | The **stack craft path** (`resolution.rs:7241-7296`) and **`Effect::ExileSourceAndReturnTransformed`** (`effects/mod.rs:4273-4351`) never call `register_permanent_replacement_abilities`, and the craft path also never calls `queue_carddef_etb_triggers`. `rules/engine.rs handle_craft` (`:1432-1453`) calls **none** of `apply_self_etb_from_definition` / `register_permanent_replacement_abilities` / `queue_carddef_etb_triggers`. A crafted-in permanent's ETB triggers and permanent replacement abilities never register — on **either** face. | correctness, latent (no `Complete` craft card on roster) | the call-site tables in §4.3/§4.4 |
| **OOS-RS4-2** | **MDFC back faces are unplayable.** Four `Complete` MDFC lands (`bridgeworks_battle`, `sea_gate_restoration`, `revitalizing_repast`, `disciple_of_freyalise`) author a fully-correct back-face land with an enters-tapped replacement that the engine can never reach, because there is no MDFC face-selection path in `casting.rs`/`lands.rs`. Invariant-#9 tension: they are marked `Complete` but half of each card is inert. | correctness / marker integrity | §8 |
| **OOS-RS4-3** | `apply_self_etb_from_definition`'s `starting_loyalty` read (`replacement.rs:1233`) is front-face-only. Already covered by **OOS-OS4-1 / queue item R10** — cross-reference rather than filing a duplicate. | capability | R10 row of the triage §3 table |

Also carried forward untouched (task fence): **OOS-RS3-1** (queue-time intervening-if, CR 603.4)
and **OOS-RS2-1** (`TurnFaceUp` unflattened cost).

---

## 11. Verification Checklist

- [ ] Step 0 probe printed `Some(true)` at `resolution.rs:1673` under the disturb test
- [ ] Probe tests written first and **recorded FAILING** against pre-fix HEAD
- [ ] `apply_self_etb_from_definition` face-aware; limitation comment **replaced**, not left
- [ ] `fire_saga_chapter_triggers` producer index namespace matches its three consumers
- [ ] `register_permanent_replacement_abilities` face-aware; comment replaced
- [ ] `turn_actions.rs` CR 714.3b sweep face-aware; agrees with `sba.rs:843`
- [ ] `deregister_face_statics` covers **all ten** families via `remove_one_registration`; doc comment fully rewritten (no surviving deferral prose)
- [ ] `face_dereg_parity.rs` gate written, registered in `tests/core/main.rs`, and non-vacuous
- [ ] All nine `ability_definition_registry.rs` `sites` arrays gained `crates/engine/src/rules/face.rs`
- [ ] `cargo test --test core ability_definition_registry` green
- [ ] `cargo test --test core bare_lookup_ratchet` green with **unchanged** ceilings
- [ ] `PROTOCOL_VERSION == 27`, `HASH_SCHEMA_VERSION == 63`, fingerprint unchanged
- [ ] `cargo build --workspace` (catches `view_model.rs` / `stack_view.rs`)
- [ ] `cargo test --all`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh`
- [ ] 0 coverage flips claimed; `tools/authoring-report.py` unchanged at 1,139/1,804
- [ ] Seeds OOS-RS4-1 / -2 filed; OOS-RS4-3 cross-referenced to OOS-OS4-1
- [ ] Close-out: OOS-OS4-2 flipped to **fully closed** in `CLAUDE.md` Current State,
      `memory/primitives/oos-retriage-plan-2026-07-18.md:579`,
      `memory/primitives/rider-seed-triage-2026-07-19.md` §2.4 + §3 R4 row + §5 banner,
      and `memory/workstream-state.md`

---

## 12. Risks & Edge Cases

1. **Highest risk — the live-read assumption at a call site the plan mis-read.** Mitigated by the
   Step 0 probe and by the two-writes-only proof (§2). If the probe disagrees, STOP and re-scope
   to an explicit parameter; do not patch around it.
2. **`ability_definition_registry` sites gate is a guaranteed red** on the first
   `cargo test --all` if Step 7 is skipped. Its failure message names the variant and the file,
   so it is self-diagnosing — but it is the single most likely "why won't this pass" moment.
3. **Over-removal in `remove_one_registration`.** A `retain(|e| e.source != obj_id)` shortcut
   would delete Class-level-up registrations (`resolution.rs:7447-7470`) that share the source
   ObjectId. The regression guard in §7.3 exists specifically for this; write it.
4. **`CdaModifyPowerToughness` asymmetry.** `Some(power) + None` registers ONE entry; both-`Some`
   registers TWO; both-`None` registers ZERO. A removal loop that always tries two will remove an
   unrelated effect. Build the same `modifications` vector the registration builds.
5. **Borrow-checker churn in Step 2c/4c.** `def` is an `Arc`-held `CardDefinition` while `state`
   is `&mut`; `effective_abilities` borrows from `def`, not from `state`, so the existing
   `state.replacement_effects.push_back(..)` inside the loop stays legal. The identical pattern
   already compiles at `replacement.rs:2095`.
6. **Item-type change.** `&def.abilities` iterates `&AbilityDefinition`; a slice from
   `effective_abilities(b)` also iterates `&AbilityDefinition`. No `**` deref changes should be
   needed; if clippy complains about `needless_borrow`, fix the borrow, do not add `#[allow]`.
7. **`fizzle_object` vs `expect_object`.** Using `expect_object` in
   `register_permanent_replacement_abilities` would `debug_assert!`-panic whenever an ETB
   replacement legitimately removed the entering permanent before this call. Use `fizzle_object`.
8. **Deviation #4 scope creep.** Fix exactly the two `def.abilities` reads named in §3.4
   (`turn_actions.rs:380`, `replacement.rs:1293`). Do **not** start auditing every other
   `def.abilities` read in the engine — that is a fresh sweep, and further finds are seeds, not
   scope.
9. **Order-of-operations in `apply_face_change`** (`face.rs:71-102`) must not change: deregister
   OLD (reads old face) → flip → rebuild Channel-A → register NEW. The nine new removals inherit
   this ordering for free because they run inside step 1.
10. **`state.timestamp_counter` is not rewound** by deregistration; a transform-there-and-back
    cycle re-registers with a later timestamp. That is correct per CR 613.7 (a new continuous
    effect gets a new timestamp) — do not "fix" it. Test 13 must therefore assert on
    *membership/count*, not on timestamps or `EffectId`s.
11. **Aura/attachment ordering** (`memory/gotchas-infra.md`; `resolution.rs:1618-1621`):
    attachment is set before `register_static_continuous_effects`. Nothing in RS4 moves that
    boundary; do not.
12. **Golden scripts (SR-9c)** should be untouched — no roster behavior changes. If any of the 210
    approved scripts move, something in Step 2/4/5 changed non-transformed behavior, which would
    mean `effective_abilities(false) != abilities` somewhere. Investigate; do not re-bless the
    script.
