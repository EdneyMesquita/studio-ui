# Changelog

All notable changes to this crate are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The crate follows Semantic Versioning, with the
pre-1.0 rule that a minor version may contain breaking changes.

## [0.1.0] - Unreleased

Initial extraction from the author's Modbus desktop client.

### Added

- Theme: `theme::install`, the `DARK` and `LIGHT` color tokens, `theme::style`, and the `theme::identity`
  connection identity palette. Dark is the default theme.
- Metrics and type scale constants, and font helpers (`body`, `medium`, `semibold`, `mono`, `mono_medium`,
  `data`, `icon_font`).
- Fonts: Inter (Regular, Medium, SemiBold) and JetBrains Mono (Regular, Medium), embedded with their license
  texts.
- Icons: the Lucide icon font (153 glyphs), the `icons` glyph constants and `icons::ALL`; `theme::icon` and
  `theme::paint_icon`. The font is produced by `tools/icons/generate.py` from a pinned lucide-static release
  (1.56.0), whose tarball is checked against its SHA-512 integrity string before use.
- Widgets: `button`, `chart`, `check`, `chip`, `dialog`, `field`, `fit`, `grid`, `inspector`, `menu`,
  `metrics`, `notice`, `palette`, `rich_grid`, `sec`, `seg`, `switch`, `timeline` and `toast`, with the
  shared helpers in `widgets`.
- `dialog::guard`: the typed confirmation of a dangerous action.
- `timeline`: a protocol-neutral timeline. Lanes of spans and marks on a time axis with the "now" edge at the
  right; kinds that map onto theme colours (send, wait, receive, ok, warn, danger, idle, faint, a hatched
  timeout and a hatched gap); an automatic millisecond axis or the caller's ticks; two layouts (`DEFAULT`, the
  24 px rows of Mb Studio's bus timeline, and `SPACED`, the 26 px lanes of OPC Studio's publish timeline); a
  hover tooltip per item, click selection, and opt-in zoom and scroll that hand the new window back. The
  geometry is pure and tested. `cargo run --example timeline` draws a bus-like and a publish-like timeline.
- `PaletteConfig`, set on `PaletteState::config`: the search hint, the groups listed before anything is typed,
  the rows shown per group, the inline groups, and the text shown when nothing matches.
- `inspector::width` and `inspector::set_width`, so an application can save the inspector width between runs.
- The `studio_ui::egui` re-export: the egui this crate is built against.
- Cargo feature `dev-hooks` (off by default). With it, in debug builds, the environment variable
  `STUDIO_UI_OPEN_MENU=<key>` opens a named menu or the palette once, to make screenshots reproducible.
- Gallery example: `cargo run --example gallery` shows every widget section in both themes.
- `scripts/check-public.sh`, a public-readiness scan, run by a pre-commit hook (`.githooks/pre-commit`) and by CI.
- CI workflow: format, clippy, tests and example builds on Linux, macOS and Windows; the public-readiness check
  and a gitleaks secret scan on Linux. Manual only (`workflow_dispatch`) while the repository is private.
- `scripts/ci-local.sh`: the same checks on the developer's machine, run by the `pre-push` hook.
- `chrome`: the IDE frame both desktop apps drew by hand, protocol-neutral. `tree` (rows with twisty, identity
  chip or icon, badge, label with a filter highlight and an optional faint tag, marks, a link button and the
  "⋯" menu button, opt-in guides), `panel` (background, head with buttons and a count, search, note, empty
  line, scrolling body with an optional foot), `stripe` (tool buttons with an attention dot and a bottom
  slot), `tabs` (identity dot and underline, pane number, dirty dot, close, overflow menu, a trailing slot,
  drag to reorder), `toolbar` (mark, connection widget, breadcrumbs that clip before the centred search box,
  the identity tint, `record_button`, `theme_toggle`), `statusbar` (segments with tiered shrinking, the live
  dot, a badge, `pick_menu`) and `welcome` (the welcome screen, `watermark`, `pane_empty`); the shared
  layout maths in `chrome` (`clamp_side_width`, `shares`, `shrink_all`, `find_ci`, `repaint_after`). Each
  module's docs map Mb Studio's and OPC Studio's code onto it. `cargo run --example chrome` draws the
  whole frame with fake data.
