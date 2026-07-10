# ROADMAP.md — Solarance Beginnings

> High-level roadmap. See `agents/milestones/proposed-roadmap.md` for detailed milestone definitions, exit gates, and scope. See `TASKS.md` for the running work checklist.

---

## Status

| Phase | Status |
|-------|--------|
| M0 — Movement Critical-Path Fix | ✅ Complete |
| M1 — Shared-Building Spike | ✅ Complete |
| M2 — Persistence + Welcome-Back | ✅ Complete |
| M3 — Two-Faction MVP Setup | ✅ Complete |
| M4 — Multi-Sector World Buildout | ✅ Complete |
| M5 — Mining Loop + Polish | 🟡 Partial (core loop + VFX + contribution flash done; sound cues pending) |
| M6 — MVP Launch & Devlog | ❌ Not started |
| M7 — Anti-Cheat Hardening via Views | ❌ Not started |

---

## Near-Term Focus (Current Branch: `main` — stable)

1. **M5 polish** — sound cues spike, completion animations
2. **Bug fixes** — #179 (modules on construction completion ✅), #176 (ore type in targeting ✅), login screen polish, cargo pickup UI
3. **QA smoke test** — verify core loop (spawn → fly → mine → contribute) end-to-end
4. **Monthly devlog** — M6 tracking (landing page, video, blog)

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
