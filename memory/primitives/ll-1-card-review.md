# LL-1 card review — "its controller creates" token recipients

**Reviewed**: 2026-09-05
**Reviewer**: card-batch-reviewer (read-only; no `.rs` file edited, no `git` command run)
**Scope**: the six defs changed by LL-1 under `crates/card-defs/src/defs/`
**Oracle source**: `mtg-rules` MCP `lookup_card` (one call per card, 6 total) — never from memory
**CR source**: `mtg-rules` MCP `get_rule` / `search_rules` (701.5, 701.6, 701.7, 701.8a, 608.2h,
400.7, 111.10)

**Findings**: 0 HIGH, 1 MEDIUM, 4 LOW

---

## Cross-cutting verification (applies to all six)

Done once, cited per card below rather than repeated:

- **Oracle text**: all six `oracle_text` strings are byte-identical to Scryfall's oracle text
  (line-continuation backslashes in the Rust literal collapse to single spaces; each was
  reassembled and compared). No typos, no omissions, no added reminder text where the card has
  none, and Emergency Eject's parenthetical reminder is reproduced verbatim including the inner
  quotes.
- **Mana costs**: Beast Within {2}{G}, Generous Gift {2}{W}, Stroke of Midnight {2}{W},
  Pongify {U}, Saw in Half {2}{B}, Emergency Eject {2}{W} — all six match.
- **Types**: all six are `types(&[CardType::Instant])`; all six oracle type lines are `Instant`
  with no supertype and no subtype. No P/T fields on any (correct: none is a creature).
- **`targets:` vec length is 1 in every def**, so `EffectTarget::DeclaredTarget { index: 0 }` is
  unambiguously the destroyed permanent in all five `recipient` sites and in all six
  `DestroyPermanent` sites. Index 0 is correct everywhere.
- **Target filters**: `TargetPermanent` for "target permanent" (Beast Within, Generous Gift);
  `TargetPermanentWithFilter(TargetFilter { non_land: true, .. })` for "target nonland permanent"
  (Stroke of Midnight, Emergency Eject — KI-1 satisfied, `non_land` verified at
  `card_definition.rs:3204` as "Must not be a land"); `TargetCreature` for "target creature"
  (Pongify, Saw in Half). All correct.
- **Token-creation ordering**: every def puts `CreateToken` in a `Sequence` *after*
  `DestroyPermanent`, unconditionally — correct, since the token is created whether or not the
  destroy actually killed the permanent (indestructible / regenerated). The `ControllerOf` arm
  reads the live object when it survived and `EffectContext::departed_permanent_players` when it
  did not (`crates/engine/src/effects/mod.rs:70-96`), so both branches name the same player.
