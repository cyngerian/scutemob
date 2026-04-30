---
name: docs
description: Manage project documentation — initialize, check drift, show status
user-invocable: true
allowed-tools: Read, Bash, Glob, Grep, Edit, Write
argument-hint: "[init|check|status]"
---

# Documentation Management

Run `/docs` with a mode: `init`, `check`, or `status`.

- `/docs init` — Interactive setup: scan for existing docs, detect project features, scaffold new ones
- `/docs check` — Audit all docs for drift and missing sections
- `/docs status` — Quick health overview table

If no argument is given, default to `status`.

---

## Mode: `/docs init` — Interactive scaffold

Sets up documentation management by scanning for existing docs, detecting project features, and proposing a documentation structure.

### Procedure

#### 1. Read existing config

Read `.claude/docs.yaml` if it exists. If not, create a minimal starting point:

```yaml
docs_dir: docs/
templates: {}
```

Note which templates are already configured — skip these in later steps.

#### 2. Scan for existing documentation

Before detecting code features, look for docs that already exist in the repo.

**Glob for `*.md` files** in these locations:
- `docs/`
- `doc/`
- Project root
- `.github/`

**Exclude** files that are not project docs:
- `CLAUDE.md`, `LICENSE.md`, `CHANGELOG.md`, `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`
- `node_modules/**`
- Any file already declared in the config

**For each found file:**
- Read it and parse H2 headings (`## ...`)
- Infer which template category it maps to based on filename and headings:
  - File with "endpoint", "auth", "api" headings or in name → `api`
  - File with "install", "setup", "configure", "run", "getting started" → `setup`
  - File with "architecture", "overview", "tech stack", "structure", "design" → `architecture`
  - File with "deploy", "build", "docker", "production" → `deployment`
  - File with "test", "fixture", "coverage" → `testing`
  - File with "script", "utility", "automation" → `scripts`
  - File with "ci", "pipeline", "workflow", "github actions" → `ci`

**Present for adoption:**

```
## Existing documentation found

1. docs/api-reference.md — sections: Endpoints, Auth, Errors
   → adopt as "api" template? (triggers: src/api/**)
2. docs/getting-started.md — sections: Install, Configure, Run
   → adopt as "setup" template? (triggers: Dockerfile, package.json, ...)
3. README.md — sections: Overview, Usage, Contributing
   → skip (not a managed doc type)

Adopt all / select by number / skip?
```

