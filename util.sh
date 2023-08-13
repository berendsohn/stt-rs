# Utilities for benchmark scripts


progress_bar_start() {
    echo -en "[\e7" # Save cursor
    for _ in $(eval echo {1..$REPEAT})
    do
        echo -n "-"
    done
    echo -en "]\e8" # Restore cursor
}

progress_bar_tick() { echo -n "+"; }

progress_bar_end() { echo; }
