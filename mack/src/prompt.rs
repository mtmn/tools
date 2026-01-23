use lofty::tag::Accessor;
use std::io::{self, Write};

use crate::track::Track;

fn prompt(prompt_text: &str) -> String {
    print!("{prompt_text}");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

pub fn prompt_tags(track: &Track) -> (Option<String>, Option<String>, Option<String>) {
    let current_artist = track.tag.artist().unwrap_or(std::borrow::Cow::Borrowed(""));
    let new_artist_input = prompt(&format!("Artist [{current_artist}]: "));
    let new_artist = if new_artist_input.is_empty() {
        None
    } else {
        Some(new_artist_input)
    };

    let current_album = track.tag.album().unwrap_or(std::borrow::Cow::Borrowed(""));
    let new_album_input = prompt(&format!("Album [{current_album}]: "));
    let new_album = if new_album_input.is_empty() {
        None
    } else {
        Some(new_album_input)
    };

    // Use common label detection logic
    let current_label =
        crate::util::get_label(&track.tag).unwrap_or(std::borrow::Cow::Borrowed(""));

    let new_label_input = prompt(&format!("Label [{current_label}]: "));
    let new_label = if new_label_input.is_empty() {
        None
    } else {
        Some(new_label_input)
    };

    (new_artist, new_album, new_label)
}
