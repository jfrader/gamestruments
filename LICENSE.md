# Licensing

Gamestruments is split at a deliberate licensing boundary.

- `packages/studio` and `apps/demo` use Strudel and are licensed under
  AGPL-3.0-or-later. See the [GNU AGPL](https://www.gnu.org/licenses/agpl-3.0.html).
- `packages/runtime` is an independent event and transition runtime licensed
  under MIT. It does not import or bundle Strudel.
- Exported scores, MIDI, and rendered audio are user content. Their use remains
  subject to the licenses of any samples or other source material they contain.

This prototype is not legal advice. The licensing layout must be reviewed
before a public release.
