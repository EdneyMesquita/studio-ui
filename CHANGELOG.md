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
  `metrics`, `notice`, `palette`, `rich_grid`, `sec`, `seg`, `switch` and `toast`, with the shared helpers in
  `widgets`.
- `dialog::guard`: the typed confirmation of a dangerous action.
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
