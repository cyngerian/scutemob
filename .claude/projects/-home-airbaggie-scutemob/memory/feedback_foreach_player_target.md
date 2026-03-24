---
name: ForEach EachPlayer uses DeclaredTarget
description: PlayerTarget::DeclaredTarget { index: 0 } is the correct pattern inside ForEach::EachPlayer loops — it references the iteration variable, not a declared target
type: feedback
---

`PlayerTarget::DeclaredTarget { index: 0 }` inside `ForEach::EachPlayer` is correct — it references the iteration variable, not a target from the `targets` vec.

**Why:** Flagged Geier Reach Sanitarium as a HIGH bug during F-4 review, turned out to be correct DSL usage. The pattern looks wrong (empty targets vec + DeclaredTarget) but ForEach reuses the DeclaredTarget variant for its loop variable.

**How to apply:** Don't flag this pattern as a bug during card def reviews. When reviewing ForEach effects, check that the inner effect uses DeclaredTarget { index: 0 } — that's the iteration player.
