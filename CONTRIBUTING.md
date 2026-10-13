# Contributing to studio-ui

Thank you for helping. This crate is shared by a family of desktop applications, so changes are judged by
what every application gets from them, not only by the application that needed them.

## What belongs in this crate

A change belongs here when:

- At least two applications use it, or will use it, and the code would otherwise be copied between them.
- It is user-interface code: a token, a font, an icon, a widget or a layout helper.

It does not belong here when it carries application or domain logic: protocols, devices, storage, network
clients, credentials, or the screens of one application. Those belong in the application or in a sibling crate.

Names describe the user interface, not the domain. Do not put domain words (such as register, queue, table,
device or the name of an application) into public names, into documentation, or into default text. If a
widget needs domain text, the caller passes it in.

## Colors and themes

- Widgets read colors only from `studio_ui::Theme`, through `theme::theme(ui)` or `Theme::of`. Do not write hex
  values in a widget. The connection palette in `theme::identity` is the single exception.
- Every new token is added to both `DARK` and `LIGHT`, with a value chosen for that theme.
- Check every visual change in both themes before you open a pull request. The dark theme is the default, and
  the light theme must not be an afterthought.

## Dependencies and egui

- The crate targets one egui version at a time (see `Cargo.toml`). A change of egui version is made here first,
  then in every application, in the same release cycle. Do not let applications depend on different egui
  versions.
- Applications use egui through `studio_ui::egui`, the re-export of the egui this crate is built against. This
  crate depends on egui directly. Do not add a second egui dependency to it.
- Keep the runtime dependency list short. Development-only dependencies stay under `[dev-dependencies]`.
- New fonts and icons need their license text in `assets/fonts/`, and an entry in `THIRD_PARTY_NOTICES.md`.

## Icons

Icons are generated. To add or change one, edit the list in `tools/icons/icons.txt` and run
`tools/icons/generate.py` as described in [tools/icons/README.md](tools/icons/README.md). Never edit
`src/icons.rs` or `assets/fonts/lucide.ttf` by hand: the next run would overwrite the change. A changed
codepoint for an existing icon is visible in the diff of `src/icons.rs`, and should be explained in the pull
request.

## Development hooks

A hook that exists only for screenshots or end-to-end scripts goes behind the `dev-hooks` feature and
`debug_assertions`, as `menu::dev_once` does, so that release builds carry none of it. Document the keys it
answers in the module's documentation.

## Versioning

The crate is pre-1.0. A minor version (`0.x` to `0.y`) may contain breaking changes. Patch versions contain
fixes only. Every breaking change is listed under a "Changed" or "Removed" heading in `CHANGELOG.md`, and its
commit message carries a `!` after the type, or a `BREAKING CHANGE:` footer.

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): short summary in the imperative

Optional body explaining why, wrapped at about 72 columns.
```

Types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`. Scopes are module names
where it helps: `theme`, `icons`, `grid`, `palette`, `menu`, `toast`, and so on. Examples:

```
feat(grid): add a sticky first column
fix(menu): highlight the row under the pointer
```

## Before you open a pull request

Run each of these and fix what they report:

```sh
cargo fmt --all
bash scripts/ci-local.sh          # what CI checks: scan, fmt, clippy (both feature sets), tests, examples, gitleaks
bash scripts/ci-local.sh --quick  # without the example build and the history scan
```

Add a unit test for any pure logic you change (layout arithmetic, filtering, scoring). Widget painting is
checked by eye in the gallery example, in both themes: `cargo run --example gallery`.

Update `CHANGELOG.md` under "Unreleased" for any user-visible change.

## Git hooks

Enable the repository's hooks once per clone:

```sh
git config core.hooksPath .githooks
```

`pre-commit` runs `scripts/check-public.sh` and `cargo fmt --check`. `pre-push` runs the full
`scripts/ci-local.sh`. While the repository is private the GitHub workflow runs only on demand
(`workflow_dispatch`), so the pre-push hook is the gate; install gitleaks so it can scan
(`brew install gitleaks`).

## Keeping the repository public-ready

Nothing that cannot be public may be committed: no personal absolute paths, no email addresses other than
GitHub no-reply addresses, no internal host names, no tokens or keys, no screenshots of private data, and no
references to private repositories or to the names of private applications. `scripts/check-public.sh` checks
most of this; it does not check names, and it does not replace reading your own diff. See
[docs/PUBLISHING.md](docs/PUBLISHING.md).

## Licensing of contributions

By contributing, you agree that your contribution is licensed under the MIT License of this repository.
