# Publishing checklist

This repository starts private. Making it public exposes its entire history, not only the files on the default
branch: every commit, every tag, every author and committer name and email, and every file ever committed.
Work through this list before changing the visibility, and again before any `crates.io` release.

## 1. Decide what the crate is

- `Cargo.toml` has `publish = false`. Keep it until you have decided to publish to crates.io.
- If you publish: check that the name `studio-ui` is free on crates.io, check that the README and the
  `include` list in `Cargo.toml` give the right package contents (`cargo package --list`), run
  `cargo publish --dry-run`, and only then remove `publish = false`.
- Making the repository public does not publish the crate, and publishing does not make the repository public.
  They are separate decisions.

## 2. Scan the full history for secrets

The working tree is not enough. Scan every commit, every branch and every tag:

```sh
gitleaks git --redact --log-opts="--all" .
gitleaks dir --redact .
```

Both must report no findings. For each finding, follow the procedure in
[If something sensitive was committed](#if-something-sensitive-was-committed) below, even when the finding
looks like a test value. A test value is still a value someone may reuse.

## 3. Scan the files and read the history yourself

```sh
bash scripts/check-public.sh
```

The script covers tracked and untracked files. It checks them for absolute home-directory paths (macOS, Linux
and Windows), PEM private key headers, well-known API token shapes, tracked `.env` files, email addresses outside
a short allow-list (GitHub no-reply addresses, the Anthropic no-reply address and documentation placeholders),
internal host names (`*.internal`, `*.lan`, `*.local`), private IPv4 addresses (except in the files listed in
`scripts/check-public.allow`), and binary files over 2 MB. It does not read commit messages, it does not know the
names of private applications or products, and it does not know which values are yours. Also check, by hand:

- Commit authors and committers. Commit metadata is public with the history:

  ```sh
  git log --all --format='%an <%ae>%n%cn <%ce>' | sort -u
  ```

  Every address shown there becomes public. Use GitHub's no-reply address for commits
  (`<id>+<user>@users.noreply.github.com`, shown in the account's email settings). If earlier commits used a
  personal address, rewrite them with `git filter-repo --mailmap` (see below) before going public.
- Commit messages, tags and branch names, for personal details or internal references.
- Files outside the source tree that were committed by mistake: screenshots, exported data, notes, logs.
- Internal references in comments and documentation: private repository names, internal document paths,
  product names that are not public, and links to internal services. The script does not search for product or
  application names. Search for each name you know, across the tree and the commit messages (`git log --all
  --grep`), with `grep -rniI`, and rewrite the text that matches.

## 4. Check the content of the public files

- `LICENSE` is present and is the license you intend (MIT).
- `THIRD_PARTY_NOTICES.md` names every font and icon file in `assets/fonts/`, and each license text is there.
- The README's install commands work from a clean clone. The tag it pins (`v0.1.0`) exists and points at the
  release commit.
- `cargo run --example gallery` works. Its sample data is made up (`.example` hosts and invented numbers), so it
  shows nothing private. The repository holds no screenshots, and any added later must show no private data.
- `CHANGELOG.md` has no entry that a public reader would not understand.
- No link in the repository points to a private host or a private repository.

## 5. Prepare the GitHub settings

- Enable private vulnerability reporting (Settings, then Code security). `SECURITY.md` depends on it.
- Protect the default branch: require the CI checks to pass before merging.
- Actions: the workflow uses only the default `GITHUB_TOKEN`. Organization accounts also need a
  `GITLEAKS_LICENSE` secret for the gitleaks action. A personal account does not.
- Restore the workflow's `push:` and `pull_request:` triggers (it is `workflow_dispatch` only while the
  repository is private, to save Actions minutes), then check that it runs green.

## 6. Change the visibility

Settings, then General, then Danger zone, then Change repository visibility. After the change:

- Open the public repository in a private browser window and check the README, the license and the CI badge.
- Check that a fork or a clone from GitHub has the same content as your local tree.

## If something sensitive was committed

Treat any credential that reached a commit as compromised, even if it was removed in the next commit. Git keeps
it in history.

1. **Rotate or revoke the credential first.** Do this at the service that issued it. Rewriting history does not
   help a credential that someone already copied.
2. **Find every occurrence.** Run the scans in step 2 and list each commit, path and value.
3. **Rewrite the history before the repository goes public.** Use
   [`git filter-repo`](https://github.com/newren/git-filter-repo). Keep the rewrite files outside the
   repository, for example in your scratch directory, so they are not committed.

   Remove a file from every commit:

   ```sh
   git filter-repo --invert-paths --path path/to/file
   ```

   Replace a string in every commit. The expressions file holds lines such as `literal:VALUE==>REDACTED`:

   ```sh
   git filter-repo --replace-text /path/outside/repo/expressions.txt
   ```

   Replace author identities. The mailmap file holds lines such as `New Name <new@example> <old@example>`:

   ```sh
   git filter-repo --mailmap /path/outside/repo/mailmap.txt
   ```

   `git filter-repo` removes the `origin` remote as a safety measure. Add it again before you push.
4. **Force-push the rewritten history**, with all branches and tags:

   ```sh
   git push --force --all origin
   git push --force --tags origin
   ```

   Every commit SHA and tag changes. Anyone with a clone must re-clone or rebase onto the new history.
5. **Ask GitHub to purge cached objects.** If the repository, or a fork of it, was ever public, the old objects
   can stay reachable through cached views and unreachable commits until GitHub removes them. Contact GitHub
   Support and ask them to remove the cached views and unreachable objects for the repository.
6. **Scan again.** Run gitleaks and `scripts/check-public.sh` on the rewritten history. Only then make the
   repository public.
7. **Prevent a repeat.** Add the file pattern to `.gitignore`, and keep `.githooks/pre-commit` enabled.
