mod score;

use chrono::{self, TimeZone};
use std::io::Read;

fn time_of_day_str(timestamp: i64) -> String {
    let time = chrono::Local.timestamp_opt(timestamp, 0).unwrap();
    time.format("%Y-%m-%d %H:%M").to_string()
}

fn duration_str(secs: u32) -> String {
    // gives string in hours and mins
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    format!("{:2}h {:2}m", hours, mins)
}

fn main() {
    let mut score = score::Score::new();
    let mut buffer = [0; 5];
    while let Ok(_) = std::io::stdin().read_exact(&mut buffer) {
        let timestamp = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        let keypresses = buffer[4];
        score.append(timestamp, keypresses);

        println!(
            "{} {:9.0}; {:5}; {:3}; total work {}; rest {}; LoR {}; μ/d {:2.0}%; μ/h {:2.0}%; last recovery {}",
            time_of_day_str(timestamp as i64 * 5),
            score.current_score(),
            score.total_keypresses(),
            keypresses,
            duration_str(score.total_work()),
            duration_str(score.needed_recovery()),
            duration_str(score.lack_of_recovery()),
            score.micro_pause_share() * 100.0,
            score.micro_pause_share_past_hour() * 100.0,
            duration_str(score.last_recovery_since()),
        );

        // print work ranges in chronological order, if its work or rest, and the duration
        let work_ranges = score.work_ranges();
        let mut current_is_work = false;
        let mut current_begin = score.day_start();
        let mut current_end = current_begin;
        for time_slice in work_ranges {
            if time_slice > 60 * 10 {
                println!(
                    "    {} -- {} {} {}",
                    time_of_day_str(current_begin as i64),
                    time_of_day_str(current_end as i64),
                    if current_is_work { "work" } else { "rest" },
                    duration_str(current_end - current_begin),
                );
                current_is_work = !current_is_work;
                current_begin = current_end;
                current_end = current_begin + time_slice;
            } else {
                current_end += time_slice;
            }
        }
    }
}
