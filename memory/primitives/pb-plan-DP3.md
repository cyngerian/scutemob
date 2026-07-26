# Primitive Batch Plan: PB-DP3 — Mode announcement is mandatory (DP-4)

**Generated**: 2026-07-26
**Primitive**: not a new DSL variant — a **validation-lift**. Mode legality (range / duplicates /
`min_modes` / `max_modes`) moves *out* of the `!modes_chosen.is_empty()` gate in
`rules/casting.rs` and `rules/abilities.rs`, so a modal spell or activated ability cast/activated
with **no announced modes** is rejected instead of silently resolving mode 0 at full price.
**CR Rules**: 601.2b, 602.2b, 700.2, 700.2a, 700.2b, 700.2d, 702.42a/b (entwine),
702.120a (escalate), 702.172a (spree)
**Cards affected**: 41 defs carry `min_modes` (37 × `min_modes: 1`, 3 × `min_modes: 2`,
1 × `min_modes: 0`). **3 `Complete` cards are live-wrong today** — `cryptic_command`,
`austere_command`, `incendiary_command`. **Zero card-def edits are required** (this is an engine
+ caller fix; the defs are already correct).
**Dependencies**: PB-AC4 (per-mode targets), PB-EF7 (modal activated abilities), PB-DP1/DP2
(suite predecessors, both shipped). All present.
**Deferred items from prior PBs**: none bear on DP-4. The RS queue is paused at RS4 and is not
touched here. `memory/workstream-state.md` "Last Handoff" carries no DP-3-relevant carry-forward.

---

## 0. STOP-CHECK: wire neutrality holds — proceed

Nothing in this plan adds, removes, or reshapes a `Command`, `GameEvent`, `Effect`,
`GameState`, `StackObject`, or `CardDefinition` field or variant. Every change is control flow
inside two existing handlers, comment corrections, caller-side argument population, and tests.

- `PROTOCOL_SCHEMA_FINGERPRINT` (`crates/engine/src/rules/protocol.rs:277`) digests the
  transitive **type closure** of `Command`/`GameEvent`. That closure is untouched.
  **`PROTOCOL_VERSION` stays 27** (`protocol.rs:260`).
- `HASH_SCHEMA_VERSION` covers serialized `GameState` field shape. Untouched.
  **HASH stays 63.**
- The two new `pub fn` helpers land in `crates/simulator`, which is outside the engine's
  wire closure entirely.

**If the runner reaches a point where a new field or variant seems necessary — in particular to
let resolution distinguish "the controller announced zero modes" from "this stack object was
built by a free-cast that never announced anything" — STOP and escalate.** That is exactly the
seed `OOS-DP3-2` below, and it is a PROTOCOL/HASH bump that PB-DP3 must not take.

---

## 1. The defect, verified by reading

`crates/engine/src/rules/casting.rs:3507`:

```rust
let validated_modes_chosen: Vec<usize> = if !modes_chosen.is_empty() && !entwine_paid {
    match &mode_selection_opt {
        None => { /* :3509-3513  non-modal + modes supplied -> error */ }
        Some(ms) => {
            /* :3516-3524  CR 700.2a range check          */
            /* :3526-3536  CR 700.2d duplicate check      */
            /* :3539-3544  CR 700.2a min_modes            */
            /* :3545-3550  CR 700.2a max_modes            */
            /* :3552       ascending sort                 */
        }
    }
} else {
    // :3556-3560  "Empty = non-modal spell or auto-select mode[0] (backward compatible)."
    modes_chosen
};
```

Every check is inside the non-empty arm. Downstream, two consumers re-derive `vec![0]` from the
empty vector:

- `casting.rs:3646-3654` — per-mode target slicing (`mode_targets_active`).
- `resolution.rs:335-341` — `chosen_mode_indices`, inside the **Spell** resolution branch
  (verified: the enclosing `if let AbilityDefinition::Spell { effect, modes, .. }` starts at
  `resolution.rs:266-278`).

So `Command::CastSpell { modes_chosen: vec![], .. }` on Cryptic Command (`min_modes: 2,
max_modes: 2`, `crates/card-defs/src/defs/cryptic_command.rs:31-32`), Austere Command
(`austere_command.rs:27-28`) or Incendiary Command (`incendiary_command.rs:37-38`) — **all three
`Complete`** (no `completeness` field ⇒ `Completeness::default() == Complete`,
`crates/card-types/src/cards/card_definition.rs:197-200`) — pays the full mana cost and resolves
exactly one mode.

Mana is paid at `casting.rs:4005-4009`, i.e. **after** `:3507`. So a rejection added at `:3507`
correctly refuses the cast before any cost is spent (CR 601.2b announces modes before costs).

The same bypass exists for activated abilities at `crates/engine/src/rules/abilities.rs:337-398`
(audit §4.2 line 214, class **B**, "same `min_modes` bypass as DP-4"): `:387-394` turns an empty
`modes_chosen` into `vec![0]`.

---

## 2. CR rule text (verbatim, from the mtg-rules MCP)

**CR 601.2b** (spell announcement — the operative first sentence):

> If the spell is modal, the player announces the mode choice (see rule 700.2). […] If the spell
> has alternative or additional costs that will be paid as it's being cast such as buyback or
> kicker costs (see rules 118.8 and 118.9), the player announces their intentions to pay any or
> all of those costs (see rule 601.2f).

**CR 700.2**:

> A spell or ability is modal if it has two or more options in a bulleted list preceded by
> instructions for a player to choose a number of those options, such as "Choose one —." Each of
> those options is a mode.

**CR 700.2a**:

> The controller of a modal spell or activated ability **chooses the mode(s) as part of casting
> that spell or activating that ability**. If one of the modes would be illegal (due to an
> inability to choose legal targets, for example), that mode can't be chosen. (See rule 601.2b.)

**CR 700.2b**:

> The controller of a modal triggered ability chooses the mode(s) as part of putting that ability
> on the stack. […] **If no mode is chosen, the ability is removed from the stack.** (See rule
> 603.3c.)

**CR 700.2d**:

> If a player is allowed to choose more than one mode for a modal spell or ability, that player
> normally can't choose the same mode more than once. However, some modal spells include the
> instruction "You may choose the same mode more than once." […]

**CR 702.42a/b** (entwine):

> "Entwine [cost]" means "You may choose **all modes** of this spell **instead of just the number
> specified**. If you do, you pay an additional [cost]." Using the entwine ability follows the
> rules for choosing modes and paying additional costs in rules 601.2b and 601.2f–h.
> […] If the entwine cost was paid, follow the text of each of the modes in the order written on
> the card when the spell resolves.

**CR 702.120a** (escalate):

> Escalate is a static ability of modal spells (see rule 700.2) that functions while the spell
> with escalate is on the stack. "Escalate [cost]" means "**For each mode you choose beyond the
> first** as you cast this spell, you pay an additional [cost]." Paying a spell's escalate cost
> follows the rules for paying additional costs in rules 601.2f–h.

### 2.1 What the CR actually settles (answers to the brief's question 1)

1. **"There is no default mode."** CR 700.2a says the controller "chooses the mode(s) **as part
   of** casting that spell." CR 601.2b puts that announcement in step 601.2b, before costs are
   determined (601.2f) or paid (601.2h). Nothing in CR 700.2 or 601.2 provides a fallback,
   default, or engine-side choice. **The engine auto-selecting mode 0 is a rules violation, not a
   convenience.** This is the DP suite's core thesis and it applies literally here.

