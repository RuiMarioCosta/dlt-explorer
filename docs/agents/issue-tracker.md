# Issue tracker

Issues live in **GitHub Issues** on [`RuiMarioCosta/dlt-explorer`](https://github.com/RuiMarioCosta/dlt-explorer/issues).

## CLI

Use the [`gh` CLI](https://cli.github.com/) for all issue operations:

- `gh issue create --title "…" --body "…" --label "…"`
- `gh issue list --state open`
- `gh issue view <number>`
- `gh issue edit <number> --add-label "…"`
- `gh issue close <number>`

## Conventions

- One issue per deliverable slice (vertical, demo-able when possible).
- Title format: imperative verb phrase (e.g. "Add streaming DLT parser").
- Labels: use triage labels from `docs/agents/triage-labels.md` plus any domain labels you add.
