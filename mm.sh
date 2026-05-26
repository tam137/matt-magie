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
    echo -e "${CYAN}               SUPRAH CHESS ENGINE MATCHUP MANAGER v1.4.2       ${NC}"
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

    local engine_options=""
    read -p "Engine options (comma-separated, e.g. Hash=128,Threads=1) [Default: none]: " options_input
    if [[ ! -z "$options_input" ]]; then
        engine_options="$options_input"
    fi

    print_header
    echo -e "${GREEN}Match started:${NC}"
    echo -e "  White: ${BOLD}${engines[$e1_idx]}${NC}"
    echo -e "  Black: ${BOLD}${engines[$e2_idx]}${NC}"
    echo -e "  Time Control: $((time_control/1000))s + $((time_inc))ms"
    echo -e "  Rounds: $rounds (Total games: $((rounds*2)))"
    if [[ ! -z "$engine_options" ]]; then
        echo -e "  Engine Options: $engine_options"
    fi
    echo ""

    local pgn="./games.pgn"
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
        $MM_EXEC "$engine1" "$engine2" "$logfile" "$pgn" "$event" "$site" "$current_round" "$time_control" "$time_inc" "$logging" "$debuging" "$engine_options"
        tail -n 12 "$pgn"
        echo ""

        echo -e "${YELLOW}=== Round $current_round.2: ${engines[$e2_idx]} (White) vs ${engines[$e1_idx]} (Black) (Colors swapped) ===${NC}"
        $MM_EXEC "$engine2" "$engine1" "$logfile" "$pgn" "$event" "$site" "$current_round" "$time_control" "$time_inc" "$logging" "$debuging" "$engine_options"
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

