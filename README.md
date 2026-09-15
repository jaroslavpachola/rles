# rles

[![CI](https://github.com/jaroslavpachola/rles/actions/workflows/ci.yml/badge.svg)](https://github.com/jaroslavpachola/rles/actions/workflows/ci.yml)

A small terminal pager written in Rust — a `less` replacement.

```sh
rles README.md
rles *.log                 # switch files with :n and :p
git log --color | rles     # ANSI colors render as-is
```

When stdout is not a terminal, `rles` behaves like `cat`, so it is safe in pipelines.

## Features

- less-compatible movement, search, marks, and multi-file keys (`h` shows the full reference)
- regex search with smart case and highlighting of all on-screen matches
- ANSI color passthrough — colored input renders correctly, other escape sequences are filtered out
- follow mode (`F`) like `tail -f`
- line numbers, horizontal scrolling, numeric count prefixes

## Install

```sh
cargo install rles
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
| `/pattern` | search forward (regex, smart case) |
| `?pattern` | search backward (regex, smart case) |
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
| `h`, `H` | help screen |

Most movement keys accept a numeric count prefix, e.g. `12j` scrolls twelve lines.

## Options

```
-N, --line-numbers    show line numbers
```

## Notes

- Long lines are chopped at the screen edge; use `←`/`→` to scroll horizontally.
- Searches are case-insensitive unless the pattern contains an uppercase letter.
- Unix terminals are the supported target.

## License

MIT
