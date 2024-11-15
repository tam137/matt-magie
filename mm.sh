#!/bin/bash

cd "$(dirname "$0")"
WORKDIR=$PWD

export RUST_BACKTRACE=full

# Function for the tournament
run_tournament() {
    engine_dir="engines"

    engines=(
        "suprah-arm"
        "mewel_V0.3.3.sh"        
        "mewel_V0.3.sh"
        "mewel_V0.1.sh"
    )

    # Check if all engines exist
    for engine in "${engines[@]}"; do
        if [[ ! -f "$engine_dir/$engine" ]]; then
            echo "Error: Engine '$engine' not found in directory '$engine_dir'."
            exit 1
        fi
    done

    # Override Tournament-specific variables
    event="Tournament_04"
    pgn="./${event}.pgn"
    round=1
    time_per_game="60000"
    
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

            sleep 10  # Pause after each engine pair
        done
    done

    echo "FINISHED"
}

# Default engines (used when not in tournament mode)
engine_1=./engines/suprah-arm
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
    if [[ ! -f "$engine_1" ]]; then
        echo "Error: Engine '$engine_1' not found."
        exit 1
    fi
    if [[ ! -f "$engine_2" ]]; then
        echo "Error: Engine '$engine_2' not found."
        exit 1
    fi
    ./Matt-Magie-arm "$engine_1" "$engine_2" "$logfile" "$pgn" "$event" "$site" "$round" "$time_per_game" "$logging"
fi