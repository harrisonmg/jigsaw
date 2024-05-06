use std::{
    fs::{write, File},
    io::{BufRead, BufReader, Read},
    path::PathBuf,
};

use anyhow::Result;
use log::{info, warn};

use game::Puzzle;

pub struct PuzzleLoader {
    queue: ImageQueue,
    current_entry: Option<ImageQueueEntry>,
}

impl PuzzleLoader {
    pub fn new(queue_file: PathBuf) -> Self {
        Self {
            queue: ImageQueue::new(queue_file),
            current_entry: None,
        }
    }

    fn load_puzzle(entry: &ImageQueueEntry) -> Result<Puzzle> {
        info!("Loading {entry:?}");
        let mut file = File::open(&entry.image_path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Puzzle::new(buf.into(), entry.target_piece_count, true)
    }

    pub fn pop_current(&mut self) {
        if let Some(entry) = self.current_entry.take() {
            self.queue.comment_out_entry(&entry)
        }
    }
}

impl Iterator for PuzzleLoader {
    type Item = Puzzle;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop_current();

        while let Some(entry) = self.queue.top_entry() {
            match Self::load_puzzle(&entry) {
                Ok(puzzle) => {
                    self.current_entry = Some(entry);
                    return Some(puzzle);
                }
                Err(e) => {
                    warn!("Error loading entry {entry:?}: {e}");
                    self.queue.comment_out_entry(&entry)
                }
            }
        }
        info!("Reached end of image queue");
        None
    }
}

#[derive(PartialEq, Eq, Debug)]
struct ImageQueueEntry {
    pub target_piece_count: u32,
    pub image_path: PathBuf,
}

impl ImageQueueEntry {
    pub fn from_line(line: &str, quiet: bool) -> Option<Self> {
        let mut split = line.split_whitespace();

        let first = split.next()?;

        if first.starts_with('#') {
            return None;
        }

        let target_piece_count = match first.parse() {
            Ok(target) => target,
            Err(e) => {
                if !quiet {
                    warn!("Error parsing target piece count: {e}");
                }
                return None;
            }
        };

        let image_path = match split.next() {
            Some(string) => string,
            None => {
                warn!("Image queue entry missing image path");
                return None;
            }
        };

        Some(Self {
            target_piece_count,
            image_path: image_path.into(),
        })
    }
}

struct ImageQueue {
    queue_file: PathBuf,
}

impl ImageQueue {
    pub fn new(queue_file: PathBuf) -> Self {
        Self { queue_file }
    }

    pub fn top_entry(&self) -> Option<ImageQueueEntry> {
        let file = File::open(&self.queue_file).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            match line {
                Ok(line) => match ImageQueueEntry::from_line(line.as_str(), false) {
                    Some(entry) => return Some(entry),
                    None => continue,
                },
                Err(e) => warn!("Error reading line in queue file: {e}"),
            }
        }

        None
    }

    pub fn comment_out_entry(&self, entry: &ImageQueueEntry) {
        let file = File::open(&self.queue_file).unwrap();
        let reader = BufReader::new(file);

        let mut new_contents = String::new();
        let mut entry_found = false;

        for line in reader.lines().map_while(Result::ok) {
            if !entry_found {
                if let Some(line_entry) = ImageQueueEntry::from_line(line.as_str(), false) {
                    if line_entry == *entry {
                        entry_found = true;
                        new_contents.push_str("# ");
                    }
                }
            }
            new_contents.push_str(line.as_str());
            new_contents.push('\n');
        }

        write(&self.queue_file, new_contents).unwrap();
    }
}
