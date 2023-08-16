const INTERVAL_LEN_SECS: u32 = 5;

const LACK_OF_RECOVERY_INTERVAL_PENALTY: f32 = 0.1 / (60.0 * 60.0 / INTERVAL_LEN_SECS as f32); // 0.1 per hour

fn duration_multiplier(intervals: u32) -> f32 {
    if intervals <= 180 * 60 / INTERVAL_LEN_SECS {
        2.0
    } else if intervals <= 240 * 60 / INTERVAL_LEN_SECS {
        1.5
    } else if intervals <= 480 * 60 / INTERVAL_LEN_SECS {
        1.0
    } else {
        // intervals <= 480 * 60 / INTERVAL_LEN_SECS
        0.5
    }
}

pub struct Score {
    // A tuple containing timestamp / 5, and the number of key presses
    series: Vec<(u32, u8)>,
}

impl Score {
    pub fn new() -> Score {
        Score { series: Vec::new() }
    }

    pub fn insert(&mut self, timestamp: u32, keypresses: u8) {
        self.series.push((timestamp, keypresses));
    }

    pub fn calculate(&self) -> f32 {
        let mut previous_timestamp = 0;
        let mut sum_keypresses = 0;
        let mut total_work_intervals: u32 = 0;
        let mut recovery_debt: u32 = 0;
        let mut num_lack_of_recovery_intervals = 0;

        // Iterate over the series
        for (timestamp, keypresses) in self.series.iter() {
            // Add the keypresses to the sum
            sum_keypresses += *keypresses as u32;

            let missing_intervals = *timestamp - previous_timestamp - 1;
            if missing_intervals >= 3 {
                recovery_debt -= 6 * missing_intervals;
            }

            assert!(*keypresses != 0);
            total_work_intervals += 1;
            recovery_debt += 1;

            if recovery_debt >= 60 * 60 / INTERVAL_LEN_SECS {
                // 1 hour
                num_lack_of_recovery_intervals += 1;
            }
            previous_timestamp = *timestamp;
        }

        let lack_of_recovery_factor =
            1.0 - num_lack_of_recovery_intervals as f32 * LACK_OF_RECOVERY_INTERVAL_PENALTY;
        // sum_keypresses / (30 * FORCE_FACTOR * POSTURE_FACTOR * ADDITIONAL_FACTORS * REPETITIVENESS_MULTIPLIER * (duration_interval_mins * lack_of_recovery_factor * duration_multiplier(duration_interval_mins)))
        sum_keypresses as f32
            / (total_work_intervals as f32
                * lack_of_recovery_factor
                * duration_multiplier(total_work_intervals))
    }
}
