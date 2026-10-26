# Requirements

- R1: Given the 2026 calendar in TOML, output the standings table sorted by points, ties broken alphabetically.
- R2: A driver can win the title if `points + max_points_left >= leader_points`.
- R3: Malformed data (unknown driver, duplicate entry, oversized result, unparseable TOML) → exit code 1 with a diagnostic on stderr; never silent wrong output.
- R4: Deterministic output — same input file, byte-identical output.
