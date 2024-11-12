#!/bin/bash

cd "$(dirname "$0")"
WORKDIR=$PWD

export RUST_BACKTRACE=full

# Function for the tournament
run_tournament() {
    engine_dir="engines"

    engines=(
        "mewel_V0.3.3.sh"
        "suprah-arm"
    )

    # Tournament-specific variables
    event="Tournament_03"
    pgn="./${event}.pgn"
    round=1
    
    touch $pgn

    for ((i=0; i<${#engines[@]}; i++)); do
        for ((j=i+1; j<${#engines[@]}; j++)); do
            e1="${engine_dir}/${engines[$i]}"
            e2="${engine_dir}/${engines[$j]}"

            # Display which engines are currently playing
            echo "Round $round: ${engines[$i]} (White) vs ${engines[$j]} (Black)"
            ./Matt-Magie-arm "$e1" "$e2" "$logfile" "$pgn" "$event" "$site" "$round" "$time_per_game" "$logging"

            # Output the tail of the PGN file after each game
            tail "$pgn"
            round=$((round+1))

            # Display which engines are currently playing
            echo "Round $round: ${engines[$j]} (White) vs ${engines[$i]} (Black)"
            ./Matt-Magie-arm "$e2" "$e1" "$logfile" "$pgn" "$event" "$site" "$round" "$time_per_game" "$logging"

            # Output the tail of the PGN file after each game
            tail "$pgn"
            round=$((round+1))

            sleep 20  # Pause after each engine pair
        done
    done

    echo "FINISHED"
}

# Default engines (used when not in tournament mode)
#engine_1=./engines/suprah-arm
engine_1=./engines/suprah-arm
#engine_2=./engines/mewel_V0.3.3.sh
engine_2=./engines/mewel_V0.1.sh

# Default variables
logfile=./mattmagie.log
pgn=./games.pgn

# PGN metadata (please avoid spaces)
event="Game"
site="local"
round="1"

# Time settings in ms
time_per_game="30000"

# Logging setting
logging="log_on"

# Initialize variables
tournament_mode=0

# Parse command line options
while getopts "ct" opt; do
    case $opt in
        c)
            # Swap engines if -c is specified
            tmp=$engine_1
            engine_1=$engine_2
            engine_2=$tmp
            ;;
        t)
            # Enable tournament mode if -t is specified
            tournament_mode=1
            ;;
        *)
            echo "Usage: $0 [-c] [-t]"
            echo "  -c  Swap engine colors (engine_2 plays as white)"
            echo "  -t  Run in tournament mode"
            exit 1
            ;;
    esac
done

# Main logic
if [ $tournament_mode -eq 1 ]; then
    # Tournament mode active
    run_tournament
else
    # Single game between engine_1 and engine_2
    ./Matt-Magie-arm "$engine_1" "$engine_2" "$logfile" "$pgn" "$event" "$site" "$round" "$time_per_game" "$logging"
fi
