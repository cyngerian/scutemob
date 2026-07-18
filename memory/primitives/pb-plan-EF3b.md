# Primitive Batch Plan: PB-EF3b — Granted keyword-triggers fire (Melee / Battle Cry / Annihilator)

**Generated**: 2026-07-18
**Primitive**: When a *trigger-bearing* keyword (Melee, Battle Cry, Annihilator N) is granted by a
continuous effect (`LayerModification::AddKeyword`), synthesize its derived `TriggeredAbilityDef`
into the object's **layer-resolved** characteristics so the trigger actually fires. Today only
**printed** keywords get the derived trigger (built inline in `builder.rs`); a granted trigger-keyword
lands in `Characteristics.keywords` and its trigger is a silent no-op.
**CR Rules**: 702.121 (Melee), 702.91 (Battle Cry), 702.86 (Annihilator), 613.1f (Layer 6 ability grants), 603.2 (attack triggers), 400.7 / 113.7a (LKI at trigger time)
**Cards affected**: 2 (0 existing fixes + 2 new authored) — **Adriana Complete (+1 clean coverage)**, **Skyhunter Strike Force partial (blocked, truthfully marked)**
**Dependencies**: none (all types pre-exist: `LayerModification::AddKeyword`, `EffectFilter::OtherCreaturesYouControl`, `AbilityDefinition::Static`, `PendingTriggerKind::Melee`)
**Deferred items from prior PBs**: closes finding **EF-W-MISS-3** (`memory/card-authoring/w-miss-roster-2026-07-17.md` L174). No other carry-forward.

**TODO sweep** (roster-recall gate): grepped `crates/card-defs/src/defs/` for pre-existing TODOs
naming granted-keyword-trigger / AddKeyword-trigger / Melee-grant / Annihilator-grant. The only
self-identified references are the two roster cards (neither file exists yet) plus documentary notes
in `tyvar_kell.rs` (mana-ability grant, different primitive — out of scope). **TODO sweep: 0
additional cards** beyond the briefed roster.

---

## Key decisions (read first)

1. **Schema bump: NO** (neither `PROTOCOL_VERSION`/`PROTOCOL_SCHEMA_FINGERPRINT` nor
   `HASH_SCHEMA_VERSION`). This is pure runtime synthesis into the *computed* `Characteristics`
   returned by `calculate_characteristics` (never stored, never hashed) plus a raw→resolved read
   change. No new type, no new enum variant, no new field, no serde change. The Adriana DSL uses the
   existing `LayerModification::AddKeyword`. **The gates prove it**: leave every version const
   untouched; `tests/protocol_schema.rs` and the hash-sentinel tests must stay green with no edit. If
   either reddens, STOP — something changed shape unexpectedly (it should not).

