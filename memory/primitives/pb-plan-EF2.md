# PB-EF2 Plan — `CreateToken` player-scoped recipient (fix swan_song, EF-W-MISS-1)

- **Task**: scutemob-102
- **Branch**: feat/pb-ef2-createtoken-player-scoped-recipient-fix-swansong-toke
- **Finding**: EF-W-MISS-1 (HIGH — latent legal-but-wrong in a `Complete` def).
  `Effect::CreateToken` always mints for the resolving effect's controller (`ctx.controller`).
  Swan Song / An Offer You Can't Refuse say "**Its controller** creates …" — the *countered
  spell's* controller, not the caster.
- **Source**: `memory/card-authoring/w-miss-engine-findings-2026-07-17.md` (EF-W-MISS-1),
  `memory/primitives/ef-batch-plan-2026-07-17.md` (§2 PB-EF2).

---

## DESIGN DECISION — recipient lives on `TokenSpec`, not on the `Effect::CreateToken` variant

The ESM brief says "add `recipient: PlayerTarget` to `Effect::CreateToken`". We place the
field on **`TokenSpec`** (which is the payload `Effect::CreateToken { spec: TokenSpec }`
carries) instead of as a second sibling field on the variant. Rationale:

1. **AC 4861's binding constraint is "all existing users unchanged — prove with the full
   suite."** There are **201** `Effect::CreateToken { … }` construction sites (160 in
   `crates/card-defs/src/defs/`, the rest in engine/tests/dungeon). Every one builds its
   `TokenSpec` via `..Default::default()` or a helper fn (`treasure_token_spec`,
   `food_token_spec`, `clue_token_spec`, …) that itself ends in `..Default::default()`.
   A `TokenSpec.recipient: PlayerTarget` defaulting to `Controller` therefore leaves **all
   201 sites literally unchanged** — zero diff, zero rustfmt/`check-defs-fmt` risk. A sibling
   field on the enum variant cannot take a per-field default (Rust has no functional-update
   for enum-variant construction), so it would force 201 mechanical edits — the opposite of
   "unchanged."
2. **Semantically correct.** `TokenSpec`'s doc comment is literally "Everything needed to
   create a token (CR 111)", and its `mana_color` field already documents "all created under
   the controller's control." The recipient (which player creates/controls the token) is part
   of "everything needed to create a token." It is not a stretch.
3. `Effect::CreateTokenAndAttachSource` also carries a `TokenSpec`; default `Controller`
   preserves Living Weapon behaviour and gives that path recipient support for free.

**This deviation is flagged to the coordinator in an ESM task comment.** If they truly want a
sibling variant field, it is a follow-up; the functional ACs (ControllerOfCounteredSpell
resolves, doubling keys off recipient, swan_song fixed + decoy) are all satisfied by the
TokenSpec placement.

---

## Step 1 — `PlayerTarget` gains two variants (card-types)

`crates/card-types/src/cards/card_definition.rs` (enum `PlayerTarget` ~line 2480). Add:

```rust
/// The controller of the spell/ability that this effect countered earlier in the
/// same resolution. CR 701.5g / "its controller creates …" (Swan Song, An Offer You
/// Can't Refuse). Captured by `Effect::CounterSpell` into
/// `EffectContext::countered_spell_controller` when a valid target spell is found —
/// captured even if the spell can't be countered (An Offer ruling 2022-04-29: an
/// uncounterable-but-legal target's controller still creates the tokens).
ControllerOfCounteredSpell,
/// The controller of the object that triggered this ability (attack/ETB/etc.).
/// Resolved from `EffectContext::triggering_creature_id`'s controller, falling back
/// to `triggering_player`, then `controller`.
ControllerOfTriggeringObject,
```

## Step 2 — `TokenSpec` gains `recipient: PlayerTarget` (card-types)

`crates/card-types/src/cards/card_definition.rs` — `pub struct TokenSpec` (~line 3698). Add
LAST field so existing positional/`..Default` construction is undisturbed:

```rust
/// CR 111.1 / CR 608.2h: which player creates (and thus controls) the token(s).
/// Defaults to `PlayerTarget::Controller` — the historical behaviour of every existing
/// user. Set to `ControllerOfCounteredSpell` for "its controller creates …" cards
/// (Swan Song, An Offer You Can't Refuse). Resolved via `resolve_player_target_list`,
/// so it may name multiple players (`EachPlayer` / `EachOpponent`), each receiving the
/// (post-replacement) count.
#[serde(default)]
pub recipient: PlayerTarget,
```

`PlayerTarget` must impl `Default` → `Controller`. Add `#[derive(Default)]`-compatible
`impl Default for PlayerTarget { fn default() -> Self { PlayerTarget::Controller } }` (or a
`#[default]` attr on `Controller` if the derive is added — either is fine; prefer an explicit
`impl` to avoid touching the derive list). Update `impl Default for TokenSpec` (~line 3743) to
set `recipient: PlayerTarget::Controller`.

## Step 3 — `EffectContext` captures the countered spell's controller (engine)

