# ROADMAP.md — Solarance Beginnings

> High-level roadmap. See `agents/milestones/proposed-roadmap.md` for detailed milestone definitions, exit gates, and scope. See `TASKS.md` for the running work checklist.

---

## Status

| Phase | Status |
|-------|--------|
| M0 — Movement Critical-Path Fix | ✅ Complete |
| M4 — Multi-Sector World Buildout | 🟡 Partial (sectors + gates + nebula art done) |
| M1 — Shared-Building Spike | ❌ Not started |
| M2 — Persistence + Welcome-Back | ❌ Not started |
| M3 — Two-Faction MVP Setup | ❌ Not started |
| M5 — Mining Loop + Polish | ❌ Not started |
| M6 — MVP Launch & Devlog | ❌ Not started |
| M7 — Anti-Cheat Hardening via Views | ❌ Not started |

---

## Near-Term Focus (Current Branch: `feat/nebula-and-uc-station-art`)

1. **Wire nebula rendering** into sector background — ✅ done
2. **Wire UC station textures** into station dispatch — ✅ done
3. **Build and smoke test** — pending user QA sign-off
4. **Merge to main** — after user confirms gameplay is functional

---

## Next After Confirmation

1. `feat/mining-visual-effects` — add client-side laser beam and asteroid hit VFX (M5)
2. `feat/galaxy-map-tab` — galaxy overview in map window (M4 polish)
3. `feat/station-shields` — shield system in station status tick (M5)
4. Farm/laboratory module production implementations (M1 infrastructure)
5. M1 Shared-Building Spike proper: `contribute_to_station` reducer, construction-site UI, completion broadcast

---

## Branch Strategy

- Work in local feature branches (`feat/*`, `fix/*`, `chore/*`)
- Micro atomic commits per branch
- Only merge to main when stable and QA-verified
- Regular pulls from upstream (`GalaxyCr8r/solarance-beginnings.git`)

## Key Resources

| Resource | Location |
|----------|----------|
| Full milestone definitions | `agents/milestones/proposed-roadmap.md` |
| Issue triage & disposition | `agents/milestones/issue-disposition.md` |
| Work checklist | `TASKS.md` |
| AI coding context | `AGENTS.md`, `CLAUDE.md` |
| Devlog & design | `docs/` (GitHub Pages website) |
