use chrono::{self, TimeZone};
use std::collections::VecDeque;

const SLICE_SIZE: u32 = 5;
const REST_TO_WORK_RATIO: u32 = 4;

const LACK_OF_RECOVERY_SLICE_PENALTY: f32 = 0.1 / (60.0 * 60.0 / SLICE_SIZE as f32); // 0.1 per hour

fn duration_multiplier(slices: u32) -> f32 {
    if slices <= 180 * 60 / SLICE_SIZE {
        2.0
    } else if slices <= 240 * 60 / SLICE_SIZE {
        1.5
    } else if slices <= 480 * 60 / SLICE_SIZE {
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
    day_start_time_slice: u32,
    previous_time_slice: u32,
    sum_keypresses: u32,
    micro_pause_slices: u32,
    work_slices: u32,
    recovery_debt: u32,
    num_lack_of_recovery_slices: u32,
    last_recovery_slice: u32,
    // Tuple of (number of slices, True: work, False: rest)
    work_ranges: Vec<u32>,
    // Queue of (time_slice, micro_pause_slices, work_slices) tuple. We consume the queue from the front, so that only the last 60 minutes are kept.
    last_hour_slices: VecDeque<(u32, u32, u32)>,
}

impl Score {
    pub fn new() -> Score {
        Score {
            day_start_time_slice: 0,
            previous_time_slice: 0,
            sum_keypresses: 0,
            micro_pause_slices: 0,
            work_slices: 0,
            recovery_debt: 0,
            num_lack_of_recovery_slices: 0,
            last_recovery_slice: 0,
            work_ranges: vec![(0)],
            last_hour_slices: VecDeque::new(),
        }
    }

    fn reset(&mut self) {
        self.day_start_time_slice = 0;
        self.previous_time_slice = 0;
        self.sum_keypresses = 0;
        self.micro_pause_slices = 0;
        self.work_slices = 0;
        self.recovery_debt = 0;
        self.num_lack_of_recovery_slices = 0;
        self.last_recovery_slice = 0;
        self.work_ranges = vec![(0)];
        self.last_hour_slices = VecDeque::new();
    }

    pub fn append(&mut self, time_slice: u32, keypresses: u8) {
        let mut hour_slice = (time_slice, 0, 0);
        if keypresses == 0 {
            println!(
                "Warning: keypresses == 0 at {}",
                chrono::Local
                    .timestamp_opt(time_slice as i64 * SLICE_SIZE as i64, 0)
                    .unwrap()
                    .format("%Y-%m-%d %H:%M")
            );
            return;
        }
        if !is_same_day(self.previous_time_slice, time_slice) {
            println!("New day");
            self.reset();
            self.last_recovery_slice = time_slice;
            self.previous_time_slice = time_slice;
            self.day_start_time_slice = time_slice;
        }
        self.sum_keypresses += keypresses as u32;

        let missing_slices = (time_slice - self.previous_time_slice).saturating_sub(1);
        if missing_slices < 3 {
            // Count as work because it's too short to be a micro pause
            self.work_slices += missing_slices;
        } else if missing_slices < 60 * 5 / SLICE_SIZE {
            self.micro_pause_slices += missing_slices;
            hour_slice.1 += missing_slices;
        } else {
            // Do "self.recovery_debt -= REST_TO_WORK_RATIO * missing_intervals;" but make sure self.recovery_debt doesn't go below 0
            self.recovery_debt = self
                .recovery_debt
                .saturating_sub(REST_TO_WORK_RATIO * missing_slices);
            self.last_recovery_slice = time_slice;
            self.work_ranges.push(missing_slices);
            self.work_ranges.push(0);
        }

        self.work_slices += 1;
        hour_slice.2 += 1;
        self.recovery_debt += 1;

        if self.recovery_debt >= 60 * 60 / SLICE_SIZE {
            // 1 hour
            self.num_lack_of_recovery_slices += 1;
        }
        self.previous_time_slice = time_slice;
        // Increment last work range
        *self.work_ranges.last_mut().unwrap() += 1;
        // Remove elements from the front of the queue until the first element is less than 60 minutes old
        while !self.last_hour_slices.is_empty()
            && self.last_hour_slices[0].0 + 60 * 60 / SLICE_SIZE < time_slice
        {
            self.last_hour_slices.pop_front().unwrap();
        }

        self.last_hour_slices.push_back(hour_slice);
    }

    pub fn current_score(&self) -> f32 {
        let total_work_slices = self.work_slices + self.micro_pause_slices;
        let lack_of_recovery_factor =
            1.0 - self.num_lack_of_recovery_slices as f32 * LACK_OF_RECOVERY_SLICE_PENALTY;
        // self.sum_keypresses / (30 * FORCE_FACTOR * POSTURE_FACTOR * ADDITIONAL_FACTORS * REPETITIVENESS_MULTIPLIER * (duration_interval_mins * lack_of_recovery_factor * duration_multiplier(duration_interval_mins)))
        self.sum_keypresses as f32
            / (total_work_slices as f32
                * lack_of_recovery_factor
                * duration_multiplier(total_work_slices))
            * 10.0
    }

    pub fn total_keypresses(&self) -> u32 {
        self.sum_keypresses
    }

    pub fn micro_pause_share(&self) -> f32 {
        1.0 - self.micro_pause_slices as f32 * SLICE_SIZE as f32 / self.total_work() as f32
    }

    pub fn micro_pause_share_past_hour(&self) -> f32 {
        let mut total_micro_pause_slices = 0;
        let mut total_work_slices = 0;
        for (_, micro_pause_slices, work_slices) in &self.last_hour_slices {
            total_micro_pause_slices += micro_pause_slices;
            total_work_slices += work_slices;
        }
        1.0 - total_micro_pause_slices as f32
            / (total_micro_pause_slices + total_work_slices) as f32
    }

    pub fn total_work(&self) -> u32 {
        let total_work_slices = self.work_slices + self.micro_pause_slices;
        total_work_slices * SLICE_SIZE
    }

    pub fn last_recovery_since(&self) -> u32 {
        (self.previous_time_slice - self.last_recovery_slice) * SLICE_SIZE
    }

    pub fn needed_recovery(&self) -> u32 {
        self.recovery_debt / REST_TO_WORK_RATIO * SLICE_SIZE
    }

    pub fn lack_of_recovery(&self) -> u32 {
        self.num_lack_of_recovery_slices * SLICE_SIZE
    }

    pub fn day_start(&self) -> u32 {
        self.day_start_time_slice * SLICE_SIZE
    }

    pub fn work_ranges(&self) -> Vec<u32> {
        let mut work_ranges = Vec::new();
        for duration in &self.work_ranges {
            work_ranges.push(*duration * SLICE_SIZE);
        }
        work_ranges
    }
}