- **Stale prose**: I grepped all six files for the retired claim shapes ("CreateToken lacks a
  player field", "fix when", "TODO", "ENGINE-BLOCKED", trailing "..."). **None survive.** Every
  comment in all six files describes the current state. `memory/conventions.md:219-224` is
  satisfied.

---

## 1. `crates/card-defs/src/defs/beast_within.rs`

- **Oracle match**: YES — "Destroy target permanent. Its controller creates a 3/3 green Beast
  creature token."
- **Mana cost / types**: YES ({2}{G}, Instant).
- **Clause coverage**: 2 clauses, 2 modelled. "Destroy target permanent" →
  `DestroyPermanent { target: DeclaredTarget{0}, cant_be_regenerated: false }` (correct — the card
  says nothing about regeneration). "Its controller creates a 3/3 green Beast creature token" →
  `CreateToken` with name `Beast`, 3/3, `Color::Green`, `CardType::Creature`, `SubType("Beast")`,
  `count: Fixed(1)`.
- **Recipient**: PRESENT and CORRECT — `beast_within.rs:45-47`,
  `PlayerTarget::ControllerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 }))`, index 0 is the
  destroyed permanent.
- **`completeness: Complete`**: **TRUE and justified.** Both printed clauses are modelled and the
  recipient is the destroyed permanent's controller, not the caster.
- **Findings**:
  - **F1 (MEDIUM)** — `crates/card-defs/src/defs/beast_within.rs:40`: the comment cites
    **"CR 701.6a / CR 608.2h"**. CR 701.6a is **Counter** ("To counter a spell or ability means to
    cancel it…"), which has nothing to do with this card. The keyword actions actually in play are
    **CR 701.7 (Create)** and **CR 701.8a (Destroy — "To destroy a permanent, move it from the
    battlefield to its owner's graveyard")**; the LKI rule for "its controller" is CR 608.2h, which
    IS cited correctly. `memory/conventions.md:356-358` explicitly requires grep-verifying a cited
    701.x/702.x number precisely because "701.x / 702.x numbers are sequential assignments that
    models hallucinate". Fix: replace `701.6a` with `701.8a` (or `701.7a`), or drop it and keep the
    608.2h + 400.7 pair the other four defs use. No game-state impact — comment only.
  - **F2 (LOW)** — the same miscitation propagated into the new test:
    `crates/engine/tests/primitives/ll1_token_recipient_controller_of.rs:305`, `:401`, `:406`
    each cite "CR 701.6a" for the recipient assertion. Fixing only the def leaves three wrong
    citations standing in the test that proves it. (Adjacent file, flagged so the fix is complete;
    the test's *behaviour* is correct and out of this review's scope.)

## 2. `crates/card-defs/src/defs/generous_gift.rs`

- **Oracle match**: YES — "Destroy target permanent. Its controller creates a 3/3 green Elephant
  creature token."
- **Mana cost / types**: YES ({2}{W}, Instant).
- **Clause coverage**: 2 clauses, 2 modelled. Token: `Elephant`, 3/3, green, Creature,
  `SubType("Elephant")` — note the token is **green** even though the spell is white; the def has
  `colors: [Color::Green]`, which matches the oracle text and is a classic place to get this wrong.
  Correct here.
- **Recipient**: PRESENT and CORRECT — `generous_gift.rs:42-44`, index 0.
- **`completeness: Complete`**: **TRUE and justified.**
- **Findings**: **none — this def is clean.** (Its CR comment cites only 608.2h and 400.7, both
  verified correct.)

## 3. `crates/card-defs/src/defs/stroke_of_midnight.rs`

- **Oracle match**: YES — "Destroy target nonland permanent. Its controller creates a 1/1 white
  Human creature token."
- **Mana cost / types**: YES ({2}{W}, Instant).
- **Clause coverage**: 2 clauses, 2 modelled. "nonland" is carried by
  `TargetPermanentWithFilter(TargetFilter { non_land: true, .. })` at `:49-52`, not by a bare
  `TargetPermanent` (KI-1 clear). Token: `Human`, 1/1, `Color::White`, Creature,
  `SubType("Human")`.
- **Recipient**: PRESENT and CORRECT — `stroke_of_midnight.rs:42-44`, index 0.
- **`completeness: Complete`**: **TRUE and justified.** This is one of the two defs
  `memory/conventions.md:365-368` names as having been wrongly gated on the already-shipped
  `TokenSpec.recipient`; the stale blocker note is gone and the marker is now earned.
- **Findings**: **none — this def is clean.**

## 4. `crates/card-defs/src/defs/pongify.rs`

- **Oracle match**: YES — "Destroy target creature. It can't be regenerated. Its controller creates
  a 3/3 green Ape creature token."
- **Mana cost / types**: YES ({U}, Instant — `blue: 1`, no generic).
- **Clause coverage**: 3 clauses, 3 modelled. The middle clause "It can't be regenerated" is
  carried by `cant_be_regenerated: true` at `:21` — present, and correctly `false` on the four
  sibling cards that do not print it. Token: `Ape`, 3/3, green, Creature, `SubType("Ape")`.
- **Recipient**: PRESENT and CORRECT — `pongify.rs:41-43`, index 0.
- **`completeness: Complete`**: **TRUE and justified.**
- **Findings**: **none — this def is clean.**

## 5. `crates/card-defs/src/defs/saw_in_half.rs`

- **Oracle match**: YES — "Destroy target creature. If that creature dies this way, its controller
  creates two tokens that are copies of that creature, except their power is half that creature's
  power and their toughness is half that creature's toughness. Round up each time."
- **Mana cost / types**: YES ({2}{B}, Instant).
- **Clause coverage**: clause 1 ("Destroy target creature") modelled; clause 2 (the conditional
  halved-copy tokens) NOT modelled — as the brief states, this is expected.
- **Recipient**: N/A — there is no `Effect::CreateToken` in this def, correctly, since the missing
  clause needs a *copy* token with a P/T override, not a fixed-stat token.
- **`completeness: Completeness::partial(...)`**: **TRUE.** The note at `:40-47` names exactly the
  clause that is missing, quotes it, and says "Only the destroy clause is implemented." It does not
  trail off, and it does not overclaim.
- **Bounded-claim spot check (I verified this independently, per the brief)**:
  - The note says the `Effect` enum has **106 variants** in
    `crates/card-types/src/cards/card_definition.rs`. I counted the top-level variants of
    `pub enum Effect` (lines 1390–2595): **exactly 106**, `DealDamage` at :1393 through
    `SetNoMaximumHandSize` at :2591. The count is accurate.
  - The note says the copy candidates are `CreateTokenCopy`, `CreateTokenAndAttachSource`, and
    `BecomeCopyOf`, and that none carries a P/T override. Verified: `CreateTokenCopy`
    (`:2287-2310`) has exactly the 5 named fields — `source`, `enters_tapped_and_attacking`,
    `except_not_legendary`, `gains_haste`, `delayed_action` — and no stat field; `BecomeCopyOf`
    (`:2271-2278`) has `copier`, `target`, `duration`; `CreateTokenAndAttachSource` (`:1967`) takes
    a plain `TokenSpec` (fixed `power: i32` / `toughness: i32`, not derived from the copied
    creature). `CopySpellOnStack` copies spells, not permanents. The claim holds.
  - **Additional support the note does not mention**: even if a P/T override existed, the *halving
    arithmetic* does not. `EffectAmount` (`:2781-3076`, 26 variants) has `PowerOf` / `ToughnessOf`
    but no divide/half/round-up variant. The gap is doubly real.
- **Findings**:
  - **F3 (LOW)** — `saw_in_half.rs:40-47`: the bounded claim names the enum, the module and the
    variant count, but omits the "at `<sha>`" anchor that `memory/conventions.md:359-360` gives as
    the phrasing template (*"absent from `<enum>` (N variants) at `<sha>`"*). Without it the count
    silently rots as the enum grows. Suggest appending the LL-1 commit sha, or the batch name, so a
    future reader can date the count.
  - **F4 (LOW)** — the same gap is stated **three times** in one 50-line file: the file header
    (`:4-10`), a comment inside the `AbilityDefinition::Spell` literal (`:36-38`), and the
    `Completeness::partial` note (`:41-46`). Only the `partial` note is machine-read
    (`tools/authoring-report.py`), so the other two are copies that can drift out of agreement with
    it — precisely the failure mode `memory/conventions.md:219-224` warns about. Suggest keeping
    the `partial` note as the single source of truth and reducing the header to one line that
    points at it; the in-struct comment at `:36-38` adds nothing and sits in an odd position
    (trailing the last struct field).
  - Neither LOW affects game state; the def's behaviour and its marker are correct.

## 6. `crates/card-defs/src/defs/emergency_eject.rs` — scrutinised hardest

- **Oracle match**: YES, including the full parenthetical reminder text.
- **Mana cost / types**: YES ({2}{W}, Instant).
- **Clause coverage**: 2 clauses. "Destroy target nonland permanent" → `DestroyPermanent` +
  `TargetPermanentWithFilter(TargetFilter { non_land: true, .. })` at `:86-89`. "Its controller
  creates a Lander token" → `CreateToken` with the Lander authored inline.
- **Recipient**: PRESENT and CORRECT — `emergency_eject.rs:79-81`, index 0.
- **Lander token vs CR 111.10u** — I looked the rule up rather than trusting the card's reminder
  text. CR 111.10u: *"A Lander token is a colorless Lander artifact token with '{2}, {T}, Sacrifice
  this token: Search your library for a basic land card, put it onto the battlefield tapped, then
  shuffle.'"* Checked field by field against `:29-83`:
  - `name: "Lander"` ✓; `colors: OrdSet::new()` = colorless ✓; `card_types: [Artifact]` ✓;
    `subtypes: [SubType("Lander")]` ✓ — and "Lander" is a registered artifact subtype
    (`crates/card-types/src/state/types.rs:1928`, `ALL_ARTIFACT_TYPES`), so it is not a
    free-text subtype that would fail a type-line check.
  - `power: 0, toughness: 0` ✓ — matches the house convention for non-creature artifact tokens
    (`treasure_token_spec` `card_definition.rs:4569-4570`, `food_token_spec` `:4592-4593`,
    `clue_token_spec` `:4646-4647` all use 0/0, not the `TokenSpec::default()` 1/1). Correct.
  - `supertypes` empty ✓, `keywords` empty ✓, `count: Fixed(1)` ✓ ("a Lander token", singular),
    `tapped: false` ✓ (the *token* is not created tapped — only the land it later fetches is),
    `enters_attacking: false` ✓, `mana_color: None` ✓, `mana_abilities: vec![]` ✓ (the Lander's
    ability is not a mana ability — it fetches a land, so it must be in `activated_abilities`, and
    it is).
  - **Cost**: `requires_tap: true` + `mana_cost: generic 2` + `sacrifice_self: true` = "{2}, {T},
    Sacrifice this token" ✓ — all three components present, none extra. `ActivationCost` derives
    `Default` (`crates/card-types/src/state/game_object.rs:460`), so the `..Default::default()`
    tail is safe and leaves every other cost component false/zero. Summoning sickness is not a
    concern: the Lander is not a creature, so the {T} is usable the turn it is created.
  - **Effect**: `Sequence([SearchLibrary { player: Controller, filter: basic_land_filter(),
    reveal: false, destination: Battlefield { tapped: true }, shuffle_before_placing: false,
    also_search_graveyard: false }, Shuffle { player: Controller }])`. This is **byte-for-byte the
    established reference implementation** of the identical printed ability in
    `crates/card-defs/src/defs/wayfarers_bauble.rs:25-37`. I verified the trailing `Shuffle` is
    required rather than redundant: `Effect::SearchLibrary`'s engine arm
    (`crates/engine/src/effects/mod.rs:4240-4432`) shuffles only when `shuffle_before_placing` is
    set, and never after placing — so without the explicit `Shuffle` the "then shuffle" clause
    would silently not happen. Correct as written. `reveal: false` matches the oracle (no reveal is
    printed). `PlayerTarget::Controller` inside the token's ability resolves to the *token's*
    controller — i.e. the destroyed permanent's controller, who received it — which is the right
    player, not the caster.
  - **Is the ability actually live on the created token?** Yes — token creation copies
    `spec.activated_abilities` onto the token's `Characteristics`
    (`crates/engine/src/effects/mod.rs:9801-9814`), the same path Food uses. So the `Complete`
    marker is not resting on an unwired field.
- **`completeness: Complete`** (changed from `partial` this batch): **TRUE and justified.** Both
  printed clauses are modelled; the parenthetical is reminder text, not a third clause; the
  reminder's content is itself fully modelled; the recipient is the destroyed permanent's
  controller. This is the second of the two defs `memory/conventions.md:365-368` names as wrongly
  gated on the already-shipped `TokenSpec.recipient` — the promotion is now earned, and it is
  additionally backed end-to-end by
  `crates/engine/tests/primitives/ll1_token_recipient_controller_of.rs:311` (asserts the Lander
  reaches p3, then has p3 pay {2}+{T}+sacrifice and fetch a basic onto the battlefield tapped).
- **Findings**:
  - **F5 (LOW)** — `emergency_eject.rs:29-83`: the Lander is a **CR 111.10u predefined token**, but
    it is authored inline in the card def rather than as a `lander_token_spec(count: u32)` helper
    beside `treasure_token_spec` / `food_token_spec` / `clue_token_spec` / `blood_token_spec` in
    `crates/card-types/src/cards/card_definition.rs:4562-4691`, which is where every other CR
    111.10 predefined token lives. Two consequences: (a) the next Lander-making card (Edge of
    Eternities prints many) will duplicate 40 lines and can drift; (b) the inline spec carries no
    CR citation, whereas each sibling helper is doc-commented with its CR 111.10x subrule. Suggest
    extracting `lander_token_spec(count)` documented as CR 111.10u and calling it here with
    `TokenSpec { recipient: ..., ..lander_token_spec(1) }`. Purely structural — the authored
    characteristics are all correct as they stand, so this is not a correctness defect.

---

## Summary

| Def | Oracle | Types | Cost | Recipient | Marker | Verdict |
|-----|--------|-------|------|-----------|--------|---------|
| `beast_within.rs` | ✓ | ✓ | ✓ | ✓ index 0 | `Complete` — TRUE | 1 MEDIUM + 1 LOW (comment only) |
| `generous_gift.rs` | ✓ | ✓ | ✓ | ✓ index 0 | `Complete` — TRUE | **clean** |
| `stroke_of_midnight.rs` | ✓ | ✓ | ✓ | ✓ index 0 | `Complete` — TRUE | **clean** |
| `pongify.rs` | ✓ | ✓ | ✓ | ✓ index 0 | `Complete` — TRUE | **clean** |
| `saw_in_half.rs` | ✓ | ✓ | ✓ | n/a (no token) | `partial` — TRUE, claim verified | 2 LOW (prose hygiene) |
| `emergency_eject.rs` | ✓ | ✓ | ✓ | ✓ index 0 | `Complete` — TRUE | 1 LOW (structural) |

- **Cards with issues**: Beast Within (F1 MEDIUM, F2 LOW), Saw in Half (F3, F4 LOW),
  Emergency Eject (F5 LOW).
- **Clean cards**: Generous Gift, Stroke of Midnight, Pongify.
- **Counts**: **0 HIGH, 1 MEDIUM, 4 LOW.**
- **No finding affects game state.** Every `completeness:` marker in the batch is independently
  justified against the looked-up oracle text: five `Complete` markers are earned clause by clause,
  and the one `partial` marker names a gap I confirmed is real (106-variant `Effect` count exact;
  no copy variant carries a P/T override; `EffectAmount` has no halving arithmetic either).
- **No stale prose survives** in any of the six files — the retired "CreateToken lacks a player
  field" claim shape is gone everywhere.