2. **Skyhunter Strike Force: BLOCKED → author `partial`, truthfully marked.** Its Lieutenant clause
   ("As long as you control your commander, other creatures you control have melee") needs a
   *"you control your commander"* condition on a continuous-effect grant. **No such primitive
   exists**: `Condition` (card_definition.rs:3466) has no commander variant, and `TargetFilter`
   (card_definition.rs:2858) has no `is_commander` field, so `Condition::YouControlPermanent(filter)`
   cannot express it either. Author Skyhunter modeling only **Flying + printed Melee** (both work),
   omit the Lieutenant anthem, mark `Completeness::partial` citing the real blocker. **Do NOT invent
   the condition** — out of scope for this correctness PB. File **OOS-EF3b-1** (Lieutenant / "control
   your commander" continuous-grant condition).

3. **Adriana, Captain of the Guard: authorable → Complete.** Printed Melee (already works) + an
   unconditional static anthem "Other creatures you control have melee." The anthem is the exact
   `stromkirk_captain.rs` shape with `EffectFilter::OtherCreaturesYouControl` and
   `LayerModification::AddKeyword(KeywordAbility::Melee)`. This is the batch's clean-coverage yield.

---

## CR Rule Text

- **702.121a** Melee is a triggered ability. "Melee" means "Whenever this creature attacks, it gets
  +1/+1 until end of turn for each opponent you attacked with a creature this combat."
- **702.121b** If a creature has multiple instances of melee, each triggers separately.
- **702.91a** Battle cry is a triggered ability. "Battle cry" means "Whenever this creature attacks,
  each other attacking creature gets +1/+0 until end of turn."
- **702.91b** If a creature has multiple instances of battle cry, each triggers separately.
- **702.86a** Annihilator is a triggered ability. "Annihilator N" means "Whenever this creature
  attacks, defending player sacrifices N permanents."
- **702.86b** If a creature has multiple instances of annihilator, each triggers separately.

**Known modeling limitation (set model vs. 702.x.b).** `Characteristics.keywords` is
`OrdSet<KeywordAbility>` (`game_object.rs:768`, `card_definition.rs:3736`). Redundant instances
collapse to ONE set entry, so printed Melee + granted Melee = one Melee. The "each instance triggers
separately" clauses (702.121b / 702.91b / 702.86b) are **not representable** under this set model;
the engine already treats keyword redundancy this way. The reconciliation therefore produces
**exactly one** derived trigger per distinct keyword entry (Annihilator(2) and Annihilator(3) are
*distinct* entries and each get one — different N, different description). The **no-double-fire
decoy test** pins that printed+granted Melee fires exactly once. Document this limitation in the
reconciliation code comment with the CR citations.

---

## Verified recon (source-confirmed)

- **Only synthesis site is `builder.rs`.** `state/builder.rs` build loop (L386 `for kw in
  spec.keywords.iter()`) synthesizes the derived `TriggeredAbilityDef` for each **printed** keyword:
  Annihilator **L483–505**, Battle Cry **L511–539**, Melee **L609–627** (plus Dethrone, Training,
  Exalted, Prowess, Enlist, Persist, etc.). `make_token` (`effects/mod.rs:7352`) builds
  `Characteristics` with keywords but **no** derived triggers — so a Melee *token* is silent today
  too (reconciliation fixes this as a bonus).
- **`layers.rs:1165`** `LayerModification::AddKeyword(kw) => { chars.keywords.insert(kw.clone()); }` —
  inserts the keyword and nothing else. This is the bug locus for granted keywords.
- **Collection reads RESOLVED chars.** `collect_triggers_for_event` (`abilities.rs` L6112) at
  **L6149–6150** reads `expect_characteristics(state, obj_id)` (post-layers) and iterates
  `resolved_chars.triggered_abilities`; the push at **L6384–6393** sets
  `embedded_effect: trigger_def.effect.clone()` and `ability_index: idx` (index into **resolved**),
  kind `Normal`. => Appending the synthesized def to RESOLVED chars makes **Battle Cry** (`ForEach`)
  and **Annihilator** (`SacrificePermanents`) resolve via their embedded effect with no further
  work. **Melee** has `effect: None` and needs kind-tagging.
- **Annihilator defending-player is set independently.** `AttackersDeclared` handler
  (`abilities.rs` L3564–3582): after `collect_triggers_for_event(SelfAttacks, Some(attacker))`, the
  loop at **L3580–3582** stamps `t.defending_player_id = defending_player` on every trigger in the
  batch (`pre_len..`), resolved from the `AttackTarget` (CR 508.5). This is **independent of
  ability_index** and works for granted Annihilator with no change.
- **The Melee/Myriad/Provoke tags read RAW chars at a RESOLVED index (the bug).** Three post-collect
  loops in the same handler tag by reading `state.fizzle_object(t.source)` (a **live, non-resolved**
  `self.objects.get`, `diagnostics.rs:373`) then `obj.characteristics.triggered_abilities.get(t.ability_index)`:
  - Myriad **L3590–3601**
  - Provoke **L3613–3651**
  - Melee **L3657–3668**
  For a **granted-only** trigger-keyword the derived def exists only in RESOLVED chars (appended at an
  index ≥ base length); the RAW lookup returns `None` → tag skipped → the trigger fizzles as a plain
  `effect: None` no-op. For PRINTED keywords raw==resolved at that index (base defs keep their
  indices; reconciliation and merge-integration only **append**), so switching to resolved chars is a
  no-op for them.
- **Melee kind → stack.** `flush_pending_triggers` (`abilities.rs` L7518) maps
  `PendingTriggerKind::Melee` → `StackObjectKind::KeywordTrigger { keyword: Melee, data: Simple }`;
  resolution (`resolution.rs` L3648) computes +count/+count from live combat state. Unchanged.
- **Reconciliation insertion point.** `calculate_characteristics` (`layers.rs` L35) starts from
  `obj.characteristics.clone()` (L41), runs the full layer loop (L48–401, incl. `EffectLayer::Ability`
  = Layer 6), then the merged-component ability integration (L413–435), then `Some(chars)` (L436).
  Reconcile at **L435→436 boundary** (after the merge `if` block, before `Some(chars)`) — this is
  after all of Layer 6 (keyword add/remove) and after merge integration, so it sees the FINAL keyword
  set. `RemoveAllAbilities`/Humility (L1173–1182) clears `chars.keywords` → reconciliation iterates
  nothing → appends nothing (correct).
- **No SelfEntersBattlefield/Panharmonicon interaction.** All three derived defs are
  `trigger_on: SelfAttacks`. `collect_triggers_for_event(SelfEntersBattlefield, …)` never matches
  them, and Panharmonicon doubles only ETB triggers. Confirmed no spurious ETB / no doubling.
- **Idempotency.** `calculate_characteristics` recomputes from `obj.characteristics.clone()` every
  call and never writes back, so appends never accumulate across calls.

---

## Engine Changes

### Change 1: Shared helper `derived_attack_trigger_for_keyword`

**File**: `crates/engine/src/state/builder.rs`
**Action**: Add a `pub(crate)` free function (top-level in the module, near the build loop):

```rust
/// CR 702.86a / 702.91a / 702.121a: the derived `TriggeredAbilityDef` for a
/// trigger-bearing attack keyword. Single source of truth shared by `builder.rs`
/// (printed keywords) and `layers::calculate_characteristics` (granted keywords)
/// so the two never drift. Returns `None` for keywords that need no derived
/// attack trigger. Descriptions are load-bearing: the layer reconciliation
/// dedups by exact description equality, and the AttackersDeclared Melee tag
/// matches on `description.starts_with("Melee")`.
pub(crate) fn derived_attack_trigger_for_keyword(
    kw: &KeywordAbility,
) -> Option<TriggeredAbilityDef> {
    match kw {
        KeywordAbility::Annihilator(n) => Some(/* exact def now at builder.rs L484–504 */),
        KeywordAbility::BattleCry       => Some(/* exact def now at builder.rs L512–538 */),
        KeywordAbility::Melee           => Some(/* exact def now at builder.rs L610–626 */),
        _ => None,
    }
}
```

Move the three inline literals **verbatim** into the match arms (Annihilator keeps its
`format!("Annihilator {n} (CR 702.86a): …")`; Melee keeps `effect: None`; Battle Cry keeps its
`ForEach { over: EachOtherAttackingCreature, … }`). Do **not** alter any field, description string,
`trigger_on`, or effect — dedup and tagging both depend on byte-identical output.

**Pattern**: the three literals currently at `builder.rs` L483–505 / L511–539 / L609–627.

### Change 2: Route builder's printed synthesis through the helper

**File**: `crates/engine/src/state/builder.rs`
**Action**: In the `for kw in spec.keywords.iter()` loop, **delete** the three `if matches!(kw,
Annihilator|BattleCry|Melee) { triggered_abilities.push(TriggeredAbilityDef { … }); }` blocks
(L483–505, L511–539, L609–627) and replace with a single call, placed anywhere in the loop body:

```rust
if let Some(def) = derived_attack_trigger_for_keyword(kw) {
    triggered_abilities.push(def);
}
```

Leave all other keyword blocks (Dethrone, Training, Exalted, Prowess, Ward, Enlist, Persist, Rampage,
…) inline and untouched. Net behavior for printed Melee/BattleCry/Annihilator is identical.

### Change 3: Post-layer reconciliation in `calculate_characteristics`

**File**: `crates/engine/src/rules/layers.rs`
**Action**: Insert at the **L435→436 boundary** (after the `if obj.zone == Battlefield &&
merged_components.len() > 1 { … }` block, immediately before `Some(chars)`):

```rust
// PB-EF3b (CR 702.86a/702.91a/702.121a, 613.1f): a trigger-bearing keyword GRANTED by a
// continuous effect (LayerModification::AddKeyword) inserts into `chars.keywords` but carries
// no derived TriggeredAbilityDef, so its trigger would be a silent no-op. Synthesize it here,
// AFTER all layers (incl. Layer 6 add/remove) and merge integration, so the derived trigger
// exists in the RESOLVED characteristics that collect_triggers_for_event reads.
//
// Keyword model is a SET (OrdSet), so printed+granted collapse to one entry (CR 702.x.b "each
// instance triggers separately" is not representable — known limitation). Dedup by exact
// description equality against the shared helper's output so a PRINTED derived def (already in
// base chars via builder.rs) is not duplicated. Humility/RemoveAllAbilities empties
// chars.keywords, so nothing is appended (correct). These are SelfAttacks triggers only — no
// ETB / Panharmonicon interaction.
for kw in chars.keywords.iter() {
    if let Some(def) = crate::state::builder::derived_attack_trigger_for_keyword(kw) {
        let already = chars
            .triggered_abilities
            .iter()
            .any(|t| t.description == def.description);
        if !already {
            chars.triggered_abilities.push(def);
        }
    }
}
```

**Dedup predicate**: exact `description` string equality. Because `builder.rs` now pushes the SAME
helper output into base chars, the printed def's description is byte-identical to the reconciliation
candidate. Verified cases:
- **printed-only** (builder-built): base has the def → `already == true` → skip (no change).
- **granted-only** (anthem / token / any non-builder path): base lacks it → append → **fires**.
- **printed + granted** (same keyword): one set entry, base has the def → skip → **no double-fire**.
- **Annihilator(2) printed + Annihilator(3) granted**: distinct set entries, distinct descriptions →
  both present → both fire (correct; genuinely different keyword instances).

(Borrow note: the loop reads `chars.keywords` while the `if !already` branch mutates
`chars.triggered_abilities` — different fields, so collect the keywords into a small `Vec` first if
the borrow checker objects, e.g. `let kws: Vec<_> = chars.keywords.iter().cloned().collect();`.)

### Change 4: Fix the raw→resolved tag reads (Melee + Myriad + Provoke)

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the `AttackersDeclared` SelfAttacks post-processing, replace the RAW
`state.fizzle_object(t.source)` + `obj.characteristics.triggered_abilities.get(t.ability_index)`
lookup with the **layer-resolved, None-tolerant** read at all three tag sites:

```rust
if let Some(chars) = crate::rules::layers::calculate_characteristics(state, t.source) {
    if let Some(ta) = chars.triggered_abilities.get(t.ability_index) {
        // …existing tag body unchanged…
    }
}
```

Sites: **Melee L3657–3668** (the load-bearing fix for granted Melee), **Myriad L3590–3601**,
**Provoke L3613–3651**.

Use `calculate_characteristics` (returns `Option`, no panic) — NOT `expect_characteristics` — to
preserve the existing fizzle-tolerant `if let Some(…)` structure (`fizzle_object` returned `Option`
and the attacker is live here, but keep it None-tolerant to avoid a debug panic if the source ever
left the batch, CR 113.7a). `t.source` is the trigger source and `t.ability_index` indexes resolved
chars (set at the L6389 push), so this is index-correct.

**Myriad/Provoke audit verdict — fix all three (W3-LC compliance).** Reading raw base characteristics
at a resolved index is simply wrong (violates the W3-LC "never read `obj.characteristics` raw on the
battlefield" rule) and is harmless for printed keywords (raw==resolved index). Myriad and Provoke ARE
grantable keywords in principle, so fixing them is defense-in-depth and consistency. **Caveat**: the
shared helper synthesizes only Melee/BattleCry/Annihilator, so a *granted* Myriad/Provoke would still
not produce a derived def and would not fire — that is a separate, out-of-scope gap. The tag fix
alone does not make granted Myriad/Provoke fire; it only makes the tags correct for the reconciled
keywords and future-proofs the pattern. Note this in the OOS seed (OOS-EF3b-2: extend the helper to
the full builder-synthesized keyword-trigger set — Myriad, Provoke, Dethrone, Training, Exalted,
Prowess, Persist, Rampage — if a card ever grants one).

### Change 5: Exhaustive-match sites — NONE

No new enum variant, struct field, or `StackObjectKind`/`KeywordAbility`/`PendingTriggerKind`
variant is introduced. **No** `state/hash.rs`, `tools/replay-viewer/src/view_model.rs`, or
`tools/tui/src/play/panels/stack_view.rs` match arms need adding. `PendingTriggerKind::Melee` and
`StackObjectKind::KeywordTrigger { keyword: Melee, … }` already exist and are handled everywhere.
Confirm via `cargo build --workspace` (the exhaustiveness gate) after the impl phase regardless.

---

## Card Definition Fixes / New Cards

### `adriana_captain_of_the_guard.rs` — NEW, author **Complete**
**Oracle text**: "Melee (Whenever this creature attacks, it gets +1/+1 until end of turn for each
opponent you attacked this combat.) / Other creatures you control have melee. (If a creature has
multiple instances of melee, each triggers separately.)" — {3}{R}{W} Legendary Creature — Human
Knight, 4/4.
**Sketch** (mirror `stromkirk_captain.rs`):
```rust
CardDefinition {
    card_id: cid("adriana-captain-of-the-guard"),
    name: "Adriana, Captain of the Guard",
    mana_cost: Some(ManaCost { generic: 3, red: 1, white: 1, ..Default::default() }),
    types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Knight"]),
    power: Some(4), toughness: Some(4),
    abilities: vec![
        AbilityDefinition::Keyword(KeywordAbility::Melee),            // printed self-Melee (already works)
        AbilityDefinition::Static {                                    // "Other creatures you control have melee."
            continuous_effect: ContinuousEffectDef {
                layer: EffectLayer::Ability,
                modification: LayerModification::AddKeyword(KeywordAbility::Melee),
                filter: EffectFilter::OtherCreaturesYouControl,
                duration: EffectDuration::WhileSourceOnBattlefield,
                condition: None,
            },
        },
    ],
    ..Default::default()
}
```
Completeness: default (`Complete`). This is the batch's +1 clean-coverage yield.

### `skyhunter_strike_force.rs` — NEW, author **partial** (blocked, truthful marker)
**Oracle text**: "Flying / Melee (…) / Lieutenant — As long as you control your commander, other
creatures you control have melee." — {2}{W} Creature — Cat Knight, 2/2.
**Sketch**: model `Flying` + printed `Melee` only; OMIT the Lieutenant anthem.
```rust
abilities: vec![
    AbilityDefinition::Keyword(KeywordAbility::Flying),
    AbilityDefinition::Keyword(KeywordAbility::Melee),
    // Lieutenant clause ENGINE-BLOCKED (see completeness note) — omitted, not modeled wrong.
],
completeness: Completeness::partial(
    "Lieutenant clause 'As long as you control your commander, other creatures you control have \
     melee' is unrepresentable: no Condition variant nor TargetFilter field expresses 'you control \
     your commander' for a continuous-effect grant (ContinuousEffectDef.condition). Flying + printed \
     Melee are modeled and correct; the anthem is omitted (not wrong game state). Blocker filed as \
     OOS-EF3b-1.",
),
```
Add a top-of-file `// ENGINE-BLOCKED (Lieutenant grant): …` comment matching the `tyvar_kell.rs`
documentation style.

**Roster note**: Both cards added via the PB brief roster; verified against oracle text via MCP
`lookup_card`. Olivia, Crimson Bride (also in the EF-W-MISS-3 group) grants a *bespoke* trigger, not
a keyword — explicitly out of scope, do not author here.

---

## Unit Tests

**File**: `crates/engine/tests/primitives/pb_ef3b_granted_keyword_triggers.rs` (new)
**Register**: add `mod pb_ef3b_granted_keyword_triggers;` to
`crates/engine/tests/primitives/main.rs` (alphabetical-ish, after `pb_ef3_attack_trigger_targets`).
**Patterns**: attacker setup + pass-priority + P/T assertion from `mechanics_m_z/melee.rs`
(`find_object`, `pass_all`, `at_step(Step::DeclareAttackers)`, `calculate_characteristics`); granted
keyword via `GameStateBuilder::add_continuous_effect(ContinuousEffect { layer: EffectLayer::Ability,
modification: LayerModification::AddKeyword(Melee|BattleCry|Annihilator(n)), filter:
EffectFilter::OtherCreaturesYouControl (or a specific creature), duration, source, .. })` following
the `indef_effect` helper in `mechanics_a_d/changeling.rs`.

Tests to write (each cites CR; each decoy must be provably non-vacuous — revert the fix and watch it
fail):

- `test_ef3b_granted_melee_via_anthem_fires_and_pumps` — **CR 702.121a**. Two creatures you control
  (a vanilla 2/2 + a Melee-granting source), attack ≥1 opponent; the vanilla creature's granted Melee
  fires and it gets +1/+1 per opponent attacked. Non-vacuity: without Change 3+4 the granted creature
  stays 2/2.
- `test_ef3b_granted_melee_multiplayer_per_opponent` — **CR 702.121a**, 4-player. Attack 3 distinct
  opponents (spread attackers); the granted-Melee creature gets +3/+3 (bonus counts distinct opponents
  attacked with creatures, ruling 2016-08-23).
- `test_ef3b_granted_battle_cry_via_anthem` — **CR 702.91a**. A granted-Battle-Cry attacker fires;
  each OTHER attacking creature gets +1/+0. Resolves via the embedded `ForEach` effect (Change 3
  only; no tagging needed). Non-vacuity: without reconciliation, other attackers are unbuffed.
- `test_ef3b_granted_annihilator_via_anthem` — **CR 702.86a**. A granted-Annihilator(1) attacker
  attacks a player; the defending player sacrifices 1 permanent. Verifies `defending_player_id` is
  correctly stamped (L3580) for a granted trigger and the embedded `SacrificePermanents` resolves.
- `test_ef3b_no_double_fire_printed_plus_granted_melee` — **CR 702.121b / set model**. A creature
  with **printed** Melee that is ALSO granted Melee by an anthem attacks 1 opponent → gets **+1/+1
  exactly once** (3/3, not 4/4). Decoy pins the description-dedup: if dedup is removed, the creature
  fires twice → +2/+2. Non-vacuity: remove the `already` check and assert this reddens.
- `test_ef3b_printed_melee_unchanged_regression` — **CR 702.121a**. A builder-built printed-Melee
  creature (no anthem) still fires exactly once (guards against reconciliation double-appending onto
  builder-synthesized defs).
- `test_ef3b_adriana_grants_melee_to_other_creatures` — **card integration, CR 702.121a**. Build
  Adriana + a vanilla 2/2 on the battlefield (registry with Adriana's def; `register_static_continuous_effects`
  runs at ETB or use builder placement + static registration), attack 1 opponent with both: the
  vanilla creature's granted Melee fires (+1/+1 → 3/3) AND Adriana's own printed Melee fires (+1/+1 →
  5/5). Asserts the anthem `EffectFilter::OtherCreaturesYouControl` excludes Adriana from a *second*
  (granted) Melee on herself (dedup + "other" filter).
- `test_ef3b_humility_strips_granted_melee` — **CR 613.1f / layer removal**. With a
  `RemoveAllAbilities` continuous effect active over the creature, the granted Melee is absent from
  resolved keywords → no derived trigger appended → Melee does not fire. Guards the
  Humility/empty-keyword path.

**Existing suites must stay green unchanged**: `mechanics_m_z/melee.rs`, `mechanics_a_d/battle_cry.rs`,
`mechanics_a_d/annihilator.rs` all use builder-built PRINTED keywords → dedup skip → identical
behavior. Run them as the regression check.

---

## Verification Checklist

- [ ] Helper `derived_attack_trigger_for_keyword` added; builder's three inline blocks replaced by one call (byte-identical defs)
- [ ] Reconciliation appended in `calculate_characteristics` at the L435→436 boundary with description-dedup
- [ ] All three tag reads (Melee/Myriad/Provoke) switched to `calculate_characteristics` (resolved, None-tolerant)
- [ ] `cargo check -p mtg-engine` compiles
- [ ] Adriana authored **Complete**; Skyhunter authored **partial** with truthful marker + OOS-EF3b-1 note
- [ ] `cargo test --all` passes (incl. existing melee/battle_cry/annihilator suites unchanged, new pb_ef3b suite, `core card_defs_fmt`)
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `cargo build --workspace` (exhaustive-match / GameState-seal gate) green
- [ ] `tools/check-defs-fmt.sh` clean (two new def files)
- [ ] **No version bump**: `PROTOCOL_VERSION`, `PROTOCOL_SCHEMA_FINGERPRINT`, `HASH_SCHEMA_VERSION` all unedited; `tests/protocol_schema.rs` + hash sentinels green without change (this is the proof no schema moved)
- [ ] OOS-EF3b-1 (Lieutenant/"control your commander" grant condition) + OOS-EF3b-2 (extend helper to full keyword-trigger set for granted Myriad/Provoke/etc.) filed
- [ ] No TODOs / partials-with-wrong-state in the two new defs (Skyhunter partial is an *omission* with truthful marker, not wrong game state — allowed)

---

## Risks & Edge Cases

- **PRIMARY — latent bug now fixed elsewhere may shift a hash.** Reconciliation appends the derived
  trigger for ANY object whose resolved keywords include a trigger-keyword but whose base lacks the
  def — this covers not just anthem grants but **Melee/BattleCry/Annihilator tokens** (`make_token`
  never synthesized them) and any non-builder construction path. If an existing golden script /
  state-hash test involves such an object and asserted the buggy *silent* state, its hash changes.
  **Treat a changed assertion as a bug FIX** (the trigger SHOULD have fired — cite CR 702.x) and
  update it, do NOT suppress. Search the corpus for token/granted trigger-keyword scenarios during
  impl; expectation is zero (that's why the bug survived), but verify.
- **Description-equality dedup is stringly-typed.** It is correct only because `builder.rs` now emits
  the SAME helper output that reconciliation compares against — any future edit to a description in
  one place without the other would silently double- or non-fire. The single-helper design prevents
  drift; the `test_ef3b_no_double_fire_printed_plus_granted_melee` and
  `test_ef3b_printed_melee_unchanged_regression` decoys pin both directions.
- **Minor perf.** Reconciliation runs on every `calculate_characteristics` (hot path): O(keywords ×
  triggered_abilities) with one `String` alloc per trigger-keyword present (helper builds the def to
  read its description). Both sets are tiny (<~10); negligible. If a bench regresses, gate the loop
  on `chars.keywords` containing any of the three (cheap `.contains`) before building defs. Do not
  prematurely optimize.
- **Borrow checker** on the reconciliation loop (read `chars.keywords`, mutate
  `chars.triggered_abilities`): collect keywords into a `Vec` first if needed (noted in Change 3).
- **DFC back-face keywords** (`layers.rs` L114–122 clears keywords and re-derives from back-face
  abilities but does not synthesize derived triggers) — reconciliation now also fixes a transformed
  DFC that gains a trigger-keyword on its back face. Correct and in-spirit; note it but no card in
  this batch exercises it.
- **Skyhunter is intentionally incomplete.** Do not "make it Complete" by substituting an
  unconditional anthem for the Lieutenant clause — that would ship wrong game state (grants melee even
  without controlling your commander). Partial + truthful marker is the correct W6 outcome.