2. **`min_modes: 0` ("choose up to one") legitimately allows an empty choice.** CR 700.2b's "If
   no mode is chosen, the ability is removed from the stack" presupposes that choosing zero modes
   is a thing a controller may do when the printed instruction permits it ("choose up to one").
   So the fix must **not** reject empty on a `min_modes: 0` object *as a matter of legality*.
   The corpus's only `min_modes: 0` object is `crates/card-defs/src/defs/hullbreaker_horror.rs:59`,
   and it is a **modal triggered ability**, not a spell — verified by reading; the `ModeSelection`
   sits inside `AbilityDefinition::Triggered` at `:35-88`. **There is no `min_modes: 0` modal
   Spell or Activated ability anywhere in the 1,798-def corpus.**
   Consequences are handled per-path in §3.3 and §3.5.

3. **Escalate announces a *count*, not just a payment.** CR 702.120a defines the escalate cost as
   a function of "each mode you choose beyond the first". The engine's
   `AdditionalCost::EscalateModes { count }` therefore carries a genuine, player-made
   announcement of *how many* modes were chosen — just not *which*. That is the CR footing for
   the narrow exemption in §3.4. What it does **not** license is the engine's choice of *which*
   modes (`0..=count`, contiguous from zero) — CR 702.120a permits any set of `count + 1` distinct
   modes. That residual is seeded as `OOS-DP3-1`.

---

## 3. Engine changes

### Change 1 — `casting.rs`: lift the checks out of the emptiness gate

**File**: `crates/engine/src/rules/casting.rs`
**Site**: replace the `let validated_modes_chosen = …` expression at **`:3507-3560`**. Leave the
`mode_selection_opt` lookup at `:3495-3506` exactly as it is (it is already computed
unconditionally, which is what makes this fix cheap).

**Decision (answers brief question 2): lift, do not bolt on.** A separate "if empty and modal,
reject" check bolted onto the existing gate would leave the gate's shape — and therefore the
class of bug — intact. The correct structure is: *validate whenever the spell is modal.* The new
expression is a three-way match on `(entwine_paid, mode_selection_opt, modes_chosen.is_empty())`,
with one narrowly-scoped escalate exemption.

