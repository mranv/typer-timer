use crate::score;

use std::fs;
use std::io::{ErrorKind, Read, Seek, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use chrono::TimeZone;

fn duration_str(secs: u32) -> String {
    // gives string in hours and mins
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    format!("{:2}h {:2}m", hours, mins)
}

pub struct Stream {
    stream_file: std::fs::File,
    stream_valid_pos: u64,
    banner_file: std::fs::File,
    score: score::Score,
}

impl Stream {
    pub fn new(file_name: PathBuf) -> Stream {
        let banner_folder = "/tmp/repeto";
        if let Err(_) = fs::metadata(banner_folder) {
            let permissions = fs::Permissions::from_mode(0o700);
            fs::create_dir(banner_folder).unwrap();
            fs::set_permissions(banner_folder, permissions).unwrap();
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(file_name.as_path())
            .unwrap();

        log::info!("Opened {} for tracking activity", file_name.display());

        Stream {
            stream_file: file,
            stream_valid_pos: 0,
            banner_file: std::fs::File::create("/tmp/repeto/banner").unwrap(),
            score: score::Score::new(),
        }
    }

    pub fn replay_since_midnight(&mut self) {
        loop {
            let mut buffer = vec![0; 5];
            match self.stream_file.read_exact(&mut buffer) {
                Ok(()) => {
                    let timestamp =
                        u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
                    let keypresses = buffer[4];
                    self.score.append(timestamp, keypresses);

                    log::debug!(
                        "Replayed event at {} with {} keypresses",
                        chrono::Local
                            .timestamp_opt(timestamp as i64 * score::SLICE_SIZE as i64, 0)
                            .unwrap()
                            .format("%Y-%m-%d %H:%M:%S"),
                        keypresses
                    );

                    self.stream_valid_pos += 5;
                }
                Err(error) if error.kind() == ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(error) => {
                    log::error!("Error reading stream file: {:?}", error);
                }
            }
        }

        log::info!(
            "Replayed all events until file position {} (end of file)",
            self.stream_valid_pos
        );

        // Go back to the last valid position. New appends will overwrite any corrupted bytes
        self.stream_file
            .seek(std::io::SeekFrom::Start(self.stream_valid_pos))
            .unwrap();
    }

    pub fn append(&mut self, timestamp: u32, keypresses: u8) {
        let mut buffer = [0; 5];
        buffer[0..4].copy_from_slice(&timestamp.to_be_bytes());
        buffer[4] = keypresses;
        self.stream_file.write_all(&buffer).unwrap();

        self.score.append(timestamp, keypresses);

        self.banner_file.set_len(0).unwrap();
        self.banner_file.seek(std::io::SeekFrom::Start(0)).unwrap();
        write!(
            self.banner_file,
            "{} / {} 🔨",
            duration_str(self.score.last_recovery_since()),
            duration_str(self.score.total_work())
        )
        .unwrap();
    }
}
