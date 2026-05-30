# Domain docs

## Layout

**Single-context** — this repo has one domain boundary.

- `CONTEXT.md` at the repo root — domain language, bounded contexts, key entities.
- `docs/adr/` — architectural decision records (one file per decision, numbered sequentially).

## Consumer rules

Skills that read domain docs (`improve-codebase-architecture`, `diagnose`, `tdd`, `grill-with-docs`):

1. **Always read `CONTEXT.md` before proposing changes** — use its vocabulary, not synonyms.
2. **Check `docs/adr/` for prior decisions** — don't contradict an existing ADR without flagging it.
3. **If `CONTEXT.md` doesn't exist yet**, prompt the user to create one before proceeding with architecture-level work.
