# Matt-Magie v1.6.0 - Chess Engine Matchup Manager

Matt-Magie is a lightweight, high-performance matchup manager written in Rust. It coordinates games between chess engines using the standard Universal Chess Interface (UCI) protocol, records matches in PGN format, and displays beautiful, console-optimized, sequentially calculated Elo scoreboards.

---

## 🛠️ Main Interface: The CLI Wrapper (`mm.sh`)

The most interactive and convenient way to use Matt-Magie is through the `mm.sh` bash script located in the root directory. 

### 1. Interactive CLI Mode
To launch the interactive, guided terminal menu:
```bash
./mm.sh
```

#### Features of the Interactive CLI:
* **Single Match (1 vs 1)**: Configure a head-to-head match between two engine versions, set custom time controls with increment, specify the number of rounds, and let the manager automatically swap colors every game.
* **Tournament Modes**: Both modes support parallel execution (concurrency) to leverage multi-core processors.
  * **Round-Robin (All-vs-All)**: Select a subset or all participating engines from your local pool and execute a complete double round-robin all-vs-all tournament.
  * **Gauntlet (Challenger vs Rest)**: Put the first selected engine to the test against all other participating engines (playing both White and Black against each). Generates an exclusive, beautiful head-to-head scoreboard summary of the challenger's performance.
* **Import/Update Engines**: Dynamically import/update compiled chess engine versions (like `suprah`) directly from your local workspace into the `engines/` directory with correct tags.
* **View PGN Statistics**: Instantly parse any local PGN file to view sequential Elo ratings calibrated at 1500 ($K=32$) in a beautifully formatted, unwrapped scoreboard.

### 2. Non-Interactive Tournament Mode (`-t`)
You can run tournaments fully non-interactively using a `.trn` configuration file. This is perfect for background runs, remote server executions, or headless environments.

To execute a tournament file:
```bash
./mm.sh -t path/to/tournament.trn
```

#### `.trn` File Format Example:
```ini
# Lines starting with '#' are treated as comments and ignored
engines = suprah-0.7.8, suprah-0.7.9, mewel_V0.3.5.sh

# Time control per game in milliseconds (e.g., 30000 ms = 30s)
time_control = 30000

# Increment per move in milliseconds (e.g., 1000 ms = 1s)
increment = 1000

# Number of rounds (each engine pair plays both White and Black per round)
rounds = 2

# PGN output filename (will automatically append .pgn if missing)
pgn = my_tournament.pgn

# Tournament mode: 'round_robin' (All-vs-All) or 'gauntlet' (Engine 1 vs All) (optional, default: round_robin)
mode = gauntlet

# Number of parallel games (concurrency) (optional, default: 1)
concurrency = 2

# Opening lines file (optional). One space separated list of UCI moves per line.
# Both games of a colour-swapped pair are played from the same line.
openings = openings/book_8ply.txt

# Engine configuration options sent via setoption name <Key> value <Value>
# comma-separated key-value pairs (optional)
engine_options = Hash=128, Threads=1

# Send `debug on` to every engine (optional, default: false). Engines may write large logs.
engine_debug = false
```

#### Parameter Details:
* **`engines`**: Comma-separated list of engine filenames. These binaries **must** be stored inside the `engines/` directory and be executable.
* **`time_control`**: Base time per engine in milliseconds.
* **`increment`**: Time increment in milliseconds added after each move.
* **`rounds`**: Number of rounds (each engine plays every other engine twice per round—once as White, once as Black—or in Gauntlet mode, the challenger plays each opponent twice per round).
* **`pgn`**: Target PGN output filename. If the file already exists, new games will be appended.
* **`engine_options`**: (Optional) Comma-separated engine settings sent immediately after handshake (e.g. `Hash=128, Threads=1`).
* **`mode`**: (Optional) Tournament format. Choose `round_robin` (default) for all-vs-all, or `gauntlet` for a challenger setup where the first engine listed in `engines` plays against all other engines (once as White and once as Black per opponent, per round).
* **`openings`**: (Optional) Path to a file of opening lines, one space separated list of UCI moves per row. Lines may have an odd number of moves, in which case Black searches first. Every opening move is checked for legality before the engines are started; an illegal line aborts that game without a PGN entry. Each colour-swapped game pair is played from the same line, and the file is cycled through when there are more pairs than lines. Without it every game starts from the initial position, which makes the games of a match highly correlated and costs a large amount of statistical resolution.
* **`engine_debug`**: (Optional) `true` sends `debug on` to every engine. Default `false`; the environment variable `MM_ENGINE_DEBUG=1` switches it on when the file does not set it, and also applies to the interactive single match.
* **`concurrency`**: (Optional) Number of games to run in parallel. Default is `1`. Running multiple games simultaneously speeds up tournaments significantly. Note that each running game spawns two engine processes, so configure this based on your system's CPU cores and memory (e.g., total threads used = `2 * concurrency * Threads`).

