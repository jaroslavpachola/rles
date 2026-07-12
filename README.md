# rles

[![CI](https://github.com/jaroslavpachola/rles/actions/workflows/ci.yml/badge.svg)](https://github.com/jaroslavpachola/rles/actions/workflows/ci.yml)

A small terminal pager written in Rust — a `less` replacement.

```sh
rles README.md
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

## Notes

- Long lines are chopped at the screen edge (horizontal scrolling is planned).
- Unix terminals are the supported target.

## License

MIT
