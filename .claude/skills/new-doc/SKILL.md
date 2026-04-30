---
name: new-doc
description: Integrate a newly added document into the project's docs/ directory. Cross-references existing docs, updates CLAUDE.md, updates the roadmap, and identifies immediate action items.
argument-hint: <filename>
---

# /new-doc — Integrate a New Document into the Project

A new document has been added to the project. Your job is to integrate it with the existing
documentation ecosystem so everything stays synchronized.

**Input**: The filename provided as `$ARGUMENTS`. If no filename is provided, ask the user.

## Process

Run the following steps using **parallel agents** wherever possible. Steps 1 and 2 are
independent and should run in parallel. Steps 3-5 depend on the analysis results and run
after.

### Step 1: Read and Analyze the New Document (parallel with Step 2)

Read the new document. Extract:
- **Purpose**: What does this document define or propose?
- **Systems affected**: Which parts of the codebase or architecture does it touch?
- **Timeline items**: Does it define new milestones, tasks, or deliverables?
- **Immediate actions**: Does it require changes NOW vs. changes in a future milestone?
- **Internal references**: Does it reference other docs? Are those references correct (filenames match)?
- **Contradictions**: Does it contradict anything in existing docs?

### Step 2: Read All Existing Documents (parallel with Step 1)

Read all documents in `docs/` and `CLAUDE.md`. Build a map of:
- Cross-references between documents
- Architecture invariants
- Current milestone and status
- Key design decisions
- Active roadmap items

### Step 3: Move and Fix the Document

- If the document is NOT already in `docs/`, move it there
- Fix any internal references (wrong filenames, broken cross-references)
- Ensure the document follows the project's naming convention: `mtg-engine-<topic>.md`

### Step 4: Update Existing Documents (parallel edits where independent)

Update these documents to integrate the new one. Make minimal, targeted edits:

**CLAUDE.md**:
- Add the new doc to the **Primary Documents** table
- Add relevant entries to the **Key Design Decisions Log** (if applicable)
- Update **Architecture Invariants** if the new doc changes any (note: invariants are "non-negotiable" — only update if the new doc explicitly supersedes something)
- Update **What's Next** section if the new doc creates immediate action items
- Update **Common Pitfalls & Gotchas** if the new doc introduces any

**Architecture doc** (`docs/mtg-engine-architecture.md`):
- Add forward references to the new doc in relevant sections
- If the new doc supersedes or evolves a section, add a note (don't delete the original — mark it as "superseded by" or "evolved in")

**Roadmap** (`docs/mtg-engine-roadmap.md`):
- If the new doc defines new milestones, add them
- If the new doc adds deliverables to existing milestones, add them
- Update the milestone overview diagram and dependency graph if affected
- Update the risk register if the new doc introduces or mitigates risks

**Other docs**: If the new doc relates to corner cases, game scripts, or other existing docs, add cross-references as appropriate.

### Step 5: Identify and Report Immediate Actions

After all updates are made, produce a summary for the user:

1. **Document integrated**: Where it now lives and what references were updated
2. **Contradictions found**: Any conflicts between the new doc and existing docs (and how you resolved them)
3. **Immediate action items**: Changes that need to happen NOW (before or during the current milestone)
4. **Future action items**: Changes deferred to later milestones
5. **Files modified**: List of all files that were edited

## Guidelines

- **Don't delete existing content** — add notes, forward references, and evolution markers
- **Preserve the original authoritative host model** as a fallback when adding distributed verification
- **Use the project's existing style**: same table formats, same heading levels, same citation conventions
- **Cross-reference using relative paths**: `docs/mtg-engine-*.md` from CLAUDE.md, just filenames within `docs/`
- **Keep CLAUDE.md concise** — it's loaded into every session context. Put details in the referenced doc.
