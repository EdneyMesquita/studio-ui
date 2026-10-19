# Known gaps and planned work

This page lists what the crate does not do yet, and why. It has no dates, and the order is not a commitment.

## Known gaps

API friction found while writing the gallery:

- `Chart` has no builders for `area` and `legend`. The fields exist (`area` defaults to off, `legend` to on), but
  callers cannot set them, so the gallery cannot show an area fill or hide the legend.
- `Grid` and `RichGrid` expose no row context-menu hook. A right-click is reported only as `secondary_clicked`,
  which gives a row index, so a row's menu has to be anchored by hand.
- Widget wording is hard-coded in places: the inspector's resize tooltip, the "Copy" and "Close" tooltips, and
  the "Esc" keycap. They should become parameters, or a small `Strings` table, so that applications can localise them.
- `PaletteItem::new` takes `&'static str` for the group, icon and shortcut. Items built from runtime data, such
  as a user's own shortcuts, must leak or intern their strings; owned strings would remove that.
- Fixed egui ids (`studio-ui-palette`, `studio-ui-toast`, `studio-ui-focus-visible`) assume one palette, one toast
  and one window per egui context. Callers should be able to pass an id. The inspector width is shared on purpose.
- `Metric` has a public `link` field but no builder method, unlike `icon` and `color`, so it is set by hand.
- `qselect` returns `bool` (whether the selection changed) where the other field helpers return the `Response`.
  A caller therefore cannot attach a tooltip to it or check its focus.
- `RichGrid::row_bg` takes a borrowed `&dyn Fn`, whose lifetime ties it to the grid builder. The call site has to
  keep the closure in a local variable first.
- The footer of `Dialog` cannot read what the body produced. A caller that needs the body's state in the footer,
  such as whether a typed guard matches, shares it through a `Cell`, as the gallery does.

## Planned work

- Sibling crate `studio-shell`: application chrome shared by the desktop apps, such as the toolbar, the tool
  stripe, the explorer tree, tabs, the status bar, and the panel head with its search and empty states. Each
  application should not carry its own copy.
- Sibling crate `studio-platform`: operating-system and remote services, such as secret storage in the OS
  keychain, SSH tunnels, and logging that never records secrets. They stay out of the UI crate because they
  depend on the system.
- Possibly convert this repository into a `studio-kit` workspace that holds the crates together, so that they
  share one release cycle and one egui version. Not decided.
- Generic widgets for the planned family views: a tree row and a hex dump with highlighted field spans (timeline
  lanes are in, as `widgets::timeline`). Each is generic user-interface code, so it belongs here rather than in an application.
- Wizard steps for dialogs with several sections, so that a multi-step task does not need a bespoke dialog.
- A `tag` pill, a small labelled chip, for labelled tags in lists and forms. Not in the crate yet.
- Snapshot tests with egui_kittest in CI (Linux, software rendering). Painting is checked by eye today, in the
  gallery; snapshots would catch a change in either theme without a person looking at it.
- A single source of design tokens that can also generate CSS for the HTML mockups. The tokens now live in
  `theme.rs` and in the mockups, and nothing keeps the two in step.
