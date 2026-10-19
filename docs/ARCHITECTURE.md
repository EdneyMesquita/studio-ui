# Architecture

This document describes how the crate is organised and why. The public API is described in the README and in
the rustdoc of each module. Planned work and known gaps are in [ROADMAP.md](ROADMAP.md).

## Module layout

```
src/
  lib.rs           crate docs; re-exports Theme, DARK, LIGHT and egui
  theme.rs         color tokens (Theme, DARK, LIGHT), theme::identity, metrics, font families and helpers,
                   fonts(), style(), install(), paint_icon / icon
  icons.rs         Lucide glyph constants (one &str per icon) and ALL, the (name, glyph) pairs
  chrome/
    mod.rs         the IDE frame; shared layout maths: clamp_side_width, shares, shrink_all, find_ci,
                   repaint_after, NARROW
    tree.rs        tree rows of the side panel: node, Row, Lead, After, row_menu, geom
    panel.rs       the side panel's frame: background, head, search, note, empty, body
    stripe.rs      the tool stripe
    tabs.rs        TabRow: the editor tab row, the tab drag payload, pane numbers
    toolbar.rs     the main toolbar, record_button, theme_toggle
    statusbar.rs   the status bar's segments and their fit, live_ring, pick_menu
    welcome.rs     the welcome screen, watermark, pane_empty
  widgets/
    mod.rs         shared helpers: text, galley, hline / vline, tip, strip, focus rings
    button.rs      Btn builder, icon_button
    chart.rs       Chart, Series: line chart with legend and hover
    check.rs       checkbox, checkbox_sm
    chip.rs        chip, dot, paint_live_dot, status_pill, Status, ChipSize
    dialog.rs      Dialog, field_row, guard
    field.rs       qfield, qselect, search_field, text_input, text_input_invalid, password_input
    fit.rs         Fit levels, row and flow arithmetic for collapsing toolbars
    grid.rs        Grid, Column: virtualized grid
    inspector.rs   Inspector: resizable side pane; shared width
    menu.rs        menu items and popups in the family frame; dev_once / dev_open
    metrics.rs     metrics strip
    notice.rs      notice strip
    palette.rs     PaletteConfig, PaletteState, PaletteItem, palette: command palette and its ranking
    rich_grid.rs   RichGrid, RichCell: virtualized grid with painted cells
    sec.rs         sec_h, kv: inspector sections and rows
    seg.rs         segmented control
    switch.rs      switch toggle
    toast.rs       Toast, show: transient confirmations
assets/fonts/      TTF files and their license texts
tools/icons/       generator for the icon font and src/icons.rs
examples/gallery.rs  every widget section, in both themes (cargo run --example gallery)
examples/chrome.rs   the whole IDE frame with fake data (cargo run --example chrome)
```

Dependencies point one way. `theme` depends only on egui. `icons` depends on nothing. The widgets depend on
`theme`, `icons` and the helpers in `widgets/mod.rs`. Some widgets use each other where the design does: the
dialog uses `icon_button`, the connection chip and the text input; the menu uses the chip; `field`'s select
and the palette use the menu's popups or its debugging hook; `rich_grid` reuses `grid`'s `Column`, `HEADER_H`
and `ROW_H`. No module depends on an application.

`chrome` sits on top of `widgets`: its modules use the strip, text and tooltip helpers, chips, menus and the
search field, and nothing in `widgets` uses `chrome`. A chrome widget takes plain data and returns one
`Response` per clickable part, indexed like its input; the menus, the tooltips' wording and what a click means
stay in the application. Where Mb Studio and OPC Studio drew a part differently, the difference is a field or
a builder method, so each app can keep its look.

The crate depends on egui directly, and `lib.rs` re-exports that egui as `studio_ui::egui`. Applications use the
re-export, so one egui version is in use across the crate and the application.

## Theme tokens and egui

`theme::Theme` is a plain struct of `Color32` values and one `Shadow`. `DARK` and `LIGHT` are `const`
instances. Nothing is computed at runtime, so a theme lookup is free.

