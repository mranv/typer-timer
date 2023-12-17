mod score;

mod event;

use directories::ProjectDirs;
use rdev;
use std::{path::PathBuf, process::ExitCode};

use crate::event::Stream;

fn stream_logfile() -> Result<PathBuf, Box<dyn std::error::Error>> {
    match ProjectDirs::from("gr", "Hadar", "Typer-Timer") {
        Some(proj_dirs) => match proj_dirs.state_dir() {
            Some(dir) => {
                std::fs::create_dir_all(dir)?;
                Ok(dir.join("events.kb2"))
            }
            None => Err(Box::from("No state dir available")),
        },
        None => Err(Box::from("No project dir available")),
    }
}

fn main() -> ExitCode {
    env_logger::init();
    log::info!("Starting up");

    let mut current_interval = None;
    let mut current_count = 0;

    let mut activity_stream: Stream;
    match stream_logfile() {
        Ok(file) => {
            activity_stream = event::Stream::new(file);
        }
        Err(e) => {
            log::error!("File error: {}", e);
            return ExitCode::from(1);
        }
    }

    activity_stream.replay_since_midnight();

    let callback = move |event: rdev::Event| {
        if !matches!(event.event_type, rdev::EventType::KeyPress(_)) {
            // Ignore mouse movements and other
            return;
        }

        let event_interval = (event
            .time
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            / 5) as u32;

        match current_interval {
            Some(interval) => {
                if interval == event_interval {
                    current_count += 1;
                } else {
                    log::debug!("Interval {} had {} events", interval, current_count);
                    activity_stream.append(interval, current_count);

                    current_interval = Some(event_interval);
                    current_count = 1;
                }
            }
            None => {
                log::debug!("Start interval {}", event_interval);
                current_interval = Some(event_interval);
            }
        }
    };

    if let Err(e) = rdev::listen(callback) {
        log::error!("Error listening to keyboard: {:?}", e);
        return ExitCode::from(1);
    }
    ExitCode::from(1)
}
