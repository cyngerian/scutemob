# PB-AC5 — Verified Scope (worker pre-plan pass)

Everything below was verified against source, not taken from the dispatch brief.
CR file: `.scryfall-cache/MagicCompRules.txt` (lives in the **main repo**, not the
worktree — it is gitignored). Effective **January 16, 2026**. It uses bare `\r`
line endings, so `grep '^702\.'` silently matches nothing; normalize with
`tr '\r' '\n'` before grepping.

## Corrections to the dispatch brief

### 1. Warp IS in the CR — do not derive from oracle text
The brief said "first confirm the cached CR file actually contains it; if absent,
derive behavior from oracle text + rulings." It is present. Cite the CR.

- **CR 702.185** Warp (also cross-referenced from 608.3g alongside Dash/Blitz)
- 702.185a: two static abilities functioning **while the card is on the stack**.
  "Warp [cost]" = "You may cast this card from your hand by paying [cost] rather
  than its mana cost" **and** "If this spell's warp cost was paid, exile the
  permanent this spell becomes at the beginning of the next end step. Its owner
  may cast this card after the current turn has ended for as long as it remains
  exiled." Alternative cost per 601.2b / 601.2f–h.
- 702.185b: "warped card in exile" = one exiled by the delayed triggered ability
  created by a warp ability. (Needs a distinguishing flag on the exiled object —
  a bare "in exile" bit is not sufficient.)
- 702.185c: "a spell was warped this turn" = cast for its warp cost this turn.
  (Needs a per-turn GameState flag, cleared at cleanup.)

Note the exile is a **delayed triggered ability at the beginning of the next end
step**, targeting *the permanent the spell becomes* — not the spell. So a warped
spell that never becomes a permanent (countered, fizzled) is never exiled. And
the recast is gated on "after the current turn has ended," not "any later turn."

### 2. Exert is a keyword ACTION (701.43), not a KeywordAbility
The brief specified `KeywordAbility::Exert (attack-declaration choice)`. That
models only one of the two shapes actually in our card defs.

- **CR 701.43a**: to exert a permanent, you choose to have it not untap during
  your next untap step.
- **701.43b**: a permanent can be exerted even if untapped, or exerted twice;
  all such effects expire during the **same** untap step.
- **701.43c**: an object not on the battlefield can't be exerted.
- **701.43d**: "You may exert [this creature] as it attacks" is an **optional
  cost to attack** (CR 508.1g). A "when you do" trigger printed in the same
  paragraph is **linked** to the static ability (CR 607.2h).

Two distinct call sites in our defs:
- `combat_celebrant.rs` — exert as optional attack cost + linked "when you do" trigger.
- `arena_of_glory.rs` — `{R}, {T}, Exert this land:` — exert as an **activation
  cost on a land**, nothing to do with attacking.

So the shape is a keyword *action* usable from (a) the 508.1g optional-attack-cost
hook and (b) an activation cost. A single `KeywordAbility::Exert` variant does not
cover `arena_of_glory`.

### 3. `KeywordAbility::DoesNotUntap` is NOT directly reusable
The brief says "PB-AC1 shipped a DoesNotUntap static, likely reusable." It is a
**static** keyword (discriminant 162, `types.rs:1685`, used by
`goblin_sharpshooter.rs`) meaning *never* untaps. Exert is a **one-shot** "doesn't
untap during your *next* untap step" that expires during that step, and stacks
(701.43b). Modeling exert by granting the static keyword would permanently lock
the permanent. Needs its own per-object state cleared during untap.
`turn_actions.rs:1205` is where the layer-resolved DoesNotUntap is computed —
the exert check belongs adjacent to it, not inside it.

### 4. Transmute confirmed
**CR 702.53a**: activated ability functioning **only while the card is in a
player's hand**. "Transmute [cost]" = "[Cost], Discard this card: Search your
library for a card with the same mana value as the discarded card, reveal that
card, and put it into your hand. Then shuffle your library. Activate only as a
sorcery." **702.53b**: the ability keeps existing in all other zones, so objects
with transmute count as "having an activated ability" for effects that care.

