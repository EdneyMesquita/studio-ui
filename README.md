# studio-ui

Design tokens, embedded fonts, the Lucide icon font and hand-painted [egui](https://github.com/emilk/egui)
widgets shared by the Studio family of desktop applications.

## What it is, and what it is not

It is:

- A dark and a light theme, expressed as color tokens and mapped onto `egui::Style`.
- Inter and JetBrains Mono, embedded, with named font helpers.
- The Lucide icon font (153 glyphs), with glyph constants.
- Widgets painted by hand to match the family's design in both themes: buttons, data grids, inspector panes,
  the command palette, charts, dialogs, menus, toasts, notices and form fields.

It is not:

- An application framework. It holds no app, protocol, device, storage or domain logic, and its API names
  describe the user interface, not any domain.
- A general-purpose component library, and not a replacement for egui's own widgets.
- Usable with other egui versions. It targets egui 0.36 only.

## Status

Pre-1.0 (`0.1.0`). The API may change between minor versions until 1.0; see
[CHANGELOG.md](CHANGELOG.md) for every change. The crate is not published to crates.io (`publish = false`).

## Install

Pin a git tag in the consuming application's `Cargo.toml`:

```toml
[dependencies]
studio-ui = { git = "https://github.com/EdneyMesquita/studio-ui", tag = "v0.1.0" }
```

The crate re-exports the egui it is built against as `studio_ui::egui`. Use that path for every egui type
and do not declare an `egui` dependency of your own, so the versions cannot drift apart. An application that
also runs on eframe must use an eframe 0.36 release, which is built on the same egui. Cargo unifies one egui
only when the versions are semver-compatible, so all family applications move to a new egui release together.

### Local development

To work on this crate and an application at the same time, point the application at a local checkout with
a `[patch]` section (the path is relative to the application's `Cargo.toml`):

```toml
[patch."https://github.com/EdneyMesquita/studio-ui"]
studio-ui = { path = "../studio-ui" }
```

Or replace the dependency line with `studio-ui = { path = "../studio-ui" }` while you work.

## Quick start

The snippets below assume `use studio_ui::egui;`. Install the fonts and both styles once, when the
application is created (this is how the gallery example starts):

```rust
use studio_ui::egui;

fn main() -> eframe::Result {
    eframe::run_native(
        "My app",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            // fonts and both styles, dark first: once, at startup
            studio_ui::theme::install(&cc.egui_ctx);
            Ok(Box::new(App::default()))
        }),
    )
}
```

`install` selects the dark theme. Switch with `ctx.set_theme(egui::Theme::Light)`. Widgets follow the active
theme on each frame.

Read colors from the theme in your own painting code:

```rust
let t = studio_ui::theme::theme(ui); // &'static Theme for the ui's current mode
ui.label(egui::RichText::new("Ready").color(t.text_2));
```

## Features

- `dev-hooks` (off by default). Lets `widgets::menu::dev_once` and `dev_open` open a named menu or the command
  palette when the environment variable `STUDIO_UI_OPEN_MENU` names its key, so that screenshots and
  end-to-end scripts can reach a popup. It works only in debug builds with the feature enabled; otherwise
  the functions return `false` and cost nothing. The crate asks with the keys `palette` and `qselect:<label>`.
  Applications choose their own keys for their own triggers.

## Tokens and metrics

### Color tokens

`studio_ui::Theme` is a struct of colors. Two constants exist, `theme::DARK` and `theme::LIGHT`, and they
have the same fields.

| Group | Tokens | Used for |
|-------|--------|----------|
| Surfaces | `editor`, `panel`, `raised`, `seam` | Editor canvas, side panels, popups and dialogs, dividing seams |
| Lines | `line`, `line_soft` | Borders and hairline rules |
| Interaction | `hover`, `row_hover`, `sel`, `sel_soft`, `field`, `field_line` | Hover fills, selected rows, input fields |
| Text | `text`, `text_2`, `text_3` | Primary, secondary and faint text |
| Accent | `accent`, `accent_hover`, `accent_text`, `on_accent` | Primary actions, links and accented text, text on accent |
| Status | `ok`, `warn`, `danger` | Success, warning and error |
| Syntax | `syn_key`, `syn_str`, `syn_num`, `syn_kw`, `syn_special`, `syn_punct`, `comment` | Highlighting of structured data and code |
| Chart | `s1` to `s4`, `Theme::series(i)` | Series colors, wrapping around after four |
| Overlay | `scrim`, `shadow` | Dialog scrim, shadow of floating layers |

The connection identity palette is `theme::identity`: the eight colors a user can give a connection
(`identity::ALL`), and `identity::ON_CHIP`, the color of the initials drawn on them. It is the only identity
hue in the family's interface.

`theme::mix` and `theme::alpha` compute the CSS-style `color-mix` and opacity that the designs use.

### Metrics

| Constant | Value | Meaning |
|----------|-------|---------|
| `FS`, `FS_SM`, `FS_XS` | 13, 12, 11 | Body, small and extra-small text size |
| `FS_DATA` | 12.5 | Data text (monospace) |
| `ROW` | 24 | Row height of the grids and minimum interactive height |
| `CONTROL` | 28 | Height of segmented controls and query-bar fields |
| `FIELD` | 32 | Height of text inputs |
| `TAB_ROW` | 36 | Tab row height in application layout |
| `BAR` | 40 | Toolbar height in application layout |
| `STATUS` | 26 | Status bar height in application layout |
| `INDENT` | 16 | Indent step of nested rows in inspector panes |
| `R_XS`, `R_SM`, `R`, `R_LG` | 3, 4, 6, 10 | Corner radii |
| `ICON`, `ICON_SM`, `ICON_XS` | 16, 14, 12 | Icon sizes |

`TAB_ROW`, `BAR` and `STATUS` are for application layout; the widgets in this crate do not use them.

### Fonts

| Helper | Font | Use |
|--------|------|-----|
| `theme::body()`, `theme::ui_font(size)` | Inter Regular | Chrome text |
| `theme::medium(size)` | Inter Medium | Labels, names, buttons |
| `theme::semibold(size)` | Inter SemiBold | Titles and panel heads |
| `theme::data()`, `theme::mono(size)` | JetBrains Mono Regular | All data |
| `theme::mono_medium(size)` | JetBrains Mono Medium | Metrics and values that need weight |
| `theme::icon_font(size)` | Lucide | Icon glyphs |

## Widgets

Every module is under `studio_ui::widgets`.

| Module | Provides |
|--------|----------|
| `button` | `Btn`, a builder used with `ui.add` (default, primary, ghost, danger, small, keycap hint, icon); `icon_button`, a square icon button with a tooltip |
| `chart` | `Chart` and `Series`: line chart with several series, time axis, legend with current values and a hover tooltip |
| `check` | `checkbox`, `checkbox_sm`: checkbox with a label |
| `chip` | `chip` (connection chip with initials), `dot`, `paint_live_dot`, `status_pill`, `Status`, `ChipSize` |
| `dialog` | `Dialog`: modal with title, icon, body and footer; `field_row` for labelled rows; `guard` for the typed confirmation of a dangerous action |
| `field` | `qfield` and `qselect` (query-bar fields; `qselect` returns whether the selection changed), `search_field`, `text_input`, `text_input_invalid`, `password_input` |
| `fit` | `Fit` and the `row` and `flow` helpers: toolbars that collapse to icons, then move items into a menu |
| `grid` | `Grid` and `Column`: virtualized data grid with a sticky header, an optional sticky first column, and click, double-click and right-click per row |
| `inspector` | `Inspector`: resizable right-hand pane with head, body and optional foot; one width shared by all inspectors (`width`, `set_width`) |
| `menu` | Items (`item`, `item_disabled`, `danger`, `option`, `conn_item`, `open_conn_item`), `sep`, `head`, and popups (`context`, `dropdown`, `select_popup`, `select_popup_scroll`, `is_open`) in the family frame |
| `metrics` | `metrics` and `Metric`: strip of tonal cells with hairline rules |
| `notice` | `notice` and `Level` (info, warn, crit): tinted strip at the top of a view |
| `palette` | `palette`, `PaletteState`, `PaletteConfig`, `PaletteItem`: command palette with grouped, ranked items |
| `rich_grid` | `RichGrid` and `RichCell`: virtualized grid whose cells paint text runs, bars, pills and buttons |
| `sec` | `sec_h`, `sec_h_with`, `kv`: section heads and key/value rows for inspector panes |
| `seg` | `seg`, `seg_fit`, `seg_stretch`, `seg_width`, `SegItem`: segmented control |
| `switch` | `switch`: toggle with a title and a description |
| `toast` | `Toast` and `toast::show`: transient confirmation at the bottom right, shown for four seconds |

Examples in each module's documentation show the call patterns. The gallery example shows every section in
both themes, with the calls that build each one: `cargo run --example gallery`.

### Command palette

The palette's wording and limits belong to the application. Set them once on `PaletteState::config`:

| Field | Default | Meaning |
|-------|---------|---------|
| `hint` | `Search actions and items` | Placeholder of the search field |
| `start_groups` | `["Actions", "Connections"]` | Groups listed before anything is typed, in this order; other groups answer only a search |
| `shown_per_group` | `40` | Rows listed per group |
| `inline_groups` | `["Actions"]` | Groups whose detail follows the label, instead of sitting at the right edge |
| `empty` | `Nothing matches “{}”.` | Shown when nothing matches; `{}` is replaced by the query |

The crate does not bind the key that opens the palette. The application calls `PaletteState::open` and then
`palette::palette(ctx, &mut state, &items)` each frame. It returns the index of the chosen item. Enter
chooses, the arrow keys, Ctrl J, Home and End move the selection, and Esc or a click outside closes it.

### Dialogs

Footer buttons are laid out from the right, so the first button added is the rightmost. `footer_start()`
lays them out from the left in the order added. `Dialog::show` returns `close: true` when Esc, the scrim or
the close button was used; the caller then drops the dialog.

`dialog::guard(ui, color, initials, name, reason, word, &mut buf)` draws the typed confirmation of a dangerous
action: a danger-tinted box that names the target (its chip and name), says why it is guarded, and holds an
input that must equal `word`. It returns whether the input matches, and the input's response.

### Inspector width

The inspector width is kept in egui's temporary data and is not persisted by this crate. It is 380 by
default and 280 to 640 by a drag on the edge; a double-click resets it. An application that wants to keep it
reads `inspector::width(ctx)` when it saves its settings, and calls `inspector::set_width(ctx, w)` at start.

### Shared helpers

`widgets/mod.rs` also provides the shared helpers: `text`, `para`, `text_lh`, `galley`, `hline` and `vline`
(crisp 1 px rules), `tip` and `tip_at` (the family's tooltip, used instead of `on_hover_text`), `strip` (a
fixed-height toolbar row), and the keyboard focus rings `focus_ring` and `focus_ring_inset`. Call
`widgets::track_focus_visible(ctx)` once per frame so focus rings appear only after keyboard navigation.

## Icons

`studio_ui::icons` holds the glyph of each icon as a string constant (for example `icons::SAVE`,
`icons::CIRCLE_CHECK`). `icons::ALL` lists every glyph as `(lucide name, glyph)`. Draw them with the icon font:

```rust
let t = studio_ui::theme::theme(ui);
studio_ui::theme::icon(ui, studio_ui::icons::SAVE, studio_ui::theme::ICON, t.text);
```

or, when you have a painter and a center point, `theme::paint_icon(painter, center, glyph, size, color)`.

The font is a subset of the pinned lucide-static release (1.56.0) that holds the 153 icons listed in
`tools/icons/icons.txt`. The glyph list in `src/icons.rs` and the subset in `assets/fonts/lucide.ttf` must
stay in step. Both are produced by `tools/icons/generate.py`; see [tools/icons/README.md](tools/icons/README.md)
for adding an icon. Lucide is licensed under ISC, and the icons derived from Feather are also under the MIT
License; see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Rules

- Widgets take colors only from `Theme` tokens. Do not write hex values in widgets.
- Every change must look right in both themes. The dark theme is the default.
- Fonts are chosen with the helpers above, not by naming font families directly.
- Widgets are painted by hand. Do not rely on egui's stock button or checkbox visuals, which are global.

Contributor rules are in [CONTRIBUTING.md](CONTRIBUTING.md). Architecture notes are in
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Planned work and known gaps are in
[docs/ROADMAP.md](docs/ROADMAP.md).

## Development

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --examples
bash scripts/check-public.sh
```

`bash scripts/ci-local.sh` runs all of these plus a gitleaks scan; the `pre-push` hook runs it. While the
repository is private the GitHub workflow runs only when started by hand. When enabled, it runs the format,
clippy, test and example builds on Linux, macOS and Windows. The Linux build installs
the development packages the examples need (xkbcommon, Mesa GL, Wayland, X11, Xcursor, Xrandr and Xi). The public-readiness check and a
gitleaks secret scan run on Linux, in their own jobs.

## Security

Report vulnerabilities privately, as described in [SECURITY.md](SECURITY.md).

## License

MIT. See [LICENSE](LICENSE). The bundled fonts and icons are under their own licenses: the SIL Open Font
License 1.1 for Inter and JetBrains Mono, and ISC (with MIT for the Feather-derived icons) for Lucide. They are
listed in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
