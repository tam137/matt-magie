import re
import sys
from collections import defaultdict

def analyze_log(log_path):
    # Regex to match engine ID lines
    # Example: 20:30:18.789 42325	->  mat		1_id name Rust-In-Pieces V0.2.6
    # Example info line: 20:36:58.442 48686	->  mat		0_info depth 4 score cp 64 time 9 nodes 7323 nps 732 pv a4b5 ...
    
    # We want to map PID to version.
    pid_to_version = {}
    
    # We want to track search statistics per version
    # stats[version] = { 'total_nodes': 0, 'total_time': 0, 'nps_values': [], 'max_depth': 0, 'moves': 0 }
    stats = defaultdict(lambda: {
        'total_nodes': 0,
        'total_time': 0,
        'nps_values': [],
        'depths': [],
        'info_lines_count': 0,
        'max_depth_reached': 0,
        'node_counts': [],
    })
    
    with open(log_path, 'r') as f:
        for line_num, line in enumerate(f, 1):
            if log_path == 'mattmagie.log' and line_num < 5762:
                continue
            line = line.strip()
            if not line:
                continue
            
            # Match engine name and PID
            # 20:37:08.440 49642	->  mat		1_id name Rust-In-Pieces V0.2.6
            name_match = re.search(r'(\d+)\s+->\s+mat\s+\d+_id name (Rust-In-Pieces V\d+\.\d+\.\d+)', line)
            if name_match:
                pid = int(name_match.group(1))
                version = name_match.group(2)
                pid_to_version[pid] = version
                continue
            
            # Match info line
            # 20:37:08.461 49641	->  mat		0_info depth 2 score cp 51 time 1 nodes 3713 nps 1856 pv ...
            info_match = re.search(r'(\d+)\s+->\s+mat\s+\d+_info depth (\d+) .* time (\d+) nodes (\d+) nps (\d+)', line)
            if info_match:
                pid = int(info_match.group(1))
                if pid in pid_to_version:
                    version = pid_to_version[pid]
                    depth = int(info_match.group(2))
                    time_ms = int(info_match.group(3))
                    nodes = int(info_match.group(4))
                    nps = int(info_match.group(5))
                    
                    stats[version]['total_nodes'] += nodes
                    stats[version]['total_time'] += time_ms
                    stats[version]['nps_values'].append(nps)
                    stats[version]['depths'].append(depth)
                    stats[version]['node_counts'].append(nodes)
                    stats[version]['max_depth_reached'] = max(stats[version]['max_depth_reached'], depth)
                    stats[version]['info_lines_count'] += 1

    print("=== ANALYSIS OF MATTMAGIE.LOG ===")
    for version, v_stats in sorted(stats.items()):
        nps_vals = v_stats['nps_values']
        depths = v_stats['depths']
        node_counts = v_stats['node_counts']
        
        if not nps_vals:
            print(f"{version}: No statistics found.")
            continue
            
        avg_nps = sum(nps_vals) / len(nps_vals)
        max_nps = max(nps_vals)
        avg_depth = sum(depths) / len(depths)
        max_depth = v_stats['max_depth_reached']
        total_nodes = sum(node_counts)
        avg_nodes = sum(node_counts) / len(node_counts)
        
        print(f"\nEngine: {version}")
        print(f"  Total Info Lines: {v_stats['info_lines_count']}")
        print(f"  Average NPS:      {avg_nps:.1f}")
        print(f"  Peak NPS:         {max_nps:,}")
        print(f"  Average Depth:    {avg_depth:.2f}")
        print(f"  Max Depth:        {max_depth}")
        print(f"  Total Nodes Searched: {total_nodes:,}")
        print(f"  Average Nodes/Search: {avg_nodes:,.1f}")

if __name__ == '__main__':
    log_file = 'mattmagie.log'
    if len(sys.argv) > 1:
        log_file = sys.argv[1]
    analyze_log(log_file)
