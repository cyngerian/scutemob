---
name: Feature branch workflow for PB batches
description: User prefers PB work done on a feature branch, merged to main at close
type: feedback
---

Use feature branches for primitive batch (PB) work — create branch before implementation, merge with `--no-ff` at close, then delete the branch.

**Why:** User explicitly requested this for PB-20. Keeps main clean during multi-commit work.

**How to apply:** When `/start-work W6-PB<N>` is claimed, create `w6-pb<N>-<slug>` branch. All impl + review-fix commits go on the branch. At close phase, `git merge --no-ff` to main and delete the branch. Watch for agent tools that reset to main — verify branch after agent runs.
