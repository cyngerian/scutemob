# PB-DX55 — implementation plan

Three halves, ONE batch. Written after a stage-0 census whose findings are in
`pb-DX55-execution-notes.md` §0. **Every site list below was DERIVED at HEAD, and every one of
them is a CEILING with its bounding method stated** — a floor is what this queue keeps shipping.

---

## Half 1 — `OOS-SIM6-3`: auto-tap covers `CastSpell` alone

### The defect

`LocalGame::auto_tap_commands_for` (`crates/simulator/src/local_game.rs:1111-1113`) opens

```rust
let Command::CastSpell(cast) = command else { return None; };
```

so on BOTH paths — `advance()` (`:884`, bot) and `submit()` (`:1001`, human, and therefore the
browser) — every other command is applied with whatever is already floating. The offer gate
`legal_actions::can_afford` (`:2314`) answers *pool **+ untapped sources*** via
`mana_solver::solve_mana_payment_with_pool`; the engine charges the **pool**
(`abilities.rs:864-867`). Offer and acceptance disagree: **18 `InsufficientMana` refusals** on the
A/B seeds, and a browser human activating a mana-cost ability gets a 422 unless they happened to
have floating mana.

### The census — mana-charging `Command` variants, bounded as a CEILING

`ManaPool` (`card-types/src/state/player.rs:51`) has exactly three state-reducing methods:
`spend` (`:195`), `spend_restricted` (`:101`, called only from inside `can_spend`/`spend`) and
`empty` (`:272`, CR 500.4, not a payment). **So every mana charge flows through `.spend(`**, and
`grep -rn '\.spend(' crates/engine/src crates/card-types/src` gives five non-test sites:
`mana.rs:383`, `engine.rs:2396`, `abilities.rs:867`, and `casting.rs:7869`/`:7877`
(`pay_cost` / `pay_cost_with_context`). Walking those five plus every caller of the two `pay_cost`
helpers up to the `Command` dispatch arms gives **24 mana-charging variants**.

Of those, the ones a human or bot can actually produce are bounded by
`params::action_to_command_with_params`, whose `Ok(Command::…)` arms are **24 variants** — the
intersection is what `auto_tap_commands_for` can ever see:

| Command | mana source | engine charge site | today |
|---|---|---|---|
| `CastSpell` | `legal_actions::effective_cast_cost_with_additional` + `x_value × x_count` | `casting.rs:4198` | **covered** |
| `ActivateAbility` | layer-resolved `ActivatedAbility.cost.mana_cost` + `x_value × x_count` | `abilities.rs:867` | **NOT covered — 18 refusals** |
| `TapForMana` | the mana ability's own activation `mana_cost` | `mana.rs:383` | not covered |
| `DeclareAttackers` | `queries::attack_tax_total` (CR 508.1h, already an engine query) | `combat.rs:750` | not covered |
| `TurnFaceUp` | the morph/megamorph/disguise cost for the chosen `TurnFaceUpMethod` | `engine.rs:2396` | not covered |
| `ActivateBloodrush` | the bloodrush activation cost | `abilities.rs:1993` | not covered |
| `PayEcho` / `PayCumulativeUpkeep` / `PayRecover` (`pay: true`) | the echo / per-counter × age / recover cost | `engine.rs:1169`/`:1381`/`:1651` | not covered, **and the offer is POOL-gated so it UNDER-offers** |

Everything else `action_to_command_with_params` can emit charges no mana
(`PassPriority`, `PlayLand`, `DeclareBlockers`, `OrderBlockers`, `Concede`, `TakeMulligan`,
`KeepHand`, `ChooseDredge`, `ChooseTriggerTargets`, `DiscardToHandSize`, `AnswerEffectChoice`,
`SaddleMount`, `ActivateLoyaltyAbility`, `ReturnCommanderToCommandZone`, `LeaveCommanderInZone`).
`ActivateLoyaltyAbility` is worth naming explicitly because it *looks* like it should: loyalty
costs are counters, never mana (`engine.rs:3762-4009` contains no payment site).

### Shape — ONE cost calculator, ONE solver, exhaustive, no wildcard