`crates/engine/src/effects/mod.rs`:
- Add field to `struct EffectContext` (~line 48):
  `pub countered_spell_controller: Option<crate::state::PlayerId>,` with a doc comment citing
  EF-W-MISS-1 / An Offer ruling.
- Initialize `None` in `EffectContext::new` (~line 157) **and every other constructor / literal
  of `EffectContext`** (grep `EffectContext {` and `..EffectContext` — there may be builders in
  resolution.rs). `cargo build` will list any missed one.
- In the `Effect::CounterSpell` arm (~line 1918): after `let pos = …` resolves to `Some(pos)`,
  set `ctx.countered_spell_controller = Some(state.stack_objects[pos].controller);` **before**
  the `cant_be_countered` `continue` (An Offer ruling: uncounterable-but-legal target still
  triggers "its controller creates"). `ctx` must be `&mut` in that arm — confirm the executor
  signature already threads `ctx: &mut EffectContext` (it does for `last_created_permanent`).

## Step 4 — resolve the two new `PlayerTarget` variants (engine)

`fn resolve_player_target_list` (`effects/mod.rs` ~line 6295). Add arms:

```rust
PlayerTarget::ControllerOfCounteredSpell => ctx
    .countered_spell_controller
    .filter(|p| state.players.get(p).map(|ps| !ps.has_lost).unwrap_or(false))
    .into_iter()
    .collect(),
PlayerTarget::ControllerOfTriggeringObject => {
    let p = ctx
        .triggering_creature_id
        .and_then(|id| state.objects.get(&id).map(|o| o.controller))
        .or(ctx.triggering_player)
        .unwrap_or(ctx.controller);
    if state.players.get(&p).map(|ps| !ps.has_lost).unwrap_or(false) {
        vec![p]
    } else {
        vec![]
    }
}
```

## Step 5 — thread recipient through the `CreateToken` executor + token doubling (engine)

`Effect::CreateToken { spec }` arm (`effects/mod.rs` ~line 666). Today it uses `ctx.controller`
for: `apply_token_creation_replacement`, `make_token`, the `TokenCreated`/`PermanentEnteredBattlefield`
events. Rewrite to loop over resolved recipients:

```rust
Effect::CreateToken { spec } => {
    let raw_count = resolve_amount(state, &spec.count, ctx);
    let resolved_count = raw_count.max(0) as u32;
    let recipients = resolve_player_target_list(state, &spec.recipient, ctx);
    // enters_attacking target is a property of the SOURCE creature, recipient-independent.
    let attack_target = if spec.enters_attacking {
        state.combat.as_ref().and_then(|c| c.attackers.get(&ctx.source).cloned())
    } else { None };
    for recipient in recipients {
        // CR 614.1: token-creation replacements (Doubling Season, Parallel Lives) are
        // per-player — apply the RECIPIENT's replacements, not the controller's.
        let (token_count, repl_events) =
            crate::rules::replacement::apply_token_creation_replacement(state, recipient, resolved_count);
        events.extend(repl_events);
        for _ in 0..token_count {
            let obj = make_token(spec, recipient);
            if let Some(id) = state.expect_add_object(obj, ZoneId::Battlefield) {
                events.push(GameEvent::TokenCreated { player: recipient, object_id: id });
                events.push(GameEvent::PermanentEnteredBattlefield { player: recipient, object_id: id });
                if let Some(ref target) = attack_target {
                    if let Some(combat) = state.combat.as_mut() { combat.attackers.insert(id, target.clone()); }
                }
                ctx.last_created_permanent = Some(id);
            }
        }
    }
}
```

Default `recipient = Controller` → `recipients == vec![ctx.controller]` → byte-identical
behaviour to today (single recipient, same doubling, same events). **This is why the full
suite must stay green with zero test edits (except the sentinel bumps).**

Leave `Effect::CreateTokenAndAttachSource` reading `ctx.controller` UNLESS its default recipient
should apply — since its TokenSpec.recipient defaults to Controller and Living Weapon always
targets the controller, simplest correct choice: keep `CreateTokenAndAttachSource` on
`ctx.controller` (it never sets a non-default recipient) and note it in a comment. Do NOT
regress it.

## Step 6 — hash + version bumps (engine, machine-forced)

- `impl HashInto for TokenSpec` (`state/hash.rs` ~line 4992): add `self.recipient.hash_into(hasher);`
  as the last line (PlayerTarget already has a `HashInto` impl at ~5037 — add discriminants for
  the two new variants there too).
- `impl HashInto for PlayerTarget` (~5037): add arms for `ControllerOfCounteredSpell` /
  `ControllerOfTriggeringObject` with fresh discriminant bytes (append after the current last).
