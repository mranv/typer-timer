mod score;

use std::io::Read;

fn main() {
    let mut score = score::Score::new();
    let mut buffer = [0; 5];
    while let Ok(_) = std::io::stdin().read_exact(&mut buffer) {
        let timestamp = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        let keypresses = buffer[4];
        score.insert(timestamp, keypresses);
        println!("{:9.0}, {:3}", score.calculate(), keypresses);
    }
}
