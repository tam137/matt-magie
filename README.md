# Matt-Magie v1.3 - Chess Engine Matchup Manager

Matt-Magie is a lightweight, high-performance matchup manager written in Rust. It facilitates games between chess engines using the standard Universal Chess Interface (UCI) protocol, records games in PGN format, and displays beautiful, unwrapped, sequentially calculated Elo scoreboards.

---

## 🛠️ Main Interface: The CLI Wrapper

The easiest and most interactive way to use Matt-Magie is through the `mm.sh` bash script in the root directory. 

To run the interactive CLI:
```bash
./mm.sh
```

### Features of the Interactive CLI:
1. **Single Match (1 vs 1)**: Configure a head-to-head match between two engine versions, swap colors automatically, and set time controls.
2. **Tournament Mode (Round-Robin)**: 
   - Select a subset of participating engines.
   - Set a mandatory, validated PGN output filename.
   - Plays a complete all-vs-all double round-robin tournament.
3. **Import Engines**: Dynamically import/update compiled chess engine versions (like `suprah`) directly from your workspace into the `engines/` directory.
4. **View PGN Statistics**: Instantly parse any tournament or match PGN to view sequential Elo ratings calibrated at 1500 ($K=32$) in a beautifully formatted, unwrapped scoreboard.

---

## ⚙️ Direct Binary Execution

If you prefer to run the matchup manager programmatically or from other scripts, you can build and invoke the compiled native binary directly:

### 1. Compile the Release Binary
```bash
cargo build --release
```

### 2. Run a Match
The compiled binary (`./target/release/Matt-Magie`) expects exactly 11 arguments:

```bash
./target/release/Matt-Magie \
  "<engine_1_path>" \
  "<engine_2_path>" \
  "<logfile_path>" \
  "<pgn_path>" \
  "<event_name>" \
  "<site>" \
  "<round_number>" \
  "<time_per_game_ms>" \
  "<increment_per_move_ms>" \
  "<logging_flag>" \
  "<debugging_flag>"
```

### Argument Details:
* **`engine_1_path` & `engine_2_path`**: Absolute or relative paths to your executable chess engines.
* **`logfile_path`**: Path where the detailed communication logs will be appended.
* **`pgn_path`**: File path where the resulting match will be appended.
* **`event_name` & `site` & `round_number`**: Metadata written directly into the PGN tags.
* **`time_per_game_ms`**: Base thinking time per game in milliseconds (e.g., `30000` for 30 seconds).
* **`increment_per_move_ms`**: Time increment added to the clock per move in milliseconds (e.g., `1000` for 1 second).
* **`logging_flag`**: Use `log_on` to write engine-to-manager UCI logs.
* **`debugging_flag`**: Use `debug_on` to pass UCI debug commands to engines.

---

## 📊 Beautiful Scoreboards & ELO Evaluation

All game outcomes are parsed and analyzed sequentially using `summary.sh` (which delegates to `summary.py`):

```bash
./summary.sh games.pgn
```

This generates a gorgeous, console-optimized scoreboard anchored at 1500:
```
======================================================================
                TOURNAMENT SCOREBOARD & ELO EVALUATION
======================================================================
Rank Engine Name               Games W/D/L     Points Score% Elo   Elo+/-
----------------------------------------------------------------------
1    Rust-In-Pieces V0.2.2     4     3/1/0     3.5    87.5   1544  +44
2    Rust-In-Pieces V0.2.6     4     1/1/2     1.5    37.5   1486  -14
3    Rust-In-Pieces V0.3.0     4     0/2/2     1.0    25.0   1471  -29
======================================================================
Note: Elo starts at 1500 and updates sequentially per game.
======================================================================
```
