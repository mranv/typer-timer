mod score;

mod event;

use rdev;
use std::path::Path;

fn main() {
    env_logger::init();
    log::info!("Starting up");

    let mut current_interval = None;
    let mut current_count = 0;

    let mut activity_stream = event::Stream::new(Path::new("/tmp/repeto/events"));
    activity_stream.replay_since_midnight();

    let callback = move |event: rdev::Event| {
        // Filter out only KeyPress
        if !matches!(event.event_type, rdev::EventType::KeyPress(_)) {
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
                    println!("Interval {} had {} events", interval, current_count);
                    activity_stream.append(interval, current_count);

                    current_interval = Some(event_interval);
                    current_count = 1;
                }
            }
            None => {
                println!("Interval set to {}", event_interval);
                current_interval = Some(event_interval);
            }
        }
    };

    if let Err(error) = rdev::listen(callback) {
        println!("Error: {:?}", error)
    }
}
