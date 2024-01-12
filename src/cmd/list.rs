use console::{Emoji, Style};

use crate::{ConventionalEmojiCommit, Gitmoji};

pub(super) fn print_gitmojis(gitmojis: &[Gitmoji]) {
    let blue = Style::new().blue();
    for gitmoji in gitmojis {
        let emoji = gitmoji.emoji();
        let code = gitmoji.code();
        let description = gitmoji.description().unwrap_or_default();
        println!("{emoji}{}\t{description}", blue.apply_to(code));
    }
}

pub(super) fn print_conventional_commit_emojis(
    conventional_commit_emojis: &[ConventionalEmojiCommit],
) {
    let blue = Style::new().blue();
    let max_width = conventional_commit_emojis
        .into_iter()
        .map(|conventional_commit_emoji| conventional_commit_emoji.r#type().len())
        .max()
        .unwrap_or(25);
    for conventional_commit_emoji in conventional_commit_emojis {
        let emoji = Emoji(conventional_commit_emoji.emoji(), "");
        let type_name = conventional_commit_emoji.r#type();
        let description = conventional_commit_emoji.description().unwrap();
        println!(
            "{emoji} {colored_type:<width$} {description}",
            colored_type = blue.apply_to(type_name),
            width = max_width + 2
        );
    }
}