# Execute Tournament Games (Non-interactive & Interactive backend)
execute_tournament_games() {
    local engines_str="$1"
    local time_control="$2"
    local time_inc="$3"
    local rounds="$4"
    local pgn="$5"
    local engine_options="${6:-}"
    local tournament_mode="${7:-round_robin}"

    # Convert comma-separated string back to array
    local OLD_IFS="$IFS"
    IFS=',' read -r -a engines <<< "$engines_str"
    IFS="$OLD_IFS"

    local num_engines=${#engines[@]}
    local match_pairs=0
    if [[ "$tournament_mode" == "gauntlet" ]]; then
        match_pairs=$((num_engines - 1))
    else
        match_pairs=$((num_engines * (num_engines - 1) / 2))
    fi
    local total_games=$((match_pairs * 2 * rounds))

    print_header
    echo -e "${GREEN}Tournament started:${NC}"
    echo -e "  Number of Engines: $num_engines"
    echo -e "  Time Control: $((time_control/1000))s + $((time_inc))ms"
    echo -e "  Rounds: $rounds"
    echo -e "  Total Games: $total_games"
    if [[ ! -z "$engine_options" ]]; then
        echo -e "  Engine Options: $engine_options"
    fi
    echo ""

    local logfile="./mattmagie.log"
    local event="Suprah-Tournament"
    local site="local"
    local logging="log_on"
    local debuging="debug_on"

    if [[ -f "$pgn" ]]; then
        echo -e "${YELLOW}Note: PGN file '$(basename "$pgn")' already exists. New games will be appended!${NC}"
        echo ""
    else
        touch "$pgn"
    fi

    local game_num=1
    for ((r=0; r<rounds; r++)); do
        if [[ "$tournament_mode" == "gauntlet" ]]; then
            local challenger_name="${engines[0]}"
            for ((j=1; j<num_engines; j++)); do
                local opp_name="${engines[$j]}"
                local e1="engines/$challenger_name"
                local e2="engines/$opp_name"

                echo -e "${YELLOW}=== Game $game_num/$total_games: $challenger_name (White) vs $opp_name (Black) ===${NC}"
                $MM_EXEC "$e1" "$e2" "$logfile" "$pgn" "$event" "$site" "$game_num" "$time_control" "$time_inc" "$logging" "$debuging" "$engine_options"
                tail -n 12 "$pgn"
                echo ""
                game_num=$((game_num+1))

                echo -e "${YELLOW}=== Game $game_num/$total_games: $opp_name (White) vs $challenger_name (Black) (Colors swapped) ===${NC}"
                $MM_EXEC "$e2" "$e1" "$logfile" "$pgn" "$event" "$site" "$game_num" "$time_control" "$time_inc" "$logging" "$debuging" "$engine_options"
                tail -n 12 "$pgn"
                echo ""
                game_num=$((game_num+1))

                sleep 1
            done
        else
            for ((i=0; i<num_engines; i++)); do
                for ((j=i+1; j<num_engines; j++)); do
                    local e1_name="${engines[$i]}"
                    local e2_name="${engines[$j]}"
                    local e1="engines/$e1_name"
                    local e2="engines/$e2_name"

                    echo -e "${YELLOW}=== Game $game_num/$total_games: $e1_name (White) vs $e2_name (Black) ===${NC}"
                    $MM_EXEC "$e1" "$e2" "$logfile" "$pgn" "$event" "$site" "$game_num" "$time_control" "$time_inc" "$logging" "$debuging" "$engine_options"
                    tail -n 12 "$pgn"
                    echo ""
                    game_num=$((game_num+1))

                    echo -e "${YELLOW}=== Game $game_num/$total_games: $e2_name (White) vs $e1_name (Black) (Colors swapped) ===${NC}"
                    $MM_EXEC "$e2" "$e1" "$logfile" "$pgn" "$event" "$site" "$game_num" "$time_control" "$time_inc" "$logging" "$debuging" "$engine_options"
                    tail -n 12 "$pgn"
                    echo ""
                    game_num=$((game_num+1))

                    sleep 1
                done
            done
        fi
    done

    echo -e "${GREEN}Tournament finished! Here is the final scoreboard:${NC}"
    if [ -f "./summary.sh" ]; then
        if [[ "$tournament_mode" == "gauntlet" ]]; then
            ./summary.sh "$pgn" --gauntlet "${engines[0]}"
        else
            ./summary.sh "$pgn"
        fi
    else
        echo "No summary.sh found."
    fi
    echo ""
}

# Run Tournament from configuration file (.trn)
run_file_tournament() {
    local trn_file="$1"

    if [[ ! -f "$trn_file" ]]; then
        echo -e "${RED}Error: Tournament file '$trn_file' not found!${NC}"
        exit 1
    fi

    # Read and parse key-value pairs
    local engines_val=""
    local tc_val=""
    local inc_val=""
    local rounds_val=""
    local pgn_val=""
    local options_val=""
    local mode_val=""

    while IFS= read -r line || [[ -n "$line" ]]; do
        # Strip comments starting with #
        line="${line%%#*}"
        # Trim leading/trailing whitespace
        line=$(echo "$line" | xargs)
        
        # Skip empty lines
        [[ -z "$line" ]] && continue

        # Parse key = value
        if [[ "$line" =~ ^([a-zA-Z_][a-zA-Z0-9_]*)[[:space:]]*=[[:space:]]*(.*)$ ]]; then
            local key="${BASH_REMATCH[1]}"
            local val="${BASH_REMATCH[2]}"
            
            case "$key" in
                engines)
                    engines_val="$val"
                    ;;
                time_control)
                    tc_val="$val"
                    ;;
                increment)
                    inc_val="$val"
                    ;;
                rounds)
                    rounds_val="$val"
                    ;;
                pgn)
                    pgn_val="$val"
                    ;;
                engine_options)
                    options_val="$val"
                    ;;
                mode)
                    mode_val="$val"
                    ;;
                *)
                    echo -e "${YELLOW}Warning: Unknown key '$key' in tournament file.${NC}"
                    ;;
            esac
        fi
    done < "$trn_file"

    # Validation
    if [[ -z "$engines_val" ]]; then
        echo -e "${RED}Error: 'engines' is not specified or empty in '$trn_file'!${NC}"
        exit 1
    fi
    if [[ -z "$tc_val" ]]; then
        echo -e "${RED}Error: 'time_control' is not specified or empty in '$trn_file'!${NC}"
        exit 1
    fi
    if [[ -z "$inc_val" ]]; then
        echo -e "${RED}Error: 'increment' is not specified or empty in '$trn_file'!${NC}"
        exit 1
    fi
    if [[ -z "$rounds_val" ]]; then
        echo -e "${RED}Error: 'rounds' is not specified or empty in '$trn_file'!${NC}"
        exit 1
    fi
    if [[ -z "$pgn_val" ]]; then
        echo -e "${RED}Error: 'pgn' is not specified or empty in '$trn_file'!${NC}"
        exit 1
    fi

    # Set default tournament mode
    if [[ -z "$mode_val" ]]; then
        mode_val="round_robin"
    fi
    if [[ "$mode_val" != "round_robin" && "$mode_val" != "gauntlet" ]]; then
        echo -e "${RED}Error: 'mode' must be either 'round_robin' or 'gauntlet', found '$mode_val'!${NC}"
        exit 1
    fi

    # Validate rounds (must be a positive integer)
    if [[ ! "$rounds_val" =~ ^[0-9]+$ || "$rounds_val" -le 0 ]]; then
        echo -e "${RED}Error: 'rounds' must be a positive integer, found '$rounds_val'!${NC}"
        exit 1
    fi

    # Validate time_control (must be a non-negative integer)
    if [[ ! "$tc_val" =~ ^[0-9]+$ ]]; then
        echo -e "${RED}Error: 'time_control' must be a non-negative integer, found '$tc_val'!${NC}"
        exit 1
    fi

    # Validate increment (must be a non-negative integer)
    if [[ ! "$inc_val" =~ ^[0-9]+$ ]]; then
        echo -e "${RED}Error: 'increment' must be a non-negative integer, found '$inc_val'!${NC}"
        exit 1
    fi

    # Format engines list: remove whitespaces around commas and clean names
    local engines=()
    local OLD_IFS="$IFS"
    IFS=','
    for eng in $engines_val; do
        eng=$(echo "$eng" | xargs)
        if [[ -n "$eng" ]]; then
            engines+=("$eng")
        fi
    done
    IFS="$OLD_IFS"

    if [ ${#engines[@]} -lt 2 ]; then
        echo -e "${RED}Error: At least 2 engines must be specified, found only ${#engines[@]}!${NC}"
        exit 1
    fi

    # Validate that all engines exist and are executable in engines/ directory
    for eng in "${engines[@]}"; do
        local path="engines/$eng"
        if [[ ! -f "$path" ]]; then
            echo -e "${RED}Error: Engine '$eng' not found at '$path'!${NC}"
            exit 1
        fi
        if [[ ! -x "$path" ]]; then
            echo -e "${RED}Error: Engine '$eng' at '$path' is not executable!${NC}"
            exit 1
        fi
    done

    # Parse PGN filename (append .pgn if missing)
    if [[ "$pgn_val" != *.pgn ]]; then
        pgn_val="${pgn_val}.pgn"
    fi
    local pgn_path="./$pgn_val"

    # Re-create comma separated list of clean engine names
    local engines_clean_str
    engines_clean_str=$(IFS=,; echo "${engines[*]}")

    # Run the tournament games
    execute_tournament_games "$engines_clean_str" "$tc_val" "$inc_val" "$rounds_val" "$pgn_path" "$options_val" "$mode_val"
}

# Run Tournament (Interactive configuration)
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

    # Tournament Mode Selection
    local mode_val="round_robin"
    echo -e "Select Tournament Mode:"
    echo -e "  [${CYAN}1${NC}] Round-Robin (All-vs-All)"
    echo -e "  [${CYAN}2${NC}] Gauntlet (Engine 1 plays against all others)"
    echo ""
    local mode_choice=""
    while true; do
        read -p "Choice [1-2] [Default: 1]: " mode_choice
        if [[ -z "$mode_choice" || "$mode_choice" == "1" ]]; then
            mode_val="round_robin"
            break
        elif [[ "$mode_choice" == "2" ]]; then
            mode_val="gauntlet"
            break
        fi
        echo -e "${RED}Invalid choice, please select 1 or 2.${NC}"
    done
    echo ""

    if [[ "$mode_val" == "gauntlet" ]]; then
        echo -e "${YELLOW}Gauntlet Mode Active: Engine '${engines[0]}' is the Challenger!${NC}"
        echo ""
    fi

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

    local engine_options=""
    read -p "Engine options (comma-separated, e.g. Hash=128,Threads=1) [Default: none]: " options_input
    if [[ ! -z "$options_input" ]]; then
        engine_options="$options_input"
    fi

    while true; do
        local engines_str
        engines_str=$(IFS=,; echo "${engines[*]}")

        execute_tournament_games "$engines_str" "$time_control" "$time_inc" "$rounds" "$pgn" "$engine_options" "$mode_val"

        # Post-tournament replay choice
        echo -e "What would you like to do?"
        echo -e "  [${CYAN}1${NC}] Replay tournament with the same settings (same engines and time controls)"
        echo -e "  [${CYAN}2${NC}] Finish tournament and return to main menu"
        echo ""

        local replay_choice=""
        while true; do
            read -p "Choice [1-2]: " replay_choice
            if [[ "$replay_choice" == "1" || "$replay_choice" == "2" ]]; then
                break
            fi
            echo -e "${RED}Invalid choice, please select 1-2.${NC}"
        done

        if [[ "$replay_choice" == "2" ]]; then
            break
        fi

        # User wants to replay, prompt for rounds (default is previous rounds)
        echo ""
        read -p "Number of rounds (each pair plays both White and Black per round) [Default: $rounds]: " rounds_input
        if [[ ! -z "$rounds_input" ]]; then
            if [[ "$rounds_input" =~ ^[0-9]+$ && "$rounds_input" -ge 1 ]]; then
                rounds="$rounds_input"
            else
                echo -e "${YELLOW}Invalid rounds input. Keeping previous value of $rounds.${NC}"
                sleep 1.5
            fi
        fi
    done
}

