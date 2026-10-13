# Icon font pipeline

`assets/fonts/lucide.ttf` and `src/icons.rs` are generated, not hand-edited.
`generate.py` downloads the pinned [lucide-static](https://www.npmjs.com/package/lucide-static)
package, subsets its Lucide font to the icons in `icons.txt`, and rewrites:

- `assets/fonts/lucide.ttf`: the subset font, rendered by `theme::icon_font`.
- `assets/fonts/Lucide-LICENSE.txt`: the licence copied from the package.
- `src/icons.rs`: one `pub const NAME: &str` per icon (Lucide name upper-snake-cased,
  e.g. `trash-2` -> `TRASH_2`), plus `ALL: &[(&str, &str)]` for galleries.

## Requirements

Python 3.12+ and fontTools:

```sh
python3 -m venv /path/outside/repo/iconvenv
/path/outside/repo/iconvenv/bin/pip install fonttools
```

## Adding an icon

1. Append the lucide name to `icons.txt`, keeping the list sorted. Use the
   current Lucide name (for example `funnel`, not `filter`).
2. Run the script from anywhere:

   ```sh
   /path/outside/repo/iconvenv/bin/python tools/icons/generate.py
   ```

3. Use the constant, e.g. `icons::TRASH_2`, rendered with `theme::icon_font(size)`.

The script fails loudly when a name does not exist in the pinned version, when
two names point at the same glyph (aliases such as `sort-asc`), when the list
has duplicates, or when it is not sorted.

## Pinned version

Pinned in `generate.py` as `LUCIDE_STATIC_VERSION` together with
`LUCIDE_STATIC_INTEGRITY` (the npm `dist.integrity` SHA-512). The tarball is
verified against the integrity string before anything is extracted.

To bump Lucide, change both constants (take the new integrity from
`https://registry.npmjs.org/lucide-static/<version>`, field `dist.integrity`),
re-run the script, and check the diff of `src/icons.rs`: codepoints of existing
icons should not move. If one does, the change is visible in review.

## Licence

Lucide is licensed under ISC. Icons derived from Feather are under the MIT
licence, and the copied `Lucide-LICENSE.txt` (copied from the package on every
run) contains both notices. Keep that file next to the font when redistributing.
