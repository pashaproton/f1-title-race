# f1-title-race

[![CI](https://github.com/pashaproton/f1-title-race/actions/workflows/ci.yaml/badge.svg?branch=main&job=test)](https://github.com/pashaproton/f1-title-race/actions/workflows/ci.yaml)
[![Mutation tests](https://github.com/pashaproton/f1-title-race/actions/workflows/ci.yaml/badge.svg?branch=main&job=mutants)](https://github.com/pashaproton/f1-title-race/actions/workflows/ci.yaml)

A command-line F1 championship standings tracker.

Given a season definition (calendar, drivers, scoring tables, race and sprint results) in a TOML
file, it prints the drivers' championship standings, the maximum number of points any driver can
still earn this season, and — after each race weekend — which drivers are still mathematically
in contention for the title.

## Sample output

```
Maximum points a driver can earn in the season: 623
Maximum points left in the season: 125
1:	••••••••••••••••••••••••••••••••••••••••••••••••••••••••••                                          Kimi Antonelli	(365),	Still possible to win: yes
2:	••••••••••••••••••••••••••••••••••••••                                                              George Russell	(242),	Still possible to win: yes
3:	••••••••••••••••••••••••••••••••••••••                                                              Lewis Hamilton	(241),	Still possible to win: yes
4:	•••••••••••••••••••••••••••••••••••••                                                               Max Verstappen	(236),	Still possible to win: no
```

A driver can still win if `points + max_points_left >= leader_points` (ties broken alphabetically).

## Usage

```sh
cargo run -- path/to/season.toml
```

The season file defines the scoring tables, the driver roster, and the calendar:

```toml
race_points = [25, 18, 15, 12, 10, 8, 6, 4, 2, 1]
sprint_points = [8, 7, 6, 5, 4, 3, 2, 1]
drivers = ["Kimi Antonelli", "George Russell", "Lewis Hamilton"]

[[races]]
name = "Monza"
sprint = false
race_result = ["Kimi Antonelli", "George Russell", "Lewis Hamilton"]

[[races]]
name = "China"
sprint = true
sprint_result = ["George Russell", "Charles Leclerc", "Lewis Hamilton"]
```

Races without a `race_result` haven't happened yet; `max_points_left` accounts for every
scheduled-but-unraced event. Malformed data (unknown driver, unparseable TOML, empty season)
fails loudly with exit code 1 — standings are never silently wrong.

## Design notes

The project went through a deliberate refactor pipeline from a first working version to its
current shape; see `DESIGN.md` for the reasoning behind the main decisions:

- **Newtypes as invariants** — `Driver` and `Points` wrap primitives so that `Points * Points`
  fails to compile; only operations meaningful in the domain are implemented.
- **One source of truth per fact** — the calendar is a single list of `Race` structs; a result
  is always attached to its weekend, and a `sprint` flag disambiguates "no sprint" from
  "sprint not yet run".
- **Loud errors** — a `ValidationError` enum with `Display` messages instead of asserts; the
  parse/validate/score pipeline returns `Result` end to end.
- **Thin main** — `main` wires a path and stdout into a testable `run(path, out)` function;
  the output contract is pinned by a golden-file integration test.

## Testing

- Unit tests use inline TOML fixtures — no dependence on files or working directory.
- An integration test runs the real binary end to end against a frozen golden fixture
  (`tests/cli.rs` + `tests/golden.toml`).
- `cargo clippy --all-targets -- -D warnings` passes with zero warnings.
- Mutation-tested with [cargo-mutants](https://crates.io/crates/cargo-mutants):
  **0 missed mutants** (35 caught, 28 unviable at last run) — every mutation of the logic
  is caught by the test suite.

## Data note

The 2026 season data included in `data/` is hypothetical and used as example input.

## License

MIT — see [LICENSE](LICENSE).