- **Run the gates and let them force the versions** (expected, per brief):
  - `cargo test --test core protocol_schema` → will fail (wire closure reaches TokenSpec).
    Bump `PROTOCOL_VERSION` 6→7 (`rules/protocol.rs:93`) and recompute
    `PROTOCOL_SCHEMA_FINGERPRINT` (the test prints expected vs actual; paste the actual).
  - `cargo test --test core hash_schema` → if the canonical fixture hash moved, bump
    `HASH_SCHEMA_VERSION` 44→45 (`state/hash.rs:406`) and update the recorded expected hash.
  - Sentinel assertions: **30 files** assert `assert_eq!(HASH_SCHEMA_VERSION, 44)` — bump all to
    45 (`grep -rl "HASH_SCHEMA_VERSION, 44" crates/ | xargs sed -i 's/HASH_SCHEMA_VERSION, 44/HASH_SCHEMA_VERSION, 45/'`).
    `protocol_schema.rs` asserts `PROTOCOL_VERSION, 6` → 7. Append history rows to the version
    tables in `hash.rs`/`protocol.rs` doc comments describing the PB-EF2 change.

## Step 7 — flip `swan_song` to Complete

`crates/card-defs/src/defs/swan_song.rs`: inside the `TokenSpec`, add
`recipient: PlayerTarget::ControllerOfCounteredSpell,`. Delete the `completeness:
Completeness::known_wrong(...)` line entirely (default is `Complete`). Keep everything else.

## Step 8 — author `An Offer You Can't Refuse`

New file `crates/card-defs/src/defs/an_offer_you_cant_refuse.rs`. Oracle: "{U} Instant.
Counter target noncreature spell. Its controller creates two Treasure tokens."

```rust
abilities: vec![AbilityDefinition::Spell {
    effect: Effect::Sequence(vec![
        Effect::CounterSpell { target: EffectTarget::DeclaredTarget { index: 0 }, exile_instead: false },
        Effect::CreateToken {
            spec: TokenSpec {
                recipient: PlayerTarget::ControllerOfCounteredSpell,
                ..treasure_token_spec(2)   // functional-update from the helper
            },
        },
    ]),
    targets: vec![TargetRequirement::TargetSpellWithFilter(TargetFilter { non_creature: true, ..Default::default() })],
    modes: None,
    cant_be_countered: false,
}],
```
Ships **Complete** (all clauses expressible — verified: `non_creature` filter honored per
negate.rs; `treasure_token_spec` exists with `ManaAbility::treasure()`). File name: verify the
authoring-report's expected filename; the report may normalize the apostrophe to
`an_offer_you_cant_refuse.rs` (drop the apostrophe). Confirm against the missing-worklist.

## Step 9 — tests (new file `crates/engine/tests/primitives/pb_ef2_create_token_recipient.rs`
+ `mod` line in `tests/primitives/main.rs`)

Cite CR. Required cases (probe by EXECUTION, per SR-34/36 lesson — resolve the spell, read
board state):
1. **swan_song happy path**: p2 casts Swan Song countering p1's instant/sorcery; after
   resolution the 2/2 Bird is controlled by **p1** (countered spell's controller).
2. **swan_song decoy**: assert the Bird is **NOT** controlled by p2 (Swan Song's caster). This
   is the field-under-test decoy — it must fail if recipient defaults to Controller.
3. **An Offer**: p2 counters p1's noncreature spell; two Treasure artifact tokens controlled by
   **p1**; assert Treasure has a mana ability.
4. **default unchanged**: a plain `CreateToken` (recipient defaulted) still mints for the
   controller (pick any existing token-maker or a builder-constructed spec).
5. **ControllerOfTriggeringObject resolves**: a minimal trigger context where
   `triggering_creature_id` is set → recipient is that object's controller (can be a focused
   unit test on `resolve_player_target_list` if a full card path is heavy).
6. **doubling keys off recipient**: give the *recipient* (p1) a Doubling-Season-style
   `WouldCreateTokens`→`DoubleTokens` replacement and Swan Song's caster (p2) none; assert p1
   gets 2 Birds. Reverse decoy: doubler on p2 only → p1 still gets exactly 1 (proves doubling
   is keyed to recipient, not controller).

## Step 10 — bookkeeping
- `memory/primitives/ef-batch-plan-2026-07-17.md`: mark EF-W-MISS-1 / PB-EF2 shipped.
- `memory/card-authoring/w-miss-roster-2026-07-17.md` + `w-miss-engine-findings-2026-07-17.md`:
  mark EF-W-MISS-1 ✅ CLOSED (PB-EF2, scutemob-102), like EF-W-MISS-2 was.
- `test-data/generated-scripts/tokens/001_swan_song_creates_bird.json`: if the fix makes it pass,
  set `review_status: "approved"`, drop `retirement_reason` (and the retired partition). Run
  `SCRIPT_FILTER=tokens_001 cargo test --test scripts run_all_scripts -- --nocapture` (never the
  HTTP server — OOM). If it does NOT pass, leave retired and add a `generation_notes` line why.
- `python3 tools/authoring-report.py` → record coverage delta (expect +1 or +2 Complete:
  swan_song flip + An Offer author).

## Gotchas
- `resolve_player_target_list` returns `Vec` — recipient may be empty (target fizzled /
  player lost) → no tokens, which is correct (An Offer ruling: illegal target → no Treasures).
- Do NOT edit any of the 160 card-def token specs — they inherit `recipient: Controller`.
- `EffectContext` has many constructors; a missed one is a compile error — fix all.
- CreateTokenAndAttachSource: leave on `ctx.controller`; note why.