**Required shape** (the runner implements exactly this control flow; wording of comments is the
runner's, but every CR citation shown must appear):

```rust
// CR 601.2b / 700.2a: a modal spell's controller announces the mode(s) as part of casting,
// before costs are determined (CR 601.2f) or paid (CR 601.2h). There is no default mode and
// the engine may not pick one — PB-DP3 / DP-4.
let validated_modes_chosen: Vec<usize> = if entwine_paid {
    // CR 702.42a: entwine chooses ALL modes "instead of just the number specified", so
    // min_modes/max_modes are overridden by the keyword itself and modes_chosen is ignored
    // (resolution.rs:313-316 expands to 0..modes.len()). Pass through unchanged and
    // UNSORTED, exactly as before — `rules/modal.rs::test_modal_entwine_overrides_modes_chosen`
    // pins that entwine + modes_chosen=[0] still executes every mode.
    modes_chosen
} else {
    match &mode_selection_opt {
        None => {
            if !modes_chosen.is_empty() {
                // PRESERVED VERBATIM from :3509-3513 — do not reword; the message is asserted
                // by rules/modal.rs::test_modal_non_modal_spell_with_modes_chosen_rejected and
                // primitives/pb_ef7_modal_activated.rs's sibling test.
                return Err(GameStateError::InvalidCommand(
                    "modes_chosen specified but spell has no modal structure (modes: Some(...)) (CR 700.2a)".into(),
                ));
            }
            modes_chosen // non-modal spell, nothing announced: unchanged
        }
        Some(ms) if modes_chosen.is_empty() => {
            // --- the DP-4 fix ---
            if escalate_modes > 0 {
                // CR 702.120a exemption — see plan §3.4. The count IS announced via
                // AdditionalCost::EscalateModes; resolution derives 0..=escalate_modes
                // (resolution.rs:321-334). Validate the DERIVED count against the printed
                // bounds so escalate cannot smuggle an illegal mode count through.
                let derived = ((escalate_modes as usize) + 1).min(ms.modes.len());
                if derived < ms.min_modes {
                    return Err(GameStateError::InvalidCommand(format!(
                        "escalate chose {} mode(s); at least {} required (CR 702.120a/700.2a)",
                        derived, ms.min_modes
                    )));
                }
                if derived > ms.max_modes {
                    return Err(GameStateError::InvalidCommand(format!(
                        "escalate chose {} mode(s); at most {} allowed (CR 702.120a/700.2a)",
                        derived, ms.max_modes
                    )));
                }
                modes_chosen // stays empty; the escalate backward-compat path owns it
            } else if ms.min_modes == 0 {
                // Fail-safe hard reject (CR 601.2b/700.2a). "Choose up to N" legitimately
                // permits announcing zero modes, but this engine cannot REPRESENT that on a
                // spell: resolution.rs:335-338 turns an empty modes_chosen on a Spell stack
                // object into vec![0], and it cannot distinguish "controller chose zero" from
                // a cascade/discover free-cast that never announced anything (copy.rs:430,
                // :646). Accepting the cast would silently resolve mode 0 — wrong game state.
                // No shipped card has this shape (the corpus's only min_modes: 0 object is a
                // TRIGGERED ability, hullbreaker_horror.rs:59). Tracked as OOS-DP3-2.
                return Err(GameStateError::InvalidCommand(
                    "a modal spell with min_modes: 0 cast with no modes announced is not \
                     representable: resolution would auto-select mode 0 (CR 601.2b/700.2a) — \
                     see OOS-DP3-2".into(),
                ));
            } else {
                return Err(GameStateError::InvalidCommand(format!(
                    "modal spell requires an explicit mode choice: at least {} mode(s) must be \
                     announced as part of casting (CR 601.2b/700.2a); none were",
                    ms.min_modes
                )));
            }
        }
        Some(ms) => {
            // UNCHANGED from :3516-3553 — range (CR 700.2a), duplicates (CR 700.2d),
            // min_modes / max_modes (CR 700.2a), ascending sort (CR 700.2a).
            // Move the existing code verbatim; do not reword any error message.
            …
            modes_chosen.sort_unstable();
            modes_chosen
        }
    }
};
```

**Notes the runner must honour:**

- `modes_chosen` is bound `mut` today (it is sorted in place at `:3552`). Keep that.
- Error messages in the *existing* arms must be byte-identical to today's. Three tests assert on
  message substrings: `rules/modal.rs:609` (`"out of range"`),
  `mechanics_m_z/spree.rs:854` (`"spree spell requires at least one mode"`), and
  `primitives/pb_ac4_per_mode_targeting.rs:886` (`"Escalate"` + `"mode_targets"`).
- The rejection must stay at this position in the function — before mana payment
  (`:4005-4009`) and before the Escalate + `mode_targets` hard-reject (`:3670-3675`), which the
  escalate exemption deliberately falls through to.

### Change 2 — `casting.rs`: reconcile the `vec![0]` in per-mode target slicing (comment only)

**File**: `crates/engine/src/rules/casting.rs:3646-3654` (inside `mode_targets_active`).
**Action**: **keep the arm, fix the comment.** After Change 1 the `else if !ms.modes.is_empty()
{ vec![0] }` arm at `:3650-3651` is reachable only when `mode_targets.is_some()` **and**
`validated_modes_chosen` is empty — which after Change 1 means escalate-with-count > 0 — and
that combination is hard-rejected 16 lines later at `:3670-3675`. So the arm is **unreachable in
practice but retained as a fail-safe**.

Per `memory/conventions.md` "Aspirationally-wrong code comments are correctness hazards": update
the block comment at `:3613-3627` and the arm itself to state (a) that empty
`validated_modes_chosen` no longer means "auto-select mode 0 for any modal spell" post-PB-DP3,
(b) that the only producer is the escalate backward-compat path, and (c) that the arm is retained
fail-safe, not live. Do **not** delete it.

### Change 3 — `resolution.rs`: the Spell `vec![0]` fallback STAYS (comment only)

**File**: `crates/engine/src/rules/resolution.rs:335-341`
**Action**: **do not remove.** Answering the brief's question 3 directly — this arm is **still
live** and load-bearing, because six code paths build a `StackObject` with
`modes_chosen: vec![]` **without ever calling `handle_cast_spell`**:

| producer | site |
|---|---|
| cascade free-cast | `crates/engine/src/rules/copy.rs:430` |
| discover free-cast | `crates/engine/src/rules/copy.rs:646` |
| free-cast / alt-entry stack builds | `crates/engine/src/rules/engine.rs:2112`, `:2176`, `:2686`, `:2853` |

Verified: `handle_cast_spell` has exactly one caller — `rules/engine.rs:137` — and
`effects/mod.rs` never constructs a `CastSpell` (grep for `modes_chosen|handle_cast_spell` in
`effects/mod.rs` returns nothing). So the guard in Change 1 cannot reach those six producers, and
deleting the resolution fallback would silently make every cascaded/discovered modal spell resolve
*nothing*.

**Required edit**: replace the comment at `:336-337` ("Auto-select first mode (default for bots
and backward compat with existing scripts/tests)") — that description is now false for the cast
path — with an accurate one naming the six producers above, citing **DP-20** as the finding that
owns them, and pointing at seed `OOS-DP3-3`. Leave `chosen_mode_indices`'s logic untouched.

### Change 4 — `abilities.rs`: same lift for modal **activated** abilities

**SCOPE CALL — explicit, per the brief's question 3.** The ESM acceptance criteria name only the
cast path (criterion 5523 says "Casting a modal spell…"). **Recommendation: include
`abilities.rs` in PB-DP3.** Reasoning, stated so the reviewer can overrule it:

- It is the **same defect**, not an adjacent one — audit §4.2 line 214 says so in as many words
  ("same `min_modes` bypass as **DP-4**"), and the code at `abilities.rs:333-398` is a
  hand-copied mirror of `casting.rs:3485-3560` (its own comment at `:334` says
  "Mirrors the Spell modal validation in casting.rs").
- **Blast radius is enumerated and tiny**: 3 card defs carry a modal *activated* ability
  (`umezawas_jitte.rs:58/69`, `goblin_cratermaker.rs:48`, `cankerbloom.rs:53`), all
  `min_modes: 1`; **every existing test that activates one already passes explicit modes**
  (`primitives/pb_os10_singleton_cleanup.rs:749/777/821/853/921/932`, and every case in
  `primitives/pb_ef7_modal_activated.rs`, whose helper at `:86-98` takes `modes_chosen` as a
  parameter). **Zero golden scripts** activate a modal ability (only three `"modes"` occurrences
  exist across the whole script corpus and all three are on `cast_spell_modal` actions).
  Net test/script edits from this half: **zero**.
- Leaving it out means shipping a PB that fixes half of one bug and files the other half as a
  seed whose fix is a 12-line edit in a file the runner already has open.

**File**: `crates/engine/src/rules/abilities.rs`
**Site**: the `else if let Some(ms) = &ability_modes { … }` arm at **`:387-394`**.
**Action**: replace the `vec![0]` fallback:

```rust
} else if let Some(ms) = &ability_modes {
    // CR 700.2a / 602.2b (PB-DP3 / DP-4): the controller chooses the mode(s) "as part of
    // … activating that ability". The engine may not pick for them.
    if ms.min_modes == 0 {
        // "Choose up to N" — announcing zero modes is legal (CR 700.2a). Unlike the Spell
        // path, this is REPRESENTABLE here: with validated_modes_chosen empty, the
        // `if !validated_modes_chosen.is_empty()` guard at :492 leaves `embedded_effect` as
        // the ability's own base effect, which is the correct "no mode chosen" behaviour.
        // No shipped card has this shape; kept correct-by-construction.
        vec![]
    } else {
        return Err(GameStateError::InvalidCommand(format!(
            "modal ability requires an explicit mode choice: at least {} mode(s) must be \
             announced as part of activating it (CR 602.2b/700.2a); none were",
            ms.min_modes
        )));
    }
} else {
    // Non-modal ability.
    vec![]
};
```

Also correct the stale comment at `:402-403` ("`validated_modes_chosen` already incorporates the
auto-select-mode-0 fallback above") — that is no longer true.

**Do not touch** `abilities.rs:8376-8421` (modal **triggered** abilities choose modes at
queue time, CR 700.2b — a different decision point, seeded as `OOS-DP3-4`) or
`resolution.rs:2056-2100` (modal triggered resolution, which already requires a non-empty
`modes_chosen` and has no `vec![0]` fallback).

### Change 5 — the Spree guard stays (no edit; verification only)

**File**: `crates/engine/src/rules/casting.rs:2938-2945`.
**Action**: **leave exactly as is.** Answering the brief's question 2: the new general guard does
*not* make it redundant in practice, and it must not be removed, for three reasons:

1. It fires **earlier** (`:2941`, during total-cost computation) than the new guard (`:3507`), so
   it wins the race and owns the error message.
2. `mechanics_m_z/spree.rs:852-857` asserts the exact substring `"spree spell requires at least
   one mode"`. Removing the guard would change the message and break that assertion.
3. It cites **CR 702.172a**, a keyword-specific rule the general CR 601.2b/700.2a message does
   not carry. Spree's "choose one or more additional costs" is a distinct requirement from the
   printed `min_modes`.

Verification only: confirm `spree.rs::test_spree_insatiable_avarice_zero_modes_rejected` still
passes with its message assertion intact, and that
`spree.rs::test_spree_non_spree_spell_unchanged` (`:626-683`, which casts a **non-modal** plain
sorcery with empty modes) is unaffected.

### 3.4 The escalate exemption — decision and justification

**Decision: escalate gets a narrow exemption. The escalate tests and cards do NOT have to start
supplying explicit modes.** The exemption is keyed on `escalate_modes > 0` (the value bound at
`casting.rs:84`, populated from `AdditionalCost::EscalateModes { count }` at `:115`).

**CR footing**: CR 702.120a — "For each mode you choose beyond the first as you cast this spell,
you pay an additional [cost]". The additional cost the player elected to pay *is* an announcement
of how many modes were chosen. The engine records that announcement in
`AdditionalCost::EscalateModes { count }`, and resolution expands it to `0..=count`
(`resolution.rs:321-334`). So under escalate the mode **count** is player-announced and the guard
validates it (see Change 1); only the mode **identities** are engine-derived, and that residual is
`OOS-DP3-1`.

**A naive hard reject would kill the escalate path — enumerated:**

| escalate test | cast site | `escalate_modes` | outcome under this design |
|---|---|---|---|
| `test_escalate_single_mode_no_extra_cost` | `mechanics_e_l/escalate.rs:244` | **0** | **REJECTED — needs edit** (see §5) |
| `test_escalate_two_modes_one_extra_cost` | `escalate.rs:342` (`count: 1`, `:341`) | 1 | exempt, unchanged |
| `test_escalate_all_three_modes` | `escalate.rs:434` (`count: 2`) | 2 | exempt, unchanged |
| `test_escalate_insufficient_mana_rejected` | `escalate.rs:539` (`count: 2`) | 2 | exempt; still errors on mana, unchanged |
| `test_escalate_no_keyword_rejected` | `escalate.rs:603` (`count: 1`) | 1 | errors earlier at `casting.rs:2896-2902`, unchanged |
| `test_escalate_modes_paid_on_stack` | `escalate.rs:664` (`count: 1`) | 1 | exempt, unchanged |
| `test_escalate_modes_exceed_available_clamped` | `escalate.rs:750` (`count: 5`) | 5 | exempt; derived count clamps to `modes.len()` = 3 ≤ `max_modes` 3, unchanged |
| `test_escalate_modes_execute_in_printed_order` | `escalate.rs:831` (`count: 2`) | 2 | exempt, unchanged |
| `test_escalate_rejected_on_non_modal_spell` | `escalate.rs:954` (`count: 1`) | 1 | errors earlier at `casting.rs:2896-2902`, unchanged |

**Blast radius on `crates/engine/tests/mechanics_e_l/escalate.rs`: exactly one line** —
`:244`, `modes_chosen: vec![]` → `vec![0]`. The test's own doc comment already says
"escalate_modes=0 means only mode[0] executes", so the edit makes the test say out loud what it
was relying on the engine to guess. Its assertions are unchanged.

`primitives/pb_ac4_per_mode_targeting.rs:834` (`test_700_2c_702_120a_escalate_with_mode_targets_
rejected_at_cast`) casts with `count: 1` and empty modes and asserts the error message contains
both `"Escalate"` and `"mode_targets"`. **The exemption is what keeps this test green** — it falls
through Change 1 to the pre-existing hard-reject at `casting.rs:3670-3675`. Without the exemption
it would fail with the new CR 601.2b message. This is independent corroboration that the
exemption is the right call.

**Rejected alternative** (record it, don't do it): require explicit modes for escalate and
cross-check `modes_chosen.len() == escalate_modes + 1`. That is strictly more CR-faithful and
would also close `OOS-DP3-1`, but it changes escalate's *semantics*, not DP-4's, costs 9 test
edits plus 2 `partial` card defs (`blessed_alliance.rs`, `collective_resistance.rs`) plus golden
script `148`, and would break the PB-AC4 Finding-1 regression test above. It is a separate PB.

### 3.5 Summary table — what happens to each `modes_chosen` shape after PB-DP3

| spell/ability | `modes_chosen` | before | after |
|---|---|---|---|
| non-modal | `[]` | accept | accept (unchanged) |
| non-modal | `[0]` | reject | reject, same message (unchanged) |
| modal, entwine paid | anything | accept, all modes | accept, all modes (unchanged) |
| modal Spell, `min_modes ≥ 1` | `[]`, no escalate | **accept → resolves mode 0** | **REJECT (CR 601.2b/700.2a)** |
| modal Spell, `min_modes ≥ 1` | `[]`, escalate `n > 0` | accept, modes `0..=n` | accept, modes `0..=n`, derived count now bounds-checked |
| modal Spell, `min_modes == 0` | `[]` | **accept → resolves mode 0** | **REJECT, fail-safe (OOS-DP3-2)** |
| modal Spell | non-empty | full validation | full validation (unchanged) |
| modal Activated, `min_modes ≥ 1` | `[]` | **accept → resolves mode 0** | **REJECT (CR 602.2b/700.2a)** |
| modal Activated, `min_modes == 0` | `[]` | accept → resolves mode 0 | **accept → resolves NO mode** (behaviour flip, correct) |
| modal Activated | non-empty | full validation | full validation (unchanged) |
| modal Triggered | any | queue-time `vec![0]` | unchanged (OOS-DP3-4) |
| cascade / discover / free-cast stack builds | `[]` | resolution `vec![0]` | unchanged (DP-20 / OOS-DP3-3) |

---

## 4. Blast radius — enumerated, not estimated

Method: the universe of modal objects is (a) the **41** card-def files containing `min_modes:`
(`crates/card-defs/src/defs/`, verified by grep — 43 hits, 2 of which are comment lines in
`hullbreaker_horror.rs:8` and `:33`) and (b) the **9** test files that construct a
`ModeSelection` inline. Every consumer was then traced.

### 4.1 Engine source

| file:line | action |
|---|---|
| `crates/engine/src/rules/casting.rs:3507-3560` | **Change 1** — restructure |
| `crates/engine/src/rules/casting.rs:3613-3627`, `:3646-3654` | **Change 2** — comment only, keep the arm |
| `crates/engine/src/rules/casting.rs:2938-2945` | **Change 5** — no edit; verify |
| `crates/engine/src/rules/resolution.rs:335-341` | **Change 3** — comment only, keep the arm |
| `crates/engine/src/rules/abilities.rs:387-394`, `:402-403` | **Change 4** — restructure + comment |

### 4.2 Engine tests

| file:line | test | required edit |
|---|---|---|
| `crates/engine/tests/rules/modal.rs:527-567` | `test_modal_default_auto_selects_mode_zero` | **Rewrite.** It currently asserts empty `modes_chosen` auto-selects mode 0 on a `min_modes: 1` spell and gains 3 life. Invert it: assert `process_command` returns `Err`, and that the message contains `"at least 1 mode"` and `"601.2b"`. Rename to `test_601_2b_modal_empty_modes_chosen_rejected`. Update the module doc at `modal.rs:13` ("Backward compat: empty modes_chosen auto-selects mode[0]") to state the CR 601.2b rule instead. |
| `crates/engine/tests/mechanics_e_l/entwine.rs:349` | `test_entwine_not_paid_only_first_mode` (`:278`) | `modes_chosen: vec![]` → `vec![0]`. Add `// CR 601.2b: the mode is announced explicitly; the engine no longer picks mode 0 (PB-DP3).` Assertions unchanged (mode 0 fires, mode 1 does not). This is the "entwine **not** paid" test — the entwine-paid tests are untouched. |
| `crates/engine/tests/mechanics_e_l/escalate.rs:244` | `test_escalate_single_mode_no_extra_cost` (`:199`) | `modes_chosen: vec![]` → `vec![0]` + same CR 601.2b comment. `escalate_modes == 0` ⇒ not exempt. All 8 other escalate tests unchanged (§3.4 table). |

**Confirmed NOT affected** (each traced, do not touch):

- `crates/engine/tests/mechanics_m_z/spree.rs` — every cast supplies explicit modes except
  `:437` (already expects rejection, Spree guard) and `:671` (a **non-modal** plain sorcery).
- `crates/engine/tests/primitives/pb_ef7_modal_activated.rs` — helper `:86-98` always takes
  explicit modes.
- `crates/engine/tests/primitives/pb_os10_singleton_cleanup.rs` — all six Jitte activations pass
  `vec![0]` / `vec![1]` / `vec![2]`; `:194` is a **non-modal** cast; `:190` is a `StackObject`
  literal, not a command.
- `crates/engine/tests/primitives/pb_ac4_card_integration.rs` — helper `:114-145` always takes
  explicit modes; `:190` is a `StackObject` literal.
- `crates/engine/tests/primitives/pb_ac4_per_mode_targeting.rs:872` — escalate `count: 1`,
  exempt (and load-bearing, §3.4).
- `crates/engine/tests/primitives/pb_ac9_wheel_and_misc.rs:683` — extracts
  `incendiary_command`'s mode-3 effect and runs it directly; never casts.
- `crates/engine/tests/rules/modal_triggers.rs` — structural assertions on card defs only; no
  `process_command`, no `CastSpell`, no `ActivateAbility` (verified by grep).
- `crates/engine/tests/primitives/pb_os1_gain_control_reversion.rs` — only a doc-comment
  mention of `ModeSelection`.
- `crates/engine/tests/scripts/harness_equivalence.rs` — carries `modes` through at `:621`; no
  `ModeSelection` and no modal card anywhere in its scenario table.
- Every other `modes_chosen: vec![]` occurrence in `crates/engine/tests/` — ~200 sites across
  ~180 files — is a non-modal card. Confirmed by intersecting the two universes above.

### 4.3 Golden scripts

Search method: grep `test-data/generated-scripts/` for all 41 modal card names → 5 files; then
inspect each one's actions.

| file:line | current | required edit |
|---|---|---|
| `test-data/generated-scripts/stack/147_entwine_promise_of_power.json:91` | `"action": "cast_spell"` on **Promise of Power** (`min_modes: 1`) | add `"modes": [0]` to that action; append to its `note`: *"CR 601.2b: the mode is announced at cast time (PB-DP3); the engine no longer auto-selects mode 0."*; add `"601.2b"` to `metadata.cr_sections_tested`. **`review_status: approved` — this script runs.** |
| `test-data/generated-scripts/stack/148_escalate_blessed_alliance.json:84` | `"action": "cast_spell"` on **Blessed Alliance** (`min_modes: 1`, escalate, scenario 1 pays no escalate ⇒ `escalate_modes == 0` ⇒ not exempt) | identical treatment: add `"modes": [0]`, CR 601.2b note, `"601.2b"` in `cr_sections_tested`. The scenario-2 action at `:147` is `cast_spell_escalate` with `escalate_modes: 1` — **exempt, do not touch.** **`review_status: approved`.** |
| `test-data/generated-scripts/stack/169_modal_choice_abzan_charm.json` | `cast_spell_modal` with explicit `"modes"` | **no edit.** `review_status: "retired"` (`:27`) so it does not run, and it already supplies modes. |
| `test-data/generated-scripts/stack/173_spree_final_showdown.json:114` | `cast_spell_modal` with `"modes"` (`:116`) | **no edit.** |
| `test-data/generated-scripts/baseline/112_shambling_ghast_decayed_sacrifice.json` | modal **triggered** ability | **no edit.** Trigger path untouched. |

### 4.4 Replay harness

| file:line | action |
|---|---|
| `crates/engine/src/testing/replay_harness.rs:491` | `modes_chosen: vec![]` → `modes_chosen: modes_chosen.clone()` in the `"cast_spell"` arm (`:457`). Today `cast_spell` **silently discards** a script's `"modes"` field — the accepted-and-discarded-input class DP-24 flags. Update the arm's doc comment to say `cast_spell` honours `modes` for modal cards. |

**Safety check the runner must run before making this edit**: `grep -n '"modes"'
test-data/generated-scripts/` must return exactly 3 hits, all inside `cast_spell_modal` actions
(`169:142`, `169:247`, `173:116`). It does today. That makes the change a strict no-op for every
existing script, and the two edits in §4.3 then work without changing either script's action
string (preserving their access to `cast_spell`'s convoke/kicker/x_value fields).

If that grep ever returns a `"modes"` on a `cast_spell` action for a **non-modal** card, that
action would start failing (`"modes_chosen specified but spell has no modal structure"`) — fix
the script, do not revert the harness change.

`cast_spell_modal` (`:1798-1818`), `cast_spell_entwine` (`:1704`), `cast_spell_escalate`
(`:1729`) and `activate_ability` (`:717`, which already forwards `modes_chosen` at `:745`) need
no change.

### 4.5 Simulator (ESM criterion 5525)

`crates/simulator/src/random_bot.rs::action_to_command` (`:128-133`) is the **single chokepoint**
for both bots — `heuristic_bot.rs:127` calls it, and `driver.rs` / `local_game.rs` construct no
`CastSpell`/`ActivateAbility` themselves (verified by grep). `LegalActionProvider`/`StubProvider`
enumerate actions but never build commands.

**Minimal change** (do not exceed this — audit §6 flags StubProvider mode gaps as M11 risk R4 and
that is not PB-DP3's scope):

1. **New public helpers** in `crates/simulator/src/legal_actions.rs` (that file already uses both
   `state.card_registry()` at `:549` and `mtg_engine::rules::layers::calculate_characteristics`
   at `:319`/`:462`, so both lookups are already available to it):

   ```rust
   /// CR 601.2b / 602.2b / 700.2a (PB-DP3): the engine no longer auto-selects mode 0, so a
   /// bot must announce a legal mode set. Choose the first `min_modes` distinct indices in
   /// printed order — always legal (never duplicates, never out of range, never over
   /// `max_modes` since `min_modes <= max_modes`). Returns empty for a non-modal object,
   /// which is exactly what a non-modal cast/activation wants.
   pub fn default_modes_chosen(ms: &ModeSelection) -> Vec<usize>;
   /// Mirrors `casting.rs:3495-3506`'s `AbilityDefinition::Spell { modes: Some(..) }` lookup.
   pub fn spell_default_modes(state: &GameState, card: ObjectId) -> Vec<usize>;
   /// Mirrors `abilities.rs:313-331` — indexes the LAYER-RESOLVED `activated_abilities`
   /// list (CR 613.1f), not `def.abilities`. Getting this wrong is an index-namespace bug.
   pub fn ability_default_modes(state: &GameState, source: ObjectId, ability_index: usize) -> Vec<usize>;
   ```

   `default_modes_chosen` body: `(0..ms.min_modes.min(ms.modes.len())).collect()`.
   For `min_modes: 0` this returns `vec![]` — correct: a bot declining a "choose up to N"
   ability is legal, and after Change 4 the activated path resolves no mode, which is right.

2. **`crates/simulator/src/random_bot.rs`**:
   - `:130` — rename `_state: &GameState` → `state: &GameState` (the parameter already exists).
   - `:151` (`LegalAction::CastSpell` arm) — `modes_chosen: spell_default_modes(state, *card)`.
   - `:189` (`LegalAction::ActivateAbility` arm) — `modes_chosen: ability_default_modes(state,
     *source, *ability_index)`; replace the now-false comment at `:188` ("bots don't yet choose
     modes; empty auto-selects mode 0").
   - `:290` (mutate cast) and `:316` (morph cast) — apply `spell_default_modes` too. It returns
     `vec![]` for every non-modal card, so this is a no-op today and cannot regress; doing it
     uniformly removes the trap.

3. **Tests**: add unit tests to the existing `#[cfg(test)] mod tests` in `legal_actions.rs`
   (that file's own convention — see `:1370`, `:1430`, `:1616`), asserting
   `spell_default_modes` returns `[0, 1]` for a `cryptic_command` object, `[0]` for a
   `min_modes: 1` modal spell, and `[]` for a non-modal card.

**Do not** add mode enumeration to `LegalAction`, do not add a mode-choosing method to the `Bot`
trait, and do not touch `local_game.rs` or `driver.rs`.

### 4.6 TUI

`tools/tui` already depends on `mtg-simulator` (`tools/tui/Cargo.toml:29`), so it reuses the same
helpers:

| file:line | action |
|---|---|
| `tools/tui/src/play/input.rs:105` | `modes_chosen: Vec::new()` → `mtg_simulator::legal_actions::spell_default_modes(&app.state, *obj_id)` |
| `tools/tui/src/play/input.rs:199` | `modes_chosen: Vec::new()` → `ability_default_modes(...)`; replace the now-false comment at `:198` |

(These keep the TUI able to cast/activate modal objects at all. A real mode prompt is M11/M13
work, not this PB.)

### 4.7 Negative-space check

After the edits, `cargo test --all` must be green. **Any failure outside the enumerated set above
is an un-enumerated site: report it in the task comment and in the review file — do not silently
patch it.** The plan's blast-radius claim is falsifiable by construction and that is the point.

---

## 5. Card definition fixes

**None.** All 41 modal defs are already correct — the defect was that the engine did not require
what they declare. The 3 live-wrong cards (`cryptic_command`, `austere_command`,
`incendiary_command`) are fixed by Change 1 with **zero** edits to their files, and their
`Completeness::Complete` markers become true rather than aspirational.

**TODO sweep (roster-recall gate)**: grepped `crates/card-defs/src/defs/` for
`TODO.*mode`, `TODO.*min_modes`, `TODO.*modal`, `TODO.*choose one`. **Result: 0 cards with a TODO
naming this primitive.** Positive assertion — the gate was run and produced no additions.
(`blessed_alliance.rs:72-74` carries a TODO, but it names "up to two target creatures" /
`OptionalTarget`, a different primitive, and `akromas_will.rs:27` names an
`min_modes/max_modes` *expressiveness* limit — "cannot express 'if you control a commander as you
cast this'" — which is a conditional-mode primitive, also not DP-4.)

---

## 6. New card definitions

None.

---

## 7. Unit tests

**Primary file (new)**: `crates/engine/tests/primitives/pb_dp3_modal_mode_announcement.rs`
**Registration (mandatory, SR-9a)**: add `mod pb_dp3_modal_mode_announcement;` to
`crates/engine/tests/primitives/main.rs`, alphabetically **immediately after line 22**
(`mod pb_dp1_actor_priority;`). A missing `mod` line silently deletes the whole file's coverage.
**Never** create a new top-level `crates/engine/tests/*.rs`.

**Pattern to follow**: `crates/engine/tests/primitives/pb_ac4_card_integration.rs` — it already
loads real card defs, uses `enrich_spec_from_def`, and has a `cast_modal(state, player, name,
targets, modes_chosen)` helper at `:114-145` that can be copied verbatim.

**Gotcha (mandatory)**: `ObjectSpec::card()` creates naked objects — always call
`enrich_spec_from_def()` so the card's `mana_cost`/types/keywords are present, or the cast will
fail for the wrong reason and the probe will pass vacuously.

### 7.1 Fail-before / pass-after probes (the real DP-4 evidence)

Each of these must be shown failing against the pre-fix engine and passing after. Record the
before/after in the task comment.

| # | test | what it proves |
|---|---|---|
| 1 | `test_601_2b_cryptic_command_empty_modes_rejected` | `CastSpell{modes_chosen: vec![]}` on `cryptic_command` (`min_modes: 2`) returns `Err`; message contains `"at least 2 mode"` and `"601.2b"`. Also assert the caster's **mana pool is unchanged** and the card is **still in hand** — proving no cost was paid (CR 601.2b/601.2h ordering). |
| 2 | `test_601_2b_austere_command_empty_modes_rejected` | same, `austere_command`. |
| 3 | `test_601_2b_incendiary_command_empty_modes_rejected` | same, `incendiary_command`. |
| 4 | `test_601_2b_min_modes_one_modal_spell_empty_modes_rejected` | synthetic `min_modes: 1, max_modes: 1` spell — empty modes now rejected. This is the broad half of the fix (37 of the 41 defs), and it is the one the audit headline understates. |
| 5 | `test_601_2b_min_modes_zero_modal_spell_empty_modes_rejected_failsafe` | synthetic `min_modes: 0, max_modes: 1` **Spell** — rejected with the OOS-DP3-2 fail-safe message. Doc comment must state that CR 700.2a *permits* zero modes and that the rejection is an engine-representability fail-safe, not a rules claim. |
| 6 | `test_602_2b_modal_activated_ability_empty_modes_rejected` | `ActivateAbility{modes_chosen: vec![]}` on `goblin_cratermaker`'s modal ability returns `Err`; assert the activation cost (mana / the creature's tap state) is unspent. |
| 7 | `test_700_2a_modal_activated_min_modes_zero_empty_accepted_resolves_no_mode` | synthetic `min_modes: 0` **Activated** ability — activation succeeds and, after resolution, **neither** mode's effect occurred (assert both board consequences absent). Behaviour flip: before, mode 0 fired. |

### 7.2 Positive regression guards (pass before AND after — label them as such)

Criterion 5524 requires that a legal 2-mode cast still resolves **both** modes with real board
consequences, not just "no error".

| # | test | assertion (real state, no target declarations needed) |
|---|---|---|
| 8 | `test_700_2a_cryptic_command_modes_2_and_3_both_resolve` | modes `[2, 3]`. Mode 2 = tap all creatures opponents control; mode 3 = draw a card. Assert an opponent's creature is `tapped == true` **and** the caster's hand count is `+1`. Both modes' `mode_targets` slices are empty (`cryptic_command.rs:66-71`), so `targets: vec![]`. |
| 9 | `test_700_2a_austere_command_modes_0_and_1_both_resolve` | modes `[0, 1]`. Assert an artifact **and** an enchantment are in their owners' graveyards, and a control creature on the battlefield **survives** (proving modes 2/3 did not fire). `mode_targets: None`, no targets. |
| 10 | `test_700_2a_incendiary_command_modes_1_and_3_both_resolve` | modes `[1, 3]`. Mode 1 = 2 damage to each creature; mode 3 = `WheelHand`. Assert a 2/2 is in the graveyard **and** each player's hand was discarded-and-redrawn to its prior size. `mode_targets[1]` and `[3]` are both empty (`incendiary_command.rs:67-76`), so `targets: vec![]`. |
| 11 | `test_700_2a_out_of_range_index_rejected_as_sole_mode` | synthetic 3-mode spell, `modes_chosen: vec![7]` → `Err`, message contains `"out of range"`. |
| 12 | `test_700_2d_duplicate_mode_rejected_as_sole_pair` | synthetic, `allow_duplicate_modes: false`, `modes_chosen: vec![0, 0]` → `Err`. |
| 13 | `test_700_2a_max_modes_exceeded_rejected` | synthetic `min_modes: 1, max_modes: 1`, `modes_chosen: vec![0, 1]` → `Err`. |
| 14 | `test_702_42b_entwine_with_empty_modes_still_resolves_all` | synthetic entwine spell, `additional_costs: vec![AdditionalCost::Entwine]`, `modes_chosen: vec![]` → succeeds and **every** mode resolves. Pins the entwine short-circuit against regression. |
| 15 | `test_702_120a_escalate_with_empty_modes_unregressed` | synthetic escalate spell (`min_modes: 1, max_modes: 3`, 3 modes), `additional_costs: vec![AdditionalCost::EscalateModes { count: 1 }]`, `modes_chosen: vec![]` → succeeds and modes 0 **and** 1 both resolve. Pins the §3.4 exemption. |
| 16 | `test_702_120a_escalate_count_zero_requires_explicit_mode` | same card, `count: 0`, empty modes → `Err` with the CR 601.2b message. Pins the exemption's **boundary** — this is the case that forced the one-line `escalate.rs:244` edit. |

### 7.3 Simulator tests

Inline in `crates/simulator/src/legal_actions.rs`'s existing `#[cfg(test)] mod tests`:

| # | test | assertion |
|---|---|---|
| 17 | `test_dp3_spell_default_modes_cryptic_command` | `spell_default_modes` returns `vec![0, 1]` for a `cryptic_command` object (`min_modes: 2`). |
| 18 | `test_dp3_spell_default_modes_min_one` | returns `vec![0]` for a `min_modes: 1` modal card. |
| 19 | `test_dp3_spell_default_modes_non_modal_is_empty` | returns `vec![]` for a non-modal card (so the change is a no-op for every non-modal cast). |
| 20 | `test_dp3_ability_default_modes_uses_layer_resolved_index` | `ability_default_modes` on `umezawas_jitte` (`JITTE_MODAL_ABILITY_INDEX == 0`) returns `vec![0]`, exercising the `calculate_characteristics` path rather than `def.abilities`. |

**End-to-end smoke** (not a committed test, run once and record the result): run the fuzzer
(`crates/simulator/src/bin/fuzzer.rs`) for a handful of seeds before and after, and confirm no new
`InvalidCommand` rejections mentioning "mode" appear. Note that `driver.rs` answers a rejected
command with a silent `PassPriority`, so a regression here is *silent* — the unit tests above are
the real gate, and the fuzzer is corroboration only.

### 7.4 Existing tests to edit (repeat of §4.2, for the runner's checklist)

- `crates/engine/tests/rules/modal.rs:527-567` — rewrite to a rejection probe, rename, fix the
  module doc at `:13`.
- `crates/engine/tests/mechanics_e_l/entwine.rs:349` — `vec![]` → `vec![0]` + CR 601.2b comment.
- `crates/engine/tests/mechanics_e_l/escalate.rs:244` — `vec![]` → `vec![0]` + CR 601.2b comment.

---

## 8. Verification checklist

- [ ] `cargo check -p mtg-engine` clean after Change 1 + Change 4
- [ ] `cargo build --workspace` clean (catches the TUI and simulator arms — the
      `replay-viewer view_model.rs` / TUI `stack_view.rs` exhaustive-match trap does **not**
      apply here since no enum changed, but build the workspace anyway)
- [ ] `cargo test --all` green — **no failure outside the §4 enumeration** (§4.7)
- [ ] `cargo clippy --workspace -- -D warnings` clean
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35 — `cargo fmt` checks none of
      the 1,798 card defs; `cargo test --all` runs the script via `core card_defs_fmt`)
- [ ] `PROTOCOL_VERSION == 27` (`rules/protocol.rs:260`) and `PROTOCOL_SCHEMA_FINGERPRINT`
      unchanged; `HASH_SCHEMA_VERSION == 63` — assert by running the existing parity tests, do
      not edit either constant
- [ ] The two golden scripts (`147`, `148`) pass:
      `SCRIPT_FILTER=147 cargo test --test run_all_scripts -- --nocapture` and same for `148`.
      **Do not start the replay-viewer HTTP server** — agent-launched, it gets SIGKILL (137).
- [ ] All 7 fail-before probes demonstrated failing on the pre-fix engine (record in the task
      comment)
- [ ] Spree message assertion (`spree.rs:854`) and PB-AC4 Finding-1 message assertion
      (`pb_ac4_per_mode_targeting.rs:886`) both still pass unmodified
- [ ] Audit rows updated per §10; seeds filed per §9
- [ ] `memory/primitive-wip.md` phase advanced; close-out appended to
      `memory/workstream-state.md`; CLAUDE.md "Current State" + "Last Updated" delta

---

## 9. Seeds to file in `docs/audits/decision-point-audit.md` §8.1

Per the §8.1 convention, seeds land in the audit (the suite's binding spec), **not** in
`memory/primitive-wip.md`, which the next `/implement-primitive` run overwrites wholesale.

| seed | finding | class | status |
|---|---|---|---|
| **OOS-DP3-1** | **Escalate derives a contiguous mode set `0..=count` from an empty `modes_chosen`.** CR 702.120a permits *any* set of `count + 1` distinct modes; `resolution.rs:321-334` always takes the first `count + 1` in printed order, so a Blessed Alliance escalated once can never be "gain 4 life + opponent sacrifices an attacker" (modes 0 and 2). PB-DP3 validates the derived *count* against `min_modes`/`max_modes` but deliberately leaves the *identities* alone, because that is escalate's semantics, not DP-4's. Fix = require explicit `modes_chosen` on escalate casts and cross-check `modes_chosen.len() == count + 1`. Blast radius: 9 tests in `mechanics_e_l/escalate.rs`, `pb_ac4_per_mode_targeting.rs:834` (whose error-message assertion depends on the current path), 2 `partial` defs (`blessed_alliance.rs`, `collective_resistance.rs`), golden script `stack/148`. No wire change. | correctness / agency loss | filed by PB-DP3 (`scutemob-151`) |
| **OOS-DP3-2** | **A modal *Spell* with `min_modes: 0` cast with zero modes announced is unrepresentable, and PB-DP3 hard-rejects it.** CR 700.2a permits announcing zero modes on "choose up to N"; the engine cannot express it because `resolution.rs:335-338` maps an empty `modes_chosen` on a Spell stack object to `vec![0]` and cannot distinguish "controller chose zero" from a free-cast that never announced anything. Latent — no such card exists (the corpus's only `min_modes: 0` object is the *triggered* `hullbreaker_horror.rs:59`), and the **activated** path handles it correctly already (`abilities.rs:491-503` leaves `embedded_effect` as the base effect). Fix needs a discriminator (`Option<Vec<usize>>` on `StackObject`, or a `modes_announced` flag) ⇒ **HASH bump**, so it is its own PB. | correctness, deferred (wire) | filed by PB-DP3 (`scutemob-151`) |
| **OOS-DP3-3** | **Six free-cast producers bypass mode announcement entirely.** `copy.rs:430` (cascade), `copy.rs:646` (discover) and `engine.rs:2112`/`:2176`/`:2686`/`:2853` build `StackObject { modes_chosen: vec![] }` directly, never touching `handle_cast_spell`, so PB-DP3's guard cannot reach them and they still auto-select mode 0 via `resolution.rs:335-338`. This is why that fallback must stay live. Already covered by **DP-20**; recorded here so the next reader of the resolution fallback knows who its callers are. Cross-reference the §5 DP-20 row. | correctness (DP-20 scope) | filed by PB-DP3 (`scutemob-151`) |
| **OOS-DP3-4** | **Modal *triggered* abilities auto-select mode 0 at queue time, and the "choose up to one" branch is dead code.** `abilities.rs:8408-8421` sets `stack_obj.modes_chosen = vec![0]` for every modal trigger with at least one mode. Its `if min_modes == 0 { vec![0] } else { vec![0] }` at `:8410-8417` has **two identical branches** — the "choose up to one" case was written and then not honoured, so `hullbreaker_horror` (CR 700.2b, "choose up to one") always bounces something and can never decline. CR 700.2b also says "If no mode is chosen, the ability is removed from the stack", which the engine never does. Adjacent to **DP-6** (trigger-target auto-selection) and should be bundled with **PB-DP8**. | correctness / agency loss | filed by PB-DP3 (`scutemob-151`) |
| **OOS-DP3-5** | **The cast-time `ModeSelection` lookup is not face-aware.** `casting.rs:3495-3506` reads `def.abilities` directly rather than `def.effective_abilities(obj.is_transformed)` (the PB-OS4b/PB-RS4 contract) and ignores `def.adventure_face` / the aftermath half — while `resolution.rs:246-264` *does* consult the adventure face for modes. A modal DFC back face, or a modal adventure half, would validate against the wrong `ModeSelection` at cast time. Latent: no such card is in the corpus. Same root cause class as OOS-OS4-2 / OOS-RS-3. | correctness, latent | filed by PB-DP3 (`scutemob-151`) |

---

## 10. Audit bookkeeping — exact rows to update on close-out

File: `docs/audits/decision-point-audit.md`

| location | current | change |
|---|---|---|
| **§4.1, line 186** | `\| Mode announcement, **omitted** \| 601.2b / 700.2a \| **D** \| rules/casting.rs:3555-3559 — see **DP-4** \|` | class **D** → **A**, with the escalate caveat: *"**A** since PB-DP3 — an omitted mode announcement is rejected before costs are paid (`rules/casting.rs:3507-3560`). One CR-702.120a-scoped exemption survives: escalate with `count > 0` announces the mode *count* via `AdditionalCost::EscalateModes` and derives the identities `0..=count` (OOS-DP3-1). Free-cast producers that bypass `handle_cast_spell` are unaffected — DP-20 / OOS-DP3-3."* Update the site reference to `:3507-3560`. |
| **§4.2, line 214** | `\| Modes \| 700.2a \| **B** \| rules/abilities.rs:386-397 — empty ⇒ vec![0]; same min_modes bypass as **DP-4** \|` | class **B** → **A**; site → `rules/abilities.rs:337-398`; note: *"**A** since PB-DP3 — an omitted mode announcement on a modal activated ability is rejected before costs are spent (CR 602.2b/700.2a). `min_modes: 0` correctly accepts an empty announcement and resolves no mode."* |
| **§5, DP-4 row (line 431)** | class D, open | prefix `**SHIPPED (PB-DP3, `scutemob-151`).**` and record: the fix is a **validation lift**, not a bolted-on empty-check, so it also closes the broader `min_modes: 1` auto-select across the other 37 modal defs (the headline understated the scope); `abilities.rs`'s twin (§4.2) shipped in the same PB; the escalate exemption and its CR footing; the `resolution.rs:335-341` fallback is **retained** because six free-cast producers bypass `handle_cast_spell`; **no wire change** — PROTOCOL 27 / HASH 63 unmoved; blast radius was 3 test lines + 2 golden scripts + 1 harness line + the simulator/TUI callers, **0 card-def edits**. |
| **§5, DP-20 row (line 457)** | mentions "cascade free-cast also gets no targets and mode 0" | append a cross-reference to **OOS-DP3-3** naming the six producer sites, so whoever fixes DP-20 knows the resolution fallback is theirs to retire. |
| **§8, PB-DP3 row (line 572)** | proposal | → **SHIPPED (`scutemob-151`)**; confirm the predicted `wire impact: none`; note that the row's "Mirror the Spree guard at `casting.rs:2940-2944`" prescription was **not** followed literally — the Spree guard was kept intact (it owns the CR 702.172a message and fires earlier) and the general fix is a lift, not a mirror. |
| **§8.1** | seed table | append the five `OOS-DP3-*` rows from §9. |
| **§9, recommendation 4 (lines 702-706)** | *"`action_to_command_with_params` must reject an empty `modes_chosen` on a modal action whose `min_modes >= 1`, because the engine will not (**DP-4**)"* | annotate: *"**Superseded by PB-DP3** — the engine now rejects it. The M11 play server no longer needs a compensating check; it needs a mode-selection **UI**, and `crates/simulator`'s `spell_default_modes` / `ability_default_modes` are the placeholder until session 7 ships one."* |

Also update, per the DP-suite close-out convention: `CLAUDE.md` "Current State" (PB-DP3 SHIPPED,
test count delta, PROTOCOL 27 / HASH 63 unmoved) and "Last Updated"; append a worker close-out to
`memory/workstream-state.md` "PB-DP suite — worker close-outs" section; set
`memory/primitive-wip.md` phase → implement/review as the pipeline advances.

---

## 11. Risks & edge cases

1. **The escalate exemption is the load-bearing judgement call.** If the reviewer disagrees and
   wants strict rejection there, the cost is 8 more test edits plus golden script `148` scenario
   2, plus a rewrite of `pb_ac4_per_mode_targeting.rs:834`'s message assertion. Flag it in the
   review file rather than reversing it mid-implement.

2. **`resolution.rs:335-341` looks like dead code and is not.** The single highest-risk mistake
   in this PB is a runner "tidying up" that arm. Six producers depend on it (§ Change 3). It is
   also the reason `min_modes: 0` on a Spell is unrepresentable (OOS-DP3-2).

3. **Error-message coupling.** Three tests assert on message substrings
   (`rules/modal.rs:609`, `spree.rs:854`, `pb_ac4_per_mode_targeting.rs:886`). Reworded existing
   messages will break them for no benefit. Only the **new** messages are new text.

4. **The harness `cast_spell` change is behaviour-widening.** It converts a silently-discarded
   field into an honoured one. Guarded by the pre-edit grep in §4.4; if that grep's result ever
   changes, a previously-passing script starts failing loudly. That is the desired direction, but
   note it in the commit message.

5. **Bots silently lose modal casts if §4.5 is skipped.** `driver.rs` answers a rejected command
   with a silent `PassPriority`, so a missed simulator fix produces no error, no log line, and no
   test failure — only a subtle drop in bot action coverage. This is why criterion 5525 exists and
   why the `legal_actions.rs` unit tests (§7.3) are mandatory rather than nice-to-have.

6. **`ability_default_modes` must index the layer-resolved list.** `LegalAction::ActivateAbility
   { ability_index }` indexes `calculate_characteristics(...).activated_abilities`
   (`legal_actions.rs:462-464`), **not** `def.abilities`. Reading modes off `def.abilities` would
   be an index-namespace bug of exactly the class PB-RS4 spent a session closing.

7. **`min_modes: 0` asymmetry between the two paths is deliberate and must be documented in
   code.** Spell: hard-reject (unrepresentable). Activated: accept and resolve nothing (correct).
   A future reader who "makes them consistent" without reading OOS-DP3-2 will reintroduce silent
   wrong game state on the Spell side.

8. **Entwine on a non-modal spell.** Today `entwine_paid` short-circuits the whole block even
   when `mode_selection_opt` is `None`. Change 1's first arm preserves that exactly. Do not
   "improve" it — entwine's own keyword validation lives at `casting.rs:2845+` and is not this
   PB's business.

9. **`turn_number`/priority setup in the new probes.** Copy the `state.turn_mut().priority_holder
   = Some(player)` idiom from `pb_ac4_card_integration.rs:123`. PB-DP1 changed post-cast priority
   to the actor (CR 117.3c); the probes assert on rejection and on resolution, not on priority, so
   they are insulated — but multi-pass helpers must use the post-DP1 pass order.
