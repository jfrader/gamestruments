# Repository agent notes

## Linear workflow

- Track project work in Linear, project **Gamestruments**:
  https://linear.app/gurisitosgames/project/gamestruments-049e669351cd
- New ideas are Linear issues in Backlog. Pick up issues, move `In Progress`,
  comment progress, move `Done` with a closing comment once merged into `main`.
- Feature-branch work on a tracked issue uses a worktree `gamestruments-GURI-N`
  beside this checkout. Do not switch this clone's branch for issue work.
- Read the `linear-workflow` skill before creating or updating any issue.

## Product

- This repo is a **game music library**, not a Pocket Circuit checkout.
- `@gamestruments/runtime` is MIT and Strudel-free. `@gamestruments/studio`
  and `apps/demo` are AGPL because they use Strudel.
- Frozen takes under `catalog/` are shipped music. Do not regenerate them when
  the generator version changes. Pocket Circuit main menu uses
  `catalog/pocket-circuit/tiny-torque-level-004/` (Grid).
- Do not keep iterating Pocket Circuit music unless Fran asks for a new catalog
  take. The next consumer work is Pocket Circuit GURI-321.

## Verify

```bash
npm run check
```

Requires Node.js 24. Final checks run in CI after push.

## Changelog

- User-visible changes go in root `CHANGELOG.md`.
- Read the `changelog` skill for what qualifies.
