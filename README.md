The goal of Matt-Magie is to implement the UCI protocol for facilitating games between chess engines in a lightweight and modular way.

For example, you can start a game by making a call:


```
# choose your engine, engine_1 will be white the other black
engine_1=/path/to/engines/stockfish
engine_2=/path/to/engines/rust_in_pieces

# choose path for logfile and pgn
logfile=/path/to/engine-manager.log
pgn=/path/to/games.pgn

# pgn meta data
event="Engine Turnament"
site="local"
round="1"

# the time settings in seconds
time_per_game="1"
time_per_move="0"


./engine_manager $engine_1 $engine_2 $logfile $pgn $event $site $round $time_per_game $time_per_move
```

Please note that not all UCI functionalities have been implemented yet.
