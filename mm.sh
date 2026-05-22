#!/bin/bash

# Ensure we are in the script's directory
cd "$(dirname "$0")"
WORKDIR=$PWD

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Path to Matt-Magie executable
MM_EXEC="./target/release/Matt-Magie"
if [[ ! -f "$MM_EXEC" ]]; then
    if [[ -f "./Matt-Magie-arm" ]]; then
        MM_EXEC="./Matt-Magie-arm"
    elif [[ -f "./buildVersions/Matt-Magie-x86" ]]; then
        MM_EXEC="./buildVersions/Matt-Magie-x86"
    elif [[ -f "./buildVersions/Matt-Magie-arm" ]]; then
        MM_EXEC="./buildVersions/Matt-Magie-arm"
    else
        # Try compiling if not found (to be fully user-friendly)
        echo -e "${YELLOW}Matt-Magie binary was not found in the release directory.${NC}"
        echo -e "Attempting native compilation with cargo build --release..."
        cargo build --release
        if [ $? -eq 0 ]; then
            MM_EXEC="./target/release/Matt-Magie"
            echo -e "${GREEN}Successfully compiled natively!${NC}"
        else
            echo -e "${RED}Error: Matt-Magie engine manager could not be compiled!${NC}"
            exit 1
        fi
    fi
fi

# Banners and Header
print_header() {
    clear
    echo -e "${CYAN}================================================================${NC}"
    echo -e "${CYAN}      __  ___      __  __         __  ___              _        ${NC}"
    echo -e "${CYAN}     /  |/  /___ _/ /_/ /_       /  |/  /___ _____ _  (_)___    ${NC}"
    echo -e "${CYAN}    / /|_/ / __ \`/ __/ __/______/ /|_/ / __ \`/ __ \`/ / / __ \\   ${NC}"
    echo -e "${CYAN}   / /  / / /_/ / /_/ /_ /_____/ /  / / /_/ / /_/ / / / /_/ /   ${NC}"
    echo -e "${CYAN}  /_/  /_/\\__,_/\\__/\\__/      /_/  /_/\\__,_/\\__, /_/_/\\____/    ${NC}"
    echo -e "${CYAN}                                           /____/               ${NC}"
    echo -e "${CYAN}                  SUPRAH CHESS ENGINE MATCHUP MANAGER           ${NC}"
    echo -e "${CYAN}================================================================${NC}"
    echo ""
}