# Check if command-line arguments are provided
if [[ $# -gt 0 ]]; then
    if [[ "$1" == "-t" ]]; then
        if [[ -z "$2" ]]; then
            echo -e "${RED}Error: Missing tournament file path! Usage: ./mm.sh -t <tournament_file.trn>${NC}"
            exit 1
        fi
        run_file_tournament "$2"
        exit 0
    else
        echo -e "${RED}Unknown option: $1${NC}"
        echo -e "Usage: ./mm.sh [-t <tournament_file.trn>]"
        exit 1
    fi
fi

# Main Menu Loop
while true; do
    print_header
    echo -e "What would you like to do?"
    echo -e "  [${CYAN}1${NC}] Single Match (2 Versions Head-to-Head)"
    echo -e "  [${CYAN}2${NC}] Tournament (Round-Robin All-vs-All)"
    echo -e "  [${CYAN}3${NC}] Import / Update New Suprah Version"
    echo -e "  [${CYAN}4${NC}] View Current PGN Statistics"
    echo -e "  [${CYAN}9${NC}] Exit"
    echo ""
    read -p "Choice [1-4, 9]: " choice
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
        9)
            echo -e "${GREEN}Goodbye! Good luck with Suprah!${NC}"
            exit 0
            ;;
        *)
            echo -e "${RED}Invalid choice, please select 1-4 or 9.${NC}"
            sleep 1.5
            ;;
    esac
done