---

## 🔌 UCI Protocol Support & Engine Compatibility

Matt-Magie acts as a matchup coordinator and is compatible with **any chess engine** that complies with the standard **Universal Chess Interface (UCI)** protocol. 

The matchup manager orchestrates games by executing the following standard UCI commands:
1. **Handshake**: Sends `uci` and expects the engine to respond with `uciok`.
2. **Options Configuration**: Sends `setoption name <Name> value <Value>` for each custom engine option right after receiving `uciok` (e.g. configuring `Hash` or `Threads`).
3. **Readiness Check**: Sends `isready` and expects the engine to respond with `readyok`.
4. **New Game Setup**: Sends `ucinewgame` before every new game.
5. **Position Transmission**: Sends `position startpos moves <move_list>` after each played move to synchronize the internal board state with the engine. When an opening line is configured, it is transmitted the same way before the first search.
6. **Options for both sides**: `setoption` values from `engine_options` are sent to both engines. Earlier versions passed them to the White side only.
7. **Search Command**: Sends time-controlled search instructions:
   `go wtime <white_time> btime <black_time> winc <white_increment> binc <black_increment>`
   The increment is included from the very first search on. It then parses the engine's output to read `bestmove <move>` and plays it on the internal manager board.
   Every `bestmove` is checked against the legal moves of the side to move. An illegal or unreadable move, or a move sent out of turn, loses the game (`Termination` `WhiteWinByIllegalMove` / `BlackWinByIllegalMove`); reason, FEN and legal moves are written to the log.
8. **Interruption & Clean Termination**: Sends `stop` to halt any active search when a game is over or times out, followed by `quit` to cleanly terminate the engine processes.

> [!WARNING]
> **Host Architecture Compatibility**: Since Matt-Magie spawns chess engines as native subprocesses, all binaries in the `engines/` directory must be compiled for and compatible with the target host architecture (e.g., `x86_64` or `aarch64/ARM`) where the manager is running.

---

## 🚀 Remote Server Deployment & Headless Execution (Optional)

For running long-running matches or massive tournaments on a remote server, Matt-Magie includes a deployment workflow.

### 1. Deploying to a Server
You can deploy your local project codebase and automatically compile the matchup manager natively on a remote server using the environment variable `REMOTE_SERVER_IP` and the provided `./deploy.sh` script:

```bash
export REMOTE_SERVER_IP="<your_server_ip>"
./deploy.sh
```
*This script uploads the source code and natively compiles the Rust binary on the server to prevent any host architecture compilation issues (e.g., Exec format errors).*

### 2. Headless/Background Execution (No-Hang-Up)
To run a tournament in the background and log out of your SSH session safely without terminating the games:

```bash
nohup ./mm.sh -t tournament.trn > tournament.log 2>&1 &
```
All console progress and final scoreboards will be redirected and saved inside `tournament.log`. You can monitor it live with:
```bash
tail -f tournament.log
```

---

## ⚙️ Direct Binary Execution

If you prefer to run the matchup coordinator programmatically or via custom automation, you can invoke the compiled native binary directly:

### 1. Compile the Release Binary
```bash
cargo build --release
```

### 2. Run a Match
The compiled binary (`./target/release/Matt-Magie`) expects 11 standard arguments, followed by up to three optional ones:

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
  "<debugging_flag>" \
  "[engine_1_options]" \
  "[engine_2_options]" \
  "[opening_moves]"
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
* **`engine_1_options`**: (Optional) Comma-separated engine settings sent via UCI `setoption` immediately after handshake (e.g., `"Hash=128,Threads=1"`).
* **`engine_2_options`**: (Optional) Settings for the second engine. Empty or missing means the same as `engine_1_options`.
* **`opening_moves`**: (Optional) Space separated UCI moves played before the engines take over (e.g., `"e2e4 c7c5 g1f3"`).

---

## 📊 Scoreboards & ELO Evaluation

All game outcomes are parsed and analyzed sequentially using `summary.sh` (which delegates to `summary.py`). By default, it prints only the scoreboard and ELO evaluation. Use the optional `-g` or `--games` switch to display the list of individual game results:

```bash
# Print scoreboard only (default)
./summary.sh games.pgn

# Print scoreboard AND individual game results
./summary.sh games.pgn -g

# Display command line help
./summary.sh -h
```

This generates a gorgeous, console-optimized scoreboard anchored at 1500:
```
======================================================================
                TOURNAMENT SCOREBOARD & ELO EVALUATION
======================================================================
Rank Engine Name               Games W/D/L     Points Score% Elo   Elo+/-
----------------------------------------------------------------------
1    Rust-In-Pieces V0.7.9     2     2/0/0     2.0    100.0  1531  +31
2    Rust-In-Pieces V0.7.8     2     0/0/2     0.0    0.0    1469  -31
======================================================================
Note: Elo starts at 1500 and updates sequentially per game.
======================================================================
```