**For adopted docs:**
- Add a template entry to `.claude/docs.yaml` using the **existing filename** (don't rename)
- Set `sections` from the actual H2 headings in the file
- Set `triggers` from the inferred template category
- Set `description` from the category
- Add `<!-- last_updated: YYYY-MM-DD -->` after the H1 heading if not present, using the file's last git commit date:
  ```bash
  git log -1 --format=%as -- <filepath>
  ```
- Do **not** restructure, rename, or rewrite the content

#### 3. Detect project characteristics

Check for features that could use documentation:

| Check | Detection |
|-------|-----------|
| API framework | `app/api/`, `src/routes/`, `src/api/`, FastAPI/Express/Actix imports |
| Docker | `Dockerfile`, `docker-compose*.yml` |
| Scripts | `scripts/`, `bin/` directories with >2 files |
| CI/CD | `.github/workflows/`, `.gitlab-ci.yml`, `Jenkinsfile` |
| Tests | `tests/`, `test/`, `pytest.ini`, `jest.config*` |
| Database | SQLite/Postgres/MySQL imports, migration files |
| Frontend | `src/components/`, `pages/`, `package.json` with React/Vue/Svelte |

**Skip categories already covered** by existing config or adopted docs.

#### 4. Propose new docs for remaining gaps

```
## Detected project features

Already configured: architecture, setup, api (adopted)

Additional features that could use documentation:
1. [x] tests/ directory (12 files) → testing.md
2. [ ] .github/workflows/ (3 workflows) → ci.md

Add all / select by number / skip?
```

Pre-check items that are high-confidence matches. The user selects which to add.

#### 5. Update config and scaffold

- Write the updated `.claude/docs.yaml` with both adopted and newly selected templates
- Scaffold new doc files only (not adopted ones) with section headings and TODO placeholders:

```markdown
# {Title}

<!-- last_updated: YYYY-MM-DD -->

## {Section 1}

<!-- TODO: Fill in {Section 1} -->

## {Section 2}

<!-- TODO: Fill in {Section 2} -->
```

#### 6. Report

```
## Docs initialized

Adopted: 2 existing docs (api-reference.md, getting-started.md)
Scaffolded: 1 new doc (testing.md)
Config: .claude/docs.yaml updated with 3 templates

Next: fill in TODO placeholders in the scaffolded docs by reading the codebase.
```

Remind the agent to fill in scaffolded docs by reading the project code.

---

## Mode: `/docs check` — Drift audit

Checks all configured docs for staleness and structural completeness.

### Procedure

#### 1. Read config

Read `.claude/docs.yaml`. If it doesn't exist, report:
```
No .claude/docs.yaml found. Run /docs init to set up documentation management.
```
And stop.

#### 2. Determine scope

- **If called standalone** (`/docs check`): check all templates regardless of frequency.
- **If called from `/done`**: only check templates with `frequency: task` (or no frequency, since task is default). Use `git diff --name-only main..HEAD` for the changed file list.
- **If called from `/end`**: check templates with `frequency: task` or `frequency: session`. Use the session's full commit range for changed files.

When scoped, only check templates whose trigger patterns match at least one changed file.

#### 3. Drift detection

For each template in the config:

1. **Check file exists.** If not, report as `MISSING`.

2. **Parse `last_updated`.** Look for `<!-- last_updated: YYYY-MM-DD -->` in the file.
   If not found, report as `UNTRACKED`.

3. **Check triggers against git.** For each trigger pattern, run:
   ```bash
   git log -1 --format=%as -- '<pattern>'
   ```
   If any trigger file was committed more recently than `last_updated`, the doc is `STALE`.

4. **If no triggers are newer**, the doc is `CURRENT`.

#### 4. Section validation

For each template, verify that every declared section heading exists in the doc:
```
for each section in template.sections:
  if "## {section}" not found in the file:
    report as MISSING SECTION
```

#### 5. Report

```
## Documentation check

| Doc | Status | Detail |
|-----|--------|--------|
| architecture.md | CURRENT | last updated 2026-04-05 |
| setup.md | STALE | Dockerfile changed 2026-04-07, doc last updated 2026-03-20 |
| api.md | MISSING SECTION | "Error Handling" section not found |
| testing.md | MISSING | file does not exist |

### Recommendations
- **setup.md**: Dockerfile changed — verify Prerequisites and Build sections are current
- **api.md**: Add missing "Error Handling" section
- **testing.md**: Run /docs init to scaffold, or create manually
```

#### 6. Act on findings

If stale docs are found, update them or explain why no update is needed (e.g., "the Dockerfile change was cosmetic, no doc impact"). When updating a doc, update the `<!-- last_updated: YYYY-MM-DD -->` comment to today's date.

---

## Mode: `/docs status` — Quick overview

A read-only summary of documentation health. No updates, no prompts.

### Procedure

#### 1. Read config

Read `.claude/docs.yaml`. If it doesn't exist, report:
```
No .claude/docs.yaml found. Run /docs init to set up documentation management.
```
And stop.

#### 2. Check each template

For each template:
- Does the file exist?
- Does it have a `last_updated` timestamp?
- How old is the timestamp?

#### 3. Report

```
## Documentation status

docs_dir: docs/
templates: 5 configured, 4 exist, 1 missing

| Doc | Exists | Last Updated | Age |
|-----|--------|--------------|-----|
| architecture.md | yes | 2026-04-05 | 2 days |
| setup.md | yes | 2026-03-20 | 18 days |
| api.md | yes | 2026-04-07 | today |
| deployment.md | yes | 2026-03-15 | 23 days |
| testing.md | no | — | — |
```

No drift analysis, no recommendations. Just a quick glance at what exists and how fresh it is.

---

## Template categories reference

When inferring triggers for adopted or detected docs, use these defaults:

| Category | Default triggers |
|----------|-----------------|
| `architecture` | `**/*.py`, `**/*.ts`, `**/*.rs`, `**/*.go` |
| `setup` | `Dockerfile`, `docker-compose*.yml`, `package.json`, `Cargo.toml`, `pyproject.toml`, `requirements*.txt`, `Makefile` |
| `api` | `src/api/**`, `app/api/**`, `src/routes/**` |
| `deployment` | `Dockerfile`, `docker-compose*.yml`, `.github/workflows/**`, `config/**` |
| `testing` | `tests/**`, `test/**`, `pytest.ini`, `conftest.py`, `jest.config*` |
| `scripts` | `scripts/**`, `bin/**` |
| `ci` | `.github/workflows/**`, `.gitlab-ci.yml`, `Jenkinsfile` |

## Notes

- The config file is `.claude/docs.yaml` (committed, durable config alongside skills and settings).
- The `.esm/` directory is for ephemeral state (worker.md, migration.json) — docs config does not go there.
- `frequency` supports `task` (default, checked on `/done`) and `session` (checked on `/end`).
- When updating a doc, always update the `<!-- last_updated: YYYY-MM-DD -->` comment.
- Section headings are case-sensitive and must match exactly.