`theme::style(&Theme) -> egui::Style` builds an egui style from one theme. `theme::install` calls it for both
themes and selects dark:

```rust
ctx.set_fonts(fonts());
ctx.set_style_of(egui::Theme::Dark, style(&DARK));
ctx.set_style_of(egui::Theme::Light, style(&LIGHT));
ctx.set_theme(egui::Theme::Dark);
```

Because both styles are stored in the context, switching theme with `ctx.set_theme` needs no rebuild. The
mapping from tokens to egui visuals is:

| egui slot | Token or value |
|-----------|----------------|
| `panel_fill` | `editor` |
| `window_fill` | `raised` |
| `window_stroke` | 1 px `line` |
| `window_corner_radius` | `R_LG` (10) |
| `menu_corner_radius` | 8 |
| `window_shadow`, `popup_shadow` | `shadow` |
| `selection.bg_fill` / `stroke` | `sel` / `text` |
| `weak_text_color` | `text_3` |
| `hyperlink_color` | `accent_text` |
| `faint_bg_color` | `row_hover` |
| `extreme_bg_color`, `text_edit_bg_color` | `field` |
| `code_bg_color` | `panel` |
| `warn_fg_color`, `error_fg_color` | `warn`, `danger` |
| widget `noninteractive` | fill `panel`, stroke `line`, text `text`, radius `R_SM` |
| widget `inactive` | fill `field`, stroke `field_line`, radius `R` |
| widget `hovered` | fill `hover`, stroke `field_line` |
| widget `active` | fill `hover`, stroke `accent` |
| `disabled_alpha` | 0.5 (controls are dimmed, not greyed) |

Spacing follows the family's density: item spacing 8 by 4, button padding 10 by 4, minimum interact height
`ROW` (24), menu margin 4, window margin 12, icon width 14. The scroll bars are floating overlays: hidden while
idle, a 3 px handle when the pointer is over the area.

Text rendering has two adjustments. The glyph transfer function is a gamma of 1.15 (dark) or 1.2 (light), which
matches the weight of the design's browser rendering better than egui's default. The text cursor is 1 px and
does not blink.

The text scale is: `Small` 11, `Body` 13, `Button` 13, `Heading` 15 in Inter SemiBold, and `Monospace` 12.5.

## Font families

Six font families are registered by `theme::fonts()`:

| Family | Chain (first to last) |
|--------|-----------------------|
| `Proportional` | Inter-Regular, egui's default fonts, lucide |
| `Name("inter-medium")` | Inter-Medium, Inter-Regular, egui's default fonts, lucide |
| `Name("inter-semibold")` | Inter-SemiBold, Inter-Regular, egui's default fonts, lucide |
| `Monospace` | JetBrainsMono-Regular, Inter-Regular, egui's default fonts |
| `Name("mono-medium")` | JetBrainsMono-Medium, JetBrainsMono-Regular, Inter-Regular, egui's default fonts |
| `Name("icons")` | lucide only |

Named families are used through the helpers (`medium`, `semibold`, `mono_medium`, `icon_font`) so that call
sites never spell a family name. egui's default fonts stay behind the embedded ones as glyph fallbacks, so
characters that Inter lacks still render. The Lucide font is also appended to the proportional chains, so an
icon glyph inside ordinary text renders. Its `y_offset_factor` is 0, so icons are not shifted relative to
text.

## Icon pipeline

1. `tools/icons/icons.txt` lists the Lucide icon names the family uses: one name per line (its comment header is
   skipped), sorted, without aliases.
2. `tools/icons/generate.py` downloads the pinned lucide-static package, checks its SHA-512 integrity against
   the pin before anything is extracted, and reads each icon's codepoint from the package's
   `font/codepoints.json`. It fails on unknown names, duplicate names, names that share a glyph, and an
   unsorted list.