New `pub fn legal_actions::command_mana_cost(state, player, command) -> Option<ManaCost>`:
an **exhaustive `match` over `Command` with NO wildcard arm**, so a future mana-bearing variant is
a compile error until it is classified. Every arm that can charge mana returns the cost the
engine's own handler will charge, derived from the SAME helper the engine or the offer gate uses
(`effective_cast_cost_with_additional` for casts, `queries::attack_tax_total` for the attack tax,
the layer-resolved ability for activations) — **never a second copy of
`effective_cast_cost_with_additional`'s shape**. Every arm that cannot returns `None` with its
reason in one line.

`auto_tap_commands_for` collapses to:

```rust
let cost = legal_actions::command_mana_cost(&self.state, player, command)?;
mana_solver::solve_mana_payment_with_pool(&self.state, player, &cost)
```

which is **the same two calls `can_afford` makes**, so the offer gate and the plan cannot
disagree — SR-38 by construction rather than by two functions that happen to agree.

### The `CastSpell` arm must be BYTE-IDENTICAL to today

Its body is the existing code moved, not rewritten. In particular the solver flattens
hybrid/Phyrexian with **all-defaults** (`PipTracker::from_cost`), while the command may carry
non-default `hybrid_choices`. That asymmetry is PRE-EXISTING on the cast path; it is preserved
exactly here and **filed as an `OOS-DX55-N` seed** rather than fixed, so this batch's fuzz A/B
measures only what it changed. The NEW arms have no prior behaviour to preserve and therefore
flatten with the command's own choices where the command carries them (PB-DX44's rule: pass the
`Command`'s own values verbatim; `&[]` there is the wrong first draft).

### SR-38, both directions

- **Over-offer** (the defect): `can_afford` and the plan are now the same arithmetic, so an
  activation that is offered is one the tap plan can fund.
- **Under-offer** (the dual, and a real capability gap): `PayEcho` / `PayCumulativeUpkeep` /
  `PayRecover` gate `pay: true` on `casting::can_pay_cost(&pool, cost)` — pool ONLY
  (`legal_actions.rs:805`, `:834`, `:860`). With auto-tap covering them the gate becomes
  `can_afford`, and the offer-layer comment that says *"the engine's payment path reads only the
  pool (it never auto-taps)"* stops being true and is rewritten.

### Probes (each RED under an executed revert)

1. `LocalGame` + `HumanChoice`: **empty pool, untapped lands**, a human activates a mana-cost
   ability — accepted, and asserted **by resolution effect**, not by the offer.
2. The same through `POST /api/game/action`, with **no manual `TapForMana` first**.
3. A bot-path A/B: the `InsufficientMana`-on-activate class 18 → 0.
4. A mechanism gate: `auto_tap_commands_for` contains no `let Command::… else` narrowing, and
   `command_mana_cost`'s match has no wildcard arm (parsed from source, with a non-vacuity floor).

---

## Half 2 — `OOS-SIM5-3`: the blocker offer the engine refuses

### The defect, and the fact that reframes it

`handle_declare_blockers` (`combat.rs:1171`) applies **4 preamble guards + 26 per-pair guards +
2 batch guards**. `legal_actions.rs:1321-1352` mirrors **five** of them (step, active combat,
already-declared, controller, tapped) and approximates a sixth (is-a-creature, off RAW
characteristics where the engine uses `calculate_characteristics`). Everything else — CR 509.1a's
attacking-player exclusion, `CrossPlayerBlock`, every evasion keyword, menace, provoke — is
unmirrored, and **the offer's shape cannot express them**: `LegalAction::DeclareBlockers
{ eligible, attackers }` is a flat cross product that discards each attacker's `AttackTarget`.

**The engine already holds TWO hand-rolled copies of the per-pair restriction list inside one
function** — the per-pair loop (`:1202-1481`) and the provoke requirement's `continue`-shaped
mirror (`:1577-1717`), which re-checks the same ~19 predicates to decide whether a must-block
requirement is satisfiable. The two are not identical: the provoke mirror omits **phased-out**,
**`CrossPlayerBlock`** and the duplicate check. So "never a second hand-rolled copy" is not a
warning about the future — it describes HEAD, and the extraction removes an existing divergence.

### Shape — ONE predicate, THREE consumers

In `crates/engine/src/rules/combat.rs`, two new `pub fn`, re-exported through `rules::queries`
so the simulator consumes a query rather than reaching into the handler:

```rust
/// CR 509.1a-c: may `blocker` legally be declared blocking `attacker` for `player`?
pub fn check_block_pair(state, player, blocker, attacker, already_blocking: &[ObjectId])
    -> Result<(), GameStateError>;

