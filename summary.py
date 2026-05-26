#!/usr/bin/env python3
import sys
import re
from collections import defaultdict

def parse_pgn(file_path):
    games = []
    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()
    except Exception as e:
        print(f"Error reading file {file_path}: {e}")
        return games
    
    # Split the content by [Event " to isolate each game block
    game_blocks = re.split(r'(?=\[Event\s+)', content)
    for block in game_blocks:
        if not block.strip():
            continue
        tags = {}
        for line in block.split('\n'):
            line = line.strip()
            match = re.match(r'\[(\w+)\s+"([^"]*)"\]', line)
            if match:
                tags[match.group(1)] = match.group(2)
        if 'White' in tags and 'Black' in tags and 'Result' in tags:
            games.append(tags)
    return games

def compute_ratings_and_scores(games):
    stats = defaultdict(lambda: {
        'games': 0,
        'wins': 0,
        'draws': 0,
        'losses': 0,
        'points': 0.0
    })
    
    # Elo ratings anchored at 1500
    ratings = defaultdict(lambda: 1500.0)
    K = 32.0
    
    for game in games:
        w = game['White'].strip()
        b = game['Black'].strip()
        res = game['Result'].strip()
        
        if res not in ("1-0", "0-1", "1/2-1/2"):
            continue
            
        stats[w]['games'] += 1
        stats[b]['games'] += 1
        
        if res == "1-0":
            s_w, s_b = 1.0, 0.0
            stats[w]['wins'] += 1
            stats[b]['losses'] += 1
            stats[w]['points'] += 1.0
        elif res == "0-1":
            s_w, s_b = 0.0, 1.0
            stats[w]['losses'] += 1
            stats[b]['wins'] += 1
            stats[b]['points'] += 1.0
        else: # "1/2-1/2"
            s_w, s_b = 0.5, 0.5
            stats[w]['draws'] += 1
            stats[b]['draws'] += 1
            stats[w]['points'] += 0.5
            stats[b]['points'] += 0.5
            
        # Get current ratings
        r_w = ratings[w]
        r_b = ratings[b]
        
        # Expected scores
        e_w = 1.0 / (1.0 + 10.0 ** ((r_b - r_w) / 400.0))
        e_b = 1.0 / (1.0 + 10.0 ** ((r_w - r_b) / 400.0))
        
        # Update ratings
        ratings[w] = r_w + K * (s_w - e_w)
        ratings[b] = r_b + K * (s_b - e_b)
        
    return stats, ratings

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 summary.py <pgn_file> [--gauntlet <challenger_name>]")
        sys.exit(1)
        
    pgn_file = sys.argv[1]
    gauntlet_challenger = None
    if len(sys.argv) >= 4 and sys.argv[2] == "--gauntlet":
        gauntlet_challenger = sys.argv[3]

    games = parse_pgn(pgn_file)
    
    if not games:
        print(f"No games found in {pgn_file}.")
        sys.exit(0)
        
    # Print individual results
    print("=" * 70)
    print(" " * 23 + "INDIVIDUAL GAME RESULTS")
    print("=" * 70)
    for i, game in enumerate(games, 1):
        w = game['White']
        b = game['Black']
        res = game['Result']
        w_disp = (w[:23] + "..") if len(w) > 25 else w
        b_disp = (b[:23] + "..") if len(b) > 25 else b
        print(f"Game {i:<2}: {w_disp:<25} vs {b_disp:<25}  -> {res}")
    print()
        
    if gauntlet_challenger:
        # Resolve challenger name to what's in the PGN file
        engine_counts = defaultdict(int)
        for game in games:
            engine_counts[game['White'].strip()] += 1
            engine_counts[game['Black'].strip()] += 1
            
        detected_challenger = max(engine_counts, key=engine_counts.get) if engine_counts else gauntlet_challenger
        
        # Check if the passed name exists exactly
        if gauntlet_challenger.strip() not in engine_counts:
            # Try fuzzy matching (substring or common numbers like version digits)
            matched = None
            # Extract digits/version numbers e.g. "0.9.3" from "suprah-0.9.3"
            version_match = re.search(r'\d+\.\d+\.\d+', gauntlet_challenger)
            version_str = version_match.group(0) if version_match else None
            
            for eng in engine_counts:
                if version_str and version_str in eng:
                    matched = eng
                    break
                # Try simple clean alphanumeric match
                eng_clean = re.sub(r'[^a-zA-Z0-9]', '', eng.lower())
                target_clean = re.sub(r'[^a-zA-Z0-9]', '', gauntlet_challenger.lower())
                if target_clean in eng_clean or eng_clean in target_clean:
                    matched = eng
                    break
            
            if matched:
                gauntlet_challenger = matched
            else:
                gauntlet_challenger = detected_challenger
                
        print("=" * 70)
        print(" " * 19 + "GAUNTLET HEAD-TO-HEAD SUMMARY")
        print("=" * 70)
        print(f"Challenger: {gauntlet_challenger}")
        print("-" * 70)
        
        total_wins = 0
        total_draws = 0
        total_losses = 0
        h2h = defaultdict(lambda: {'wins': 0, 'draws': 0, 'losses': 0})
        
        for game in games:
            w = game['White'].strip()
            b = game['Black'].strip()
            res = game['Result'].strip()
            if res not in ("1-0", "0-1", "1/2-1/2"):
                continue
                
            if w == gauntlet_challenger:
                opp = b
                if res == "1-0":
                    h2h[opp]['wins'] += 1
                    total_wins += 1
                elif res == "0-1":
                    h2h[opp]['losses'] += 1
                    total_losses += 1
                else:
                    h2h[opp]['draws'] += 1
                    total_draws += 1
            elif b == gauntlet_challenger:
                opp = w
                if res == "0-1":
                    h2h[opp]['wins'] += 1
                    total_wins += 1
                elif res == "1-0":
                    h2h[opp]['losses'] += 1
                    total_losses += 1
                else:
                    h2h[opp]['draws'] += 1
                    total_draws += 1
                    
        for opp, st in sorted(h2h.items()):
            w_d_l = f"{st['wins']}/{st['draws']}/{st['losses']}"
            opp_disp = (opp[:30] + "..") if len(opp) > 32 else opp
            print(f"vs {opp_disp:<32} {w_d_l:<9}")
            
        print("-" * 70)
        total_wdl = f"{total_wins}/{total_draws}/{total_losses}"
        print(f"{'TOTAL (W/D/L):':<35} {total_wdl:<9}")
        print("=" * 70)
        return

    stats, ratings = compute_ratings_and_scores(games)
    
    # Sort engines by points descending, then by Elo descending
    sorted_engines = sorted(
        stats.keys(),
        key=lambda e: (stats[e]['points'], ratings[e]),
        reverse=True
    )
    
    # Print tournament scoreboard & ELO evaluation beautifully
    print("=" * 70)
    print(" " * 16 + "TOURNAMENT SCOREBOARD & ELO EVALUATION")
    print("=" * 70)
    print(f"{'Rank':<4} {'Engine Name':<25} {'Games':<5} {'W/D/L':<9} {'Points':<6} {'Score%':<6} {'Elo':<5} {'Elo+/-':<6}")
    print("-" * 70)
    
    for rank, engine in enumerate(sorted_engines, 1):
        estats = stats[engine]
        games_played = estats['games']
        w_d_l = f"{estats['wins']}/{estats['draws']}/{estats['losses']}"
        pts = estats['points']
        pct = (pts / games_played * 100.0) if games_played > 0 else 0.0
        elo = round(ratings[engine])
        elo_diff = elo - 1500
        elo_diff_str = f"+{elo_diff}" if elo_diff >= 0 else f"{elo_diff}"
        engine_disp = (engine[:23] + "..") if len(engine) > 25 else engine
        
        print(f"{rank:<4} {engine_disp:<25} {games_played:<5} {w_d_l:<9} {pts:<6.1f} {pct:<6.1f} {elo:<5} {elo_diff_str:<6}")
        
    print("=" * 70)
    print("Note: Elo starts at 1500 and updates sequentially per game.")
    print("=" * 70)

if __name__ == '__main__':
    main()
