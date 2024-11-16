#!/bin/bash

PGN_FILE="$1"

if [ ! -f "$PGN_FILE" ]; then
    echo "FIle $PGN_FILE not found."
    exit 1
fi

gawk '
BEGIN {
    RS = ""
    FS = "\n"
}

{
    result = ""
    white = ""
    black = ""

    for (i = 1; i <= NF; i++) {
        if ($i ~ /^\[Result "/) {
            match($i, /^\[Result "([^"]+)"/, arr)
            result = arr[1]
        }
        else if ($i ~ /^\[White "/) {
            match($i, /^\[White "([^"]+)"/, arr)
            white = arr[1]
        }
        else if ($i ~ /^\[Black "/) {
            match($i, /^\[Black "([^"]+)"/, arr)
            black = arr[1]
        }
    }

    if (result != "" && white != "" && black != "") {
        print result " " white " vs " black

        # Punktevergabe
        if (result == "1-0") {
            points[white] += 1
            games[white] += 1
            games[black] += 1
        } else if (result == "0-1") {
            points[black] += 1
            games[white] += 1
            games[black] += 1
        } else if (result == "1/2-1/2") {
            points[white] += 0.5
            points[black] += 0.5
            games[white] += 1
            games[black] += 1
        }
    }
}

END {
    PROCINFO["sorted_in"] = "@val_num_desc"

    print "\n[Platz]\t[Spiele]\t[Punkte]\t[EngineName]"
    rank = 0
    for (player in points) {
        rank += 1
        printf "%d\t%d\t%.1f\t%s\n", rank, games[player], points[player], player
    }
}
' "$PGN_FILE"