3. It writes the subset font `assets/fonts/lucide.ttf` (through fontTools), the license text
   `assets/fonts/Lucide-LICENSE.txt`, and `src/icons.rs`: one `pub const` per icon, each holding one codepoint
   of the Lucide font, plus `ALL`.
4. `theme::fonts()` embeds the subset, and `theme::icon_font` renders the glyphs.

The glyph list and the font subset must match, which is why both come from the same run of the script. Add an
icon by updating `icons.txt` and regenerating both files; see `tools/icons/README.md`. The Lucide license texts,
including the Feather-derived MIT notice, live in `assets/fonts/`.

## Widget conventions

**Hand-painted.** Each widget allocates its rectangle (`allocate_exact_size`, `allocate_rect`) and paints it
with the `Painter`. Stock egui button and checkbox visuals are global, so the crate does not use them. `Btn`
implements `egui::Widget`, so `ui.add(Btn::new(...))` reads like any other widget.

**Colors from the theme.** Each widget takes `theme::theme(ui)`, which picks `DARK` or `LIGHT` from
`ui.visuals().dark_mode`. A light popup inside a dark window therefore still draws in light colors.

**Text layout.** `galley` and `galley_truncated` lay out one line. egui caches the layout across frames, so a
widget that draws the same text every frame costs little. `css_text` and `lh` reproduce the design's line box
(`line-height: 1.45`) for paragraphs and tooltips.

**Crisp lines.** Rules are 1 px lines centred on a pixel (`hline`, `vline`, which call
`round_to_pixel_center`).

**Focus.** `track_focus_visible(ctx)` runs once per frame and records whether the last interaction was keyboard
navigation. `focus_ring` and `focus_ring_inset` draw the 2 px accent outline only while focus is
keyboard-visible, so a click does not leave a ring.

**Tooltips.** `tip` and `tip_at` place the family's tooltip on a chosen side, with the design's delay. Use them
instead of `on_hover_text`, which uses egui's own style.

**Virtualization.** `grid` and `rich_grid` lay out and paint only the rows inside the viewport, so 5,000 rows
cost about the same as 40. `chart` returns before doing any work when its rectangle is off-screen. `toast`
requests repaints only while it moves and at its expiry.

**Shared state in egui memory.** Small pieces of shared state live in `ctx.data` under `studio-ui-` ids: the
inspector width (`studio-ui-inspector-width`), an inspector edge drag in progress (`studio-ui-inspector-drag`),
the focus-visible flag (`studio-ui-focus-visible`) and the select popup's focus (`studio-ui-select-focus-chosen`).
They are temporary: they are not persisted between runs. An application that wants persistence saves the value
and sets it again at start. The palette and the toast use fixed egui ids of their own (`studio-ui-palette`,
`studio-ui-toast`), so this crate assumes one of each per egui context.

**Debugging hook.** `menu::dev_once` and `menu::dev_open` open a named menu or the palette when the
`STUDIO_UI_OPEN_MENU` environment variable names its key. The hook needs the `dev-hooks` feature and a debug
build; otherwise it compiles to `false`. It lets a developer or a screenshot script reach a popup without
clicking. The crate asks with the keys `palette` and `qselect:<label>`.

**Pure logic in unit tests.** Layout arithmetic (`fit::flow`, `metrics::grid_shape`, the inspector's width
rules), ranking and filtering (`palette::score` and the palette's filter), the chart's tick steps and number formatting
(`chart::nice_step`, `fmt_num`) and the toast's motion are plain functions with unit tests. Painting is checked
visually in the gallery, in both themes.

## Lints

`Cargo.toml` forbids `unsafe` code. It warns on `unwrap`, `expect`, `dbg!` and `todo!` (clippy), and on public
types without a `Debug` implementation (`missing_debug_implementations`). CI runs clippy with `-D warnings`.

## Planned work

Planned sibling crates, the known gaps of the API and the other planned work are in [ROADMAP.md](ROADMAP.md).
The sibling crates are not present and no API is promised. They will depend on this crate for tokens and
widgets; this crate will not depend on them.