/// The whole declaration, including CR 702.111b menace and CR 702.39a provoke.
pub fn validate_block_declaration(state, player, blockers: &[(ObjectId, ObjectId)])
    -> Result<(), GameStateError>;
```

- `handle_declare_blockers` calls `validate_block_declaration` and then mutates. Its per-pair loop
  and its provoke mirror both become `check_block_pair` calls. **The error values and their ORDER
  must not change** — the refusal messages are asserted in tests and read by the rejection
  channel, so the extraction is behaviour-preserving and is proven so by the suite.
- `queries::legal_blocks(state, player) -> Vec<(ObjectId, Vec<ObjectId>)>` answers, per
  controlled creature, which declared attackers it may block, using `check_block_pair`.
- `LegalAction::DeclareBlockers` gains `legal_blocks: Vec<(ObjectId, Vec<ObjectId>)>`;
  `eligible` becomes the blockers with a non-empty slice and `attackers` the union, so existing
  consumers keep working and the new field is what the bots and the browser pick from.
- The whole offer is suppressed when `player == combat.attacking_player` (**CR 509.1a**,
  `OOS-DX51-3`) and when `legal_blocks` is empty.
- `random_bot::action_to_command`'s and `heuristic_bot`'s blocker arms pick from `legal_blocks`,
  then the assembled declaration is passed through `validate_block_declaration` and pruned until
  it is legal — which is what covers the SET-level guards (menace, provoke) a per-pair predicate
  structurally cannot. **No repeat cap and no retry loop** (PB-DX21 deleted that shape); the prune
  is a single deterministic pass over the chosen pairs.

### A CR cite correction the extraction must carry

`combat.rs:1271` justifies the "an attacker must be attacking the declaring player (or their
planeswalker)" guard with **CR 509.1c**. That is the wrong rule. CR 509.1c is the *requirements*
rule (*"…if the number of requirements that are being obeyed is fewer than the maximum possible
number…"*), which is correctly cited three lines over at `:1525`/`:1532`/`:1729` for provoke.
The rule that says an attacker must be attacking the blocking player is **CR 509.1a**, verbatim:
*"the defending player chooses one creature for it to block that's attacking that player, a
planeswalker they control, or a battle they protect."* Verified against the rules server.
Corrected here and filed, because this is PB-DX38's class landing on the exact guard this half
extracts.

### Probes

Per predicate — attacking-player, `CrossPlayerBlock`, flying — a fixture on which **the offer is
absent AND the engine refusal is absent**, asserted together, because either one alone is the
half that keeps shipping.

---

## Half 3 — `OOS-SIM5-5`: modal ACTIVATED abilities

### The defect, and the fifth copy

`queries::ability_target_requirements` (`queries.rs:223`) takes no chosen modes and returns
`ActivatedAbility.targets`. Its own doc (`:215-220`) calls the per-mode slice *"out of scope"* and
claims the flat list is *"the correct answer for the single-mode case"*. **That claim is false for
every member of the corpus**: all three modal activated abilities declare `targets: vec![]` and
put their requirements entirely in `mode_targets`, so the query returns `vec![]` for them on every
board, `targeting.rs:241` reads that as `NotTargeted`, `params.rs:459` still fills
`modes_chosen: [0]`, and `handle_activate_ability` refuses with *"modal spell with per-mode targets
requires exactly 1 target(s) for the chosen mode(s) but got 0 (CR 700.2c)"* — **22 refusals**, the
largest class at HEAD.

`casting::per_mode_target_requirements` (`casting.rs:5853`) has exactly four real call sites: two
spell-side (`casting.rs:3752`, `queries.rs:200`) and two trigger-side inside PB-DX35's
`trigger_modal_plan` (`abilities.rs:8770`, `:8776`). **`handle_activate_ability` re-derives its
body inline at `abilities.rs:439-455`** — same `debug_assert_eq!`, same
`flat_map`/`get`/`unwrap_or_default` — a fifth copy, and the one PB-DX35 did not consolidate.
A third copy is the defect; there are already five.

### The census — a CEILING, by the corpus rather than by grep

`grep -rn "modes: Some" crates/card-defs/src/defs/` gives **41** lines, none in a comment; a
brace-depth walk classifies them **3 Activated / 31 Spell / 7 Triggered**, and 3 + 31 + 7 = 41, so
nothing is unclassified. Both grant channels were checked too (the single bare `ActivatedAbility`
struct literal in the corpus, `bootleggers_stash.rs:48`, declares `modes: None`; no
`LayerModification::AddActivatedAbility` carries modes). The three are

| def | `Completeness` | per-mode targets |
|---|---|---|
| `cankerbloom` | **`Complete`** | mode 0 `TargetArtifact`, mode 1 `TargetEnchantment`, mode 2 none |
| `goblin_cratermaker` | **`Complete`** | mode 0 `TargetCreature`, mode 1 `TargetPermanentWithFilter` |
| `umezawas_jitte` (ability index 1) | **`Complete`** | mode 0 none, mode 1 `TargetCreature`, mode 2 none |

**All three are deck-legal `Complete`.** The v4 memo prices this seed at *2 refusals, 1.9%*; the
population is 3 deck-legal cards and the class is 22 refusals, 31.4%.

### Shape

1. `casting::ability_mode_selection(state, source, ability_index)` — the peer of the existing
   `spell_mode_selection`, reading the LAYER-RESOLVED `ActivatedAbility.modes` (the same place
   `handle_activate_ability` reads it, so the two cannot drift).
2. `queries::ability_target_requirements` gains a `modes_chosen: &[usize]` parameter and slices
   through **`casting::per_mode_target_requirements`** — the shared helper, not a sixth copy. It
   keeps `spell_target_requirements`' stated convention: no chosen modes on an ability that has
   `mode_targets` ⇒ `vec![]`, because advertising targets for an unchosen mode is worse than
   advertising none.
3. `handle_activate_ability`'s inline slice (`abilities.rs:439-455`) is **deleted** in favour of
   the shared helper. Behaviour byte-identical; the point is that there is one arithmetic.
4. `legal_actions::ability_default_modes` becomes **legality-aware**, the way PB-DX35 made the
   trigger side — but under **CR 700.2a**, not CR 700.2b. Those are different rules and the
   distinction is load-bearing: 700.2a governs *"a modal spell or **activated ability**"* and says
   only *"if one of the modes would be illegal (due to an inability to choose legal targets, for
   example), that mode can't be chosen"*; 700.2b governs a modal **triggered** ability and adds
   *"if no mode is chosen, the ability is removed from the stack"*. An activated ability is never
   removed from a stack it was never put on — if no mode is legal it simply cannot be activated
   (CR 601.2b via CR 602.2b), which is why the consequence here is an OFFER SUPPRESSION and not
   PB-DX35's `None`-means-remove. Verified against the rules server; **the plan's own first draft
   cited 700.2b**, which is PB-DX35's rule, not this one.
   The choice is the first mode whose slice has a candidate for every mandatory slot. It already has
   exactly two callers — `params.rs:459` and (after this batch) `targeting.rs` — so both sides
   agree by construction.
5. `targeting.rs`'s ActivateAbility arm passes `ability_default_modes(..)` instead of nothing,
   which is **literally what its CastSpell sibling already does** with `spell_default_modes`
   (`targeting.rs:169-175`), for the reason its own doc gives.
6. SR-38: if no mode is legal and `min_modes >= 1`, the activation is **not offered**.
7. `tools/play-server/src/view.rs:3330-3355` renders per-mode slots by reading `ms.mode_targets`
   directly — a sixth copy in the browser. It consumes the query instead.

### Probes

A bot announces and the engine ACCEPTS a per-mode target on a real modal activated ability from
`all_cards()` (all three are `Complete`, so the fixture uses a real def, not a synthetic one),
plus the 22 → 0 class measurement.

---

## Wire

Predicted **PROTOCOL 44 / HASH 85 UNMOVED, zero bumps** — reasoning per half in
`pb-DX55-execution-notes.md` §0.4. The load-bearing fact for half 3 is that
**`Command::ActivateAbility` already carries `modes_chosen`** (`command.rs:124`), so no command
field is added. Both gates are executed against the final tree.
