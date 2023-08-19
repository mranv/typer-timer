use chrono::{self, TimeZone};

const SLICE_SIZE: u32 = 5;
const REST_TO_WORK_RATIO: u32 = 4;

const LACK_OF_RECOVERY_INTERVAL_PENALTY: f32 = 0.1 / (60.0 * 60.0 / SLICE_SIZE as f32); // 0.1 per hour

fn duration_multiplier(intervals: u32) -> f32 {
    if intervals <= 180 * 60 / SLICE_SIZE {
        2.0
    } else if intervals <= 240 * 60 / SLICE_SIZE {
        1.5
    } else if intervals <= 480 * 60 / SLICE_SIZE {
        1.0
    } else {
        // intervals <= 480 * 60 / INTERVAL_LEN_SECS
        0.5
    }
}

/// Returns true if the two time slices are on the same day in the local timezone
fn is_same_day(time_slice1: u32, time_slice2: u32) -> bool {
    let time1 = chrono::Local
        .timestamp_opt(time_slice1 as i64 * SLICE_SIZE as i64, 0)
        .unwrap();
    let time2 = chrono::Local
        .timestamp_opt(time_slice2 as i64 * SLICE_SIZE as i64, 0)
        .unwrap();
    time1.date_naive() == time2.date_naive()
}

pub struct Score {
    previous_time_slice: u32,
    sum_keypresses: u32,
    total_work_intervals: u32,
    recovery_debt: u32,
    num_lack_of_recovery_cycles: u32,
}

impl Score {
    pub fn new() -> Score {
        Score {
            previous_time_slice: 0,
            sum_keypresses: 0,
            total_work_intervals: 0,
            recovery_debt: 0,
            num_lack_of_recovery_cycles: 0,
        }
    }

    fn reset(&mut self) {
        self.previous_time_slice = 0;
        self.sum_keypresses = 0;
        self.total_work_intervals = 0;
        self.recovery_debt = 0;
        self.num_lack_of_recovery_cycles = 0;
    }

    pub fn append(&mut self, time_slice: u32, keypresses: u8) {
        if !is_same_day(self.previous_time_slice, time_slice) {
            self.reset();
        }
        self.sum_keypresses += keypresses as u32;

        let missing_intervals = time_slice - self.previous_time_slice - 1;
        if missing_intervals >= 60 * 5 / SLICE_SIZE {
            // Do "self.recovery_debt -= REST_TO_WORK_RATIO * missing_intervals;" but make sure self.recovery_debt doesn't go below 0
            self.recovery_debt = self
                .recovery_debt
                .saturating_sub(REST_TO_WORK_RATIO * missing_intervals);
        } else {
            self.total_work_intervals += missing_intervals;
        }

        assert!(keypresses != 0);
        self.total_work_intervals += 1;
        self.recovery_debt += 1;

        if self.recovery_debt >= 60 * 60 / SLICE_SIZE {
            // 1 hour
            self.num_lack_of_recovery_cycles += 1;
        }
        self.previous_time_slice = time_slice;
    }

    pub fn current_score(&self) -> f32 {
        let lack_of_recovery_factor =
            1.0 - self.num_lack_of_recovery_cycles as f32 * LACK_OF_RECOVERY_INTERVAL_PENALTY;
        // self.sum_keypresses / (30 * FORCE_FACTOR * POSTURE_FACTOR * ADDITIONAL_FACTORS * REPETITIVENESS_MULTIPLIER * (duration_interval_mins * lack_of_recovery_factor * duration_multiplier(duration_interval_mins)))
        self.sum_keypresses as f32
            / (self.total_work_intervals as f32
                * lack_of_recovery_factor
                * duration_multiplier(self.total_work_intervals))
            * 10.0
    }

    pub fn total_keypresses(&self) -> u32 {
        self.sum_keypresses
    }

    pub fn total_work(&self) -> u32 {
        self.total_work_intervals * SLICE_SIZE
    }

    pub fn needed_recovery(&self) -> u32 {
        self.recovery_debt / REST_TO_WORK_RATIO * SLICE_SIZE
    }

    pub fn lack_of_recovery(&self) -> u32 {
        self.num_lack_of_recovery_cycles * SLICE_SIZE
    }
}