- `hexview`: a protocol-neutral hex field view. `FieldTable` (the grid style of OPC Studio's trace inspector,
  `off │ field │ value │ bytes`, virtualised, or the mini style of Mb Studio's frame inspector, `bytes · field
  · value` with wrapping values, a check glyph and a faint note) and `HexBlock` (16-byte lines in a scroll
  area that lays out the lines in view only, or bytes wrapped at the width) over plain `Field`s: name, value,
  depth, byte range or none for a derived row, group, class, tone, check, varint. Byte classes are colour
  slots (`Plain`, `Header`, `Length`, `Id`, `Kind`, `Value`, `Status`, `Check`); the legend's words are the
  application's. The table and the block of one `id_salt` share their hover both ways, and a click on a byte
  hands back the innermost field under it; a varint's bytes are joined by a rule; `HexBlock::context` tints a
  selected field's parent group. `Fields` and `from_fn` lend an application's own rows without copying them.
  The layout maths and the field maths are pure and tested; the module's docs map OPC Studio's trace
  inspector and decoder, Mb Studio's frame inspector and the MQTT packet inspector onto it.
  `cargo run --example hexview` draws an OPC UA, a Modbus and an MQTT message.
- `RichGrid::scroll_to`, to bring a row into view.
- `granted`: a protocol-neutral requested → granted list. `GrantList` over plain `Grant`s (key, requested,
  granted, unit, what is in effect, a note) with a `State` the application works out (`Pending`,
  `AsRequested`, `Revised`, `Granted`, `Default`, `Unsupported`; `State::compare` for the simple cases), drawn
  as an inspector's `requested → granted unit` rows (OPC Studio's CreateSubscription inspector) or as a grid
  with a column per side, what is in effect and a note (the MQTT CONNECT → CONNACK table). A revised or
  unsupported value carries the mark (warn, medium weight); "not sent: as requested" and "absent: <default>"
  are the states' own words. `inline_pieces` and `column_pieces` are pure and tested; `granted_color` and
  `granted_font` serve grid cells that paint a granted value themselves. `cargo run --example granted`.
- `docview`: a protocol-neutral document view. `Document` draws a project file's toolbar (the seg of its views,
  the path with a tooltip, the unsaved-edits dot, the application's actions, Revert and Save, Ctrl+S; labels
  collapse to icons and marked actions move into "⋯" as Mb Studio's map toolbar did) and the validation strip
  of `Check`s; `source` and `source_inline` draw a text coloured by `Tok` spans (`source_job`, and
  `yaml_spans` for the family's YAML files); `diff` draws `git diff`-style lines with their counts;
  `WorkingCopy<T>` keeps the saved and the edited document, a revision, dirty, revert, saved and reload. The
  module's docs map Mb Studio's register map, OPC Studio's watch list and the MQTT topic map onto it.
  `cargo run --example docview`.

### Changed (relative to the source application)

- Element ids use the `studio-ui-` prefix (for example `studio-ui-inspector-width`, `studio-ui-palette` and
  `studio-ui-toast`). They previously used a different prefix.
- The palette's start groups were fixed; they are now set with `PaletteConfig`, together with the search hint
  and the empty-state text. Nothing in the palette is specific to one application any more.
- Renamed `theme::conn` to `theme::identity`.
- Renamed the syntax colors of `Theme`: `j_key`, `j_str`, `j_num`, `j_kw`, `j_bson` and `j_punct` are now
  `syn_key`, `syn_str`, `syn_num`, `syn_kw`, `syn_special` and `syn_punct`. `j_bson` became `syn_special`,
  not a different name for the same role.
- The `STUDIO_UI_OPEN_MENU` hook is compiled only with the `dev-hooks` feature, in debug builds.