### 5. Alternative costs
**CR 118.9** confirmed. Enforcement constraint: **118.9a** — only **one**
alternative cost may be applied to a spell. **118.9c** — an alternative cost does
not change the spell's mana cost; effects reading mana cost see the original.

## Verified card roster (yield is ~8, not ~14)

Only these are actually unblocked by PB-AC5's four primitives:

| Card | Primitive | Note |
|---|---|---|
| `starfield_shepherd` | Warp | 4 markers |
| `timeline_culler` | Warp | Warp {B}, Pay 2 life — composite cost |
| `dimir_infiltrator` | Transmute | |
| `combat_celebrant` | Exert | attack cost + linked "when you do" trigger (607.2h) |
| `arena_of_glory` | Exert | exert as **activation** cost on a land |
| `force_of_will` | ExileFromHand | exile blue card **+ pay 1 life** (composite) |
| `force_of_vigor` | ExileFromHand | exile green card |
| `force_of_negation` | ExileFromHand | + exile-on-counter replacement (verify 2nd gap) |
| `force_of_despair` | ExileFromHand | **PARTIAL** — 2nd gap: "destroy all creatures that entered this turn" TargetFilter. Alt cost is also opponent's-turn-only. |

Every pitch card's cost is **colored-card-specific** (blue/green/black) and
sometimes composite (FoW adds 1 life; Timeline Culler adds 2 life). `Cost::ExileFromHand`
must carry a card filter, not just a count.

### Explicitly OUT of scope — different alt-cost shapes, do not touch
`gush` (return 2 Islands), `snuff_out` (land-conditional life payment),
`invigorate` (opponent gains 3 life), `flare_of_{denial,malice,fortitude,cultivation}`
(sacrifice a nontoken creature), `mindbreak_trap` (conditional free-cast + variable
targets), `bolass_citadel`, `endurance`/`grief` (markers are ETB-targeting gaps, not
pitch — their pitch cost may already work), `susurian_voidborn` (marker is a
death-trigger filter gap, not warp), `chrome_mox` (Imprint = ETB effect, not a cost),
`chaos_warp` (false positive — card name).

This is consistent with `feedback_pb_yield_calibration.md`: briefs overcount
in-scope cards 2–3×.

## Discriminant chain (verified from current code, not from the brief)

- Max `KeywordAbility` discriminant hashed: **162** (`DoesNotUntap`) → next free **163**
- `AbilityDefinition` variants: **68**
- `StackObjectKind` variants: **27**
- `HASH_SCHEMA_VERSION` = **31** (`hash.rs:235`) → must bump to **32**

## Existing infra to model against

- `AltCostKind` at `state/types.rs:113` — 20+ variants. **Foretell** is the closest
  model for Warp: it already carries exile-then-recast-later state
  (`game_object.rs:29` FaceDownKind foretold; `game_object.rs:789` "only be cast
  for its foretell cost after the current turn"). Study Foretell + Dash/Blitz
  (608.3g groups them with Warp) before designing Warp.
- Delayed triggered ability at beginning of next end step: find the existing
  Dash/Blitz "return to hand at end step" machinery — Warp's exile trigger is
  structurally identical.

## Hazards carried forward
1. `tools/replay-viewer/src/view_model.rs` matches `KeywordAbility` **and**
   `StackObjectKind` exhaustively; `tools/tui/src/play/panels/stack_view.rs`
   matches `StackObjectKind` exhaustively. Run `cargo build --workspace` after
   **every** impl phase — runners miss this ~50% of the time.
2. New mutable runtime fields (exerted flag, warp exile state, warped-this-turn)
   MUST get `HashInto` impls in `state/hash.rs` and bump `HASH_SCHEMA_VERSION`.
3. Harness: exert choice at declare-attackers, warp/transmute casts need
   `script_schema.rs` + `translate_player_action` wiring and `legal_actions.rs` arms.
4. Do not commit phantom `.claude/skills/*` deletions in this fresh worktree.
