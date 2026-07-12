# rles

[![CI](https://github.com/jaroslavpachola/rles/actions/workflows/ci.yml/badge.svg)](https://github.com/jaroslavpachola/rles/actions/workflows/ci.yml)

A small terminal pager written in Rust — a `less` replacement.

```sh
rles README.md
rles *.log        # switch files with :n and :p
git log | rles
```

When stdout is not a terminal, `rles` behaves like `cat`, so it is safe in pipelines.

## Install

```sh
cargo install --path .
```

Prebuilt binaries are attached to [GitHub releases](https://github.com/jaroslavpachola/rles/releases).

## Keys

| Key | Action |
| --- | --- |
| `q`, `Q`, `ctrl-c` | quit |
| `j`, `↓`, `Enter`, `ctrl-e` | one line down |
| `k`, `↑`, `ctrl-y` | one line up |
| `Space`, `f`, `PgDn`, `ctrl-f` | page down |
| `b`, `PgUp`, `ctrl-b` | page up |
| `d`, `ctrl-d` | half page down |
| `u`, `ctrl-u` | half page up |
| `g`, `<`, `Home` | go to top |
| `G`, `>`, `End` | go to bottom |
| `/pattern` | search forward (regex) |
| `?pattern` | search backward (regex) |
| `n` / `N` | repeat search / in opposite direction |
| `←` / `→` | scroll horizontally |
| `<n>g`, `<n>G` | go to line n |
| `<n>p`, `<n>%` | go to n percent of the file |
| `-N` | toggle line numbers at runtime |
| `:n` / `:p` | next / previous file |
| `=`, `ctrl-g` | file name, position, and file count |
| `F` | follow the file like `tail -f` (any key stops) |
| `m<letter>` | set a mark at the current position |
| `'<letter>` | jump to a mark (`''` returns to the previous position) |

Most movement keys accept a numeric count prefix, e.g. `12j` scrolls twelve lines.

## Options

```
-N, --line-numbers    show line numbers
```

## Notes

- Long lines are chopped at the screen edge; use `←`/`→` to scroll horizontally.
- Unix terminals are the supported target.

## License

MIT
