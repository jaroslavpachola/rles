# Roadmap

Shipped releases are documented by their git tags (v0.1.0–v0.5.0). Planned:

## v0.6.0 — line wrapping

- wrap long lines by default, matching less
- `-S` / `--chop-long-lines` flag and runtime `-S` toggle to switch back to chopping
- keep horizontal scrolling working in chop mode

## v0.7.0 — filtering and search polish

- `&pattern` filter mode: show only matching lines (empty pattern clears)
- `ESC-u` to clear search highlighting without losing the pattern
- search history on the `/` and `?` prompts (up/down arrows)

## v0.8.0 — large files

- lazy loading / memory-mapped input so multi-GB logs open instantly
- incremental line indexing in the background
- follow mode reads appended bytes instead of re-reading the whole file

## v0.9.0 — daily-driver polish

- read default options from the `LESS` environment variable
- `v` opens the current file at the current line in `$EDITOR`
- exit codes and `-F`/`-X`-style behavior aligned with less
- man page generated from the help text

## v1.0.0 — distribution

- publish to crates.io (`cargo install rles`)
- release checklist (changelog, version bump, tag)
- wider CI targets (musl, ARM Linux)