# Scan engines directory
list_engines() {
    local files=()
    if [ -d "engines" ]; then
        for f in engines/*; do
            if [[ -x "$f" && -f "$f" ]]; then
                files+=("$(basename "$f")")
            fi
        done
    fi
    echo "${files[@]}"
}

# Import Engine from suprah build
import_engine() {
    print_header
    echo -e "${YELLOW}--- Import / Update Suprah Engine ---${NC}"
    echo ""
    
    local source_path="../suprah/target/release/suprah"
    if [[ ! -f "$source_path" ]]; then
        echo -e "${RED}Error: Suprah release binary not found at:${NC}"
        echo -e "  $source_path"
        echo -e "Please build suprah first using: cargo build --release in the suprah directory."
        echo ""
        read -p "Press Enter to continue..." temp
        return
    fi
    
    echo -e "${GREEN}Found:${NC} Suprah binary at $source_path"
    echo ""
    read -p "Enter a tag/label for this version (e.g. v0.2.0, experimental, v2): " tag
    
    if [[ -z "$tag" ]]; then
        tag="imported-$(date +%Y%m%d-%H%M)"
    fi
    
    local target_path="engines/suprah-$tag"
    mkdir -p engines
    cp "$source_path" "$target_path"
    chmod +x "$target_path"
    
    echo ""
    echo -e "${GREEN}Successfully imported!${NC}"
    echo -e "File saved at: ${BOLD}$target_path${NC}"
    echo ""
    read -p "Press Enter to continue..." temp
}

# Run Single Match (1 vs 1)
run_single_match() {
    local engines=($(list_engines))
    if [ ${#engines[@]} -lt 1 ]; then
        print_header
        echo -e "${RED}No chess engines found in the 'engines/' directory!${NC}"
        echo -e "Please import a version first or place it there manually."
        echo ""
        read -p "Press Enter to continue..." temp
        return
    fi

    print_header
    echo -e "${YELLOW}--- Configure Single Match ---${NC}"
    echo ""
    echo "Available Engines:"
    for i in "${!engines[@]}"; do
        echo -e "  [${CYAN}$((i+1))${NC}] ${engines[$i]}"
    done
    echo ""

    # Choose Engine 1 (White)
    local e1_idx=""
    while true; do
        read -p "Choose Engine 1 [1-${#engines[@]}]: " choice
        if [[ "$choice" -ge 1 && "$choice" -le "${#engines[@]}" ]] 2>/dev/null; then
            e1_idx=$((choice-1))
            break
        fi
        echo -e "${RED}Invalid choice.${NC}"
    done

    # Choose Engine 2 (Black)
    local e2_idx=""
    while true; do
        read -p "Choose Engine 2 [1-${#engines[@]}]: " choice
        if [[ "$choice" -ge 1 && "$choice" -le "${#engines[@]}" ]] 2>/dev/null; then
            e2_idx=$((choice-1))
            break
        fi
        echo -e "${RED}Invalid choice.${NC}"
    done

    local engine1="engines/${engines[$e1_idx]}"
    local engine2="engines/${engines[$e2_idx]}"

    echo ""
    # Time Settings
    local time_control="30000"
    read -p "Time control per game in milliseconds [Default: 30000 ms = 30s]: " tc_input
    if [[ ! -z "$tc_input" ]]; then
        time_control="$tc_input"
    fi

    local time_inc="0"
    read -p "Increment per move in milliseconds [Default: 0 ms]: " inc_input
    if [[ ! -z "$inc_input" ]]; then
        time_inc="$inc_input"
    fi

    local rounds="1"
    read -p "Number of rounds (each round plays both White and Black) [Default: 1]: " rounds_input
    if [[ ! -z "$rounds_input" ]]; then
        rounds="$rounds_input"
    fi

    print_header
    echo -e "${GREEN}Match started:${NC}"
    echo -e "  White: ${BOLD}${engines[$e1_idx]}${NC}"
    echo -e "  Black: ${BOLD}${engines[$e2_idx]}${NC}"
    echo -e "  Time Control: $((time_control/1000))s + $((time_inc))ms"
    echo -e "  Rounds: $rounds (Total games: $((rounds*2)))"
    echo ""

    local pgn="./suprah_games.pgn"
    local logfile="./mattmagie.log"
    local event="Suprah-Single-Match"
    local site="local"
    local logging="log_on"
    local debuging="debug_on"

    # Make sure pgn exists
    touch "$pgn"

    local current_round=1
    for ((r=0; r<rounds; r++)); do
        echo -e "${YELLOW}=== Round $current_round.1: ${engines[$e1_idx]} (White) vs ${engines[$e2_idx]} (Black) ===${NC}"
        $MM_EXEC "$engine1" "$engine2" "$logfile" "$pgn" "$event" "$site" "$current_round" "$time_control" "$time_inc" "$logging" "$debuging"
        tail -n 12 "$pgn"
        echo ""

        echo -e "${YELLOW}=== Round $current_round.2: ${engines[$e2_idx]} (White) vs ${engines[$e1_idx]} (Black) (Colors swapped) ===${NC}"
        $MM_EXEC "$engine2" "$engine1" "$logfile" "$pgn" "$event" "$site" "$current_round" "$time_control" "$time_inc" "$logging" "$debuging"
        tail -n 12 "$pgn"
        echo ""
        
        current_round=$((current_round+1))
        
        if [ $r -lt $((rounds-1)) ]; then
            echo "Short break (2s)..."
            sleep 2
        fi
    done

    echo -e "${GREEN}Match finished! Here is the scoreboard summary:${NC}"
    if [ -f "./summary.sh" ]; then
        ./summary.sh "$pgn"
    else
        echo "No summary.sh found."
    fi
    echo ""
    read -p "Press Enter to continue..." temp
}

# Run Tournament
run_tournament() {
    local all_engines=($(list_engines))
    if [ ${#all_engines[@]} -lt 2 ]; then
        print_header
        echo -e "${RED}At least 2 engines in the 'engines/' directory are required to run a tournament!${NC}"
        echo -e "Found: ${#all_engines[@]}"
        echo ""
        read -p "Press Enter to continue..." temp
        return
    fi

    print_header
    echo -e "${YELLOW}--- Configure Tournament Mode ---${NC}"
    echo ""
    echo "Available Engines:"
    for i in "${!all_engines[@]}"; do
        echo -e "  [${CYAN}$((i+1))${NC}] ${all_engines[$i]}"
    done
    echo ""

    local engines=()
    while true; do
        read -p "Select participating engines (comma-separated numbers, e.g. 1,3,4, or 'all') [Default: all]: " selection
        if [[ -z "$selection" || "$selection" == "all" ]]; then
            engines=("${all_engines[@]}")
            break
        fi

        # Parse comma-separated list
        IFS=',' read -r -a selected_indices <<< "$selection"
        local valid=true
        local selected_engines=()
        for idx in "${selected_indices[@]}"; do
            idx=$(echo "$idx" | tr -d ' ')
            if [[ "$idx" =~ ^[0-9]+$ && "$idx" -ge 1 && "$idx" -le ${#all_engines[@]} ]]; then
                selected_engines+=("${all_engines[$((idx-1))]}")
            else
                valid=false
                break
            fi
        done

        if [ "$valid" = true ]; then
            if [ ${#selected_engines[@]} -lt 2 ]; then
                echo -e "${RED}Error: You must select at least 2 engines!${NC}"
            else
                engines=("${selected_engines[@]}")
                break
            fi
        else
            echo -e "${RED}Invalid input. Please enter valid numbers separated by commas (e.g. 1,3,4).${NC}"
        fi
    done

    echo ""
    echo "Participating Engines:"
    for i in "${!engines[@]}"; do
        echo -e "  - ${CYAN}${engines[$i]}${NC}"
    done
    echo ""

    # Time Settings
    local time_control="30000"
    read -p "Time control per game in milliseconds [Default: 30000 ms = 30s]: " tc_input
    if [[ ! -z "$tc_input" ]]; then
        time_control="$tc_input"
    fi

    local time_inc="0"
    read -p "Increment per move in milliseconds [Default: 0 ms]: " inc_input
    if [[ ! -z "$inc_input" ]]; then
        time_inc="$inc_input"
    fi

    local rounds="1"
    read -p "Number of rounds (each pair plays both White and Black per round) [Default: 1]: " rounds_input
    if [[ ! -z "$rounds_input" ]]; then
        rounds="$rounds_input"
    fi

    # PGN Filename Setting
    local pgn=""
    while true; do
        read -p "Enter PGN filename (required, e.g. my_tournament.pgn): " pgn_input
        pgn_input=$(echo "$pgn_input" | tr -d ' ')
        if [[ ! -z "$pgn_input" ]]; then
            if [[ "$pgn_input" != *.pgn ]]; then
                pgn_input="${pgn_input}.pgn"
            fi
            pgn="./$pgn_input"
            break
        fi
        echo -e "${RED}PGN filename is required!${NC}"
    done

    # Calculate total games
    local num_engines=${#engines[@]}
    local match_pairs=$((num_engines * (num_engines - 1) / 2))
    local total_games=$((match_pairs * 2 * rounds))

    print_header
    echo -e "${GREEN}Tournament started:${NC}"
    echo -e "  Number of Engines: $num_engines"
    echo -e "  Time Control: $((time_control/1000))s + $((time_inc))ms"
    echo -e "  Rounds: $rounds"
    echo -e "  Total Games: $total_games"
    echo ""

    # pgn is defined dynamically above
    local logfile="./mattmagie.log"
    local event="Suprah-Tournament"
    local site="local"
    local logging="log_on"
    local debuging="debug_on"

    # If the PGN file already exists, notify the user and append to it. Otherwise, create it empty.
    if [[ -f "$pgn" ]]; then
        echo -e "${YELLOW}Note: PGN file '$pgn_input' already exists. New games will be appended!${NC}"
        echo ""
    else
        touch "$pgn"
    fi

    local game_num=1
    for ((r=0; r<rounds; r++)); do
        for ((i=0; i<num_engines; i++)); do
            for ((j=i+1; j<num_engines; j++)); do
                local e1_name="${engines[$i]}"
                local e2_name="${engines[$j]}"
                local e1="engines/$e1_name"
                local e2="engines/$e2_name"

                echo -e "${YELLOW}=== Game $game_num/$total_games: $e1_name (White) vs $e2_name (Black) ===${NC}"
                $MM_EXEC "$e1" "$e2" "$logfile" "$pgn" "$event" "$site" "$game_num" "$time_control" "$time_inc" "$logging" "$debuging"
                tail -n 12 "$pgn"
                echo ""
                game_num=$((game_num+1))

                echo -e "${YELLOW}=== Game $game_num/$total_games: $e2_name (White) vs $e1_name (Black) (Colors swapped) ===${NC}"
                $MM_EXEC "$e2" "$e1" "$logfile" "$pgn" "$event" "$site" "$game_num" "$time_control" "$time_inc" "$logging" "$debuging"
                tail -n 12 "$pgn"
                echo ""
                game_num=$((game_num+1))

                sleep 1
            done
        done
    done

    echo -e "${GREEN}Tournament finished! Here is the final scoreboard:${NC}"
    if [ -f "./summary.sh" ]; then
        ./summary.sh "$pgn"
    else
        echo "No summary.sh found."
    fi
    echo ""
    read -p "Press Enter to continue..." temp
}

# Main Menu Loop
while true; do
    print_header
    echo -e "What would you like to do?"
    echo -e "  [${CYAN}1${NC}] Single Match (2 Versions Head-to-Head)"
    echo -e "  [${CYAN}2${NC}] Tournament (Round-Robin All-vs-All)"
    echo -e "  [${CYAN}3${NC}] Import / Update New Suprah Version"
    echo -e "  [${CYAN}4${NC}] View Current PGN Statistics"
    echo -e "  [${CYAN}5${NC}] Exit"
    echo ""
    read -p "Choice [1-5]: " choice
    case $choice in
        1)
            run_single_match
            ;;
        2)
            run_tournament
            ;;
        3)
            import_engine
            ;;
        4)
            print_header
            echo -e "${YELLOW}--- Current PGN Statistics ---${NC}"
            echo ""
            echo "Available PGN files:"
            pgns=(*.pgn)
            if [ -e "${pgns[0]}" ]; then
                for i in "${!pgns[@]}"; do
                    echo -e "  [${CYAN}$((i+1))${NC}] ${pgns[$i]}"
                done
                echo ""
                read -p "Select a PGN file to analyze [1-${#pgns[@]}]: " pgn_choice
                if [[ "$pgn_choice" -ge 1 && "$pgn_choice" -le "${#pgns[@]}" ]] 2>/dev/null; then
                    selected_pgn="${pgns[$((pgn_choice-1))]}"
                    print_header
                    echo -e "${GREEN}Statistics for $selected_pgn:${NC}"
                    echo ""
                    ./summary.sh "$selected_pgn"
                else
                    echo -e "${RED}Invalid choice.${NC}"
                fi
            else
                echo -e "${RED}No PGN files found.${NC}"
            fi
            echo ""
            read -p "Press Enter to continue..." temp
            ;;
        5)
            echo -e "${GREEN}Goodbye! Good luck with Suprah!${NC}"
            exit 0
            ;;
        *)
            echo -e "${RED}Invalid choice, please select 1-5.${NC}"
            sleep 1.5
            ;;
    esac
done
