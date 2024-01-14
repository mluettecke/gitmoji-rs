use std::process::exit;

use console::Term;
use tracing::{info, warn};
use url::Url;

use crate::git::has_staged_changes;
use crate::{
    git, CommitSpecification, EmojiFormat, Error, GitmojiConfig, Result, EXIT_CANNOT_UPDATE,
    EXIT_NO_CONFIG,
};

mod commit;
mod config;
#[cfg(feature = "hook")]
mod hook;
mod list;
mod search;
mod update;

pub use self::commit::*;
pub use self::config::*;
#[cfg(feature = "hook")]
pub use self::hook::*;
use self::list::{print_conventional_commit_emojis, print_gitmojis};
use self::search::filter;
use self::update::{update_conventional_emoji_commits, update_gitmojis};

async fn get_config_or_stop() -> GitmojiConfig {
    match read_config_or_fail().await {
        Ok(config) => config,
        Err(err) => {
            warn!("Oops, cannot read config because {err}");
            eprintln!("⚠️  No configuration found, try run `gitmoji init` to fetch a configuration");
            exit(EXIT_NO_CONFIG)
        }
    }
}

async fn update_config_or_stop(config: GitmojiConfig) -> GitmojiConfig {
    let url = config.update_url().to_string();
    match config.specification() {
        CommitSpecification::Default => match update_gitmojis(config).await {
            Ok(config) => config,
            Err(err) => {
                warn!("Oops, cannot update the config because {err}");
                eprintln!("⚠️  Configuration not updated, maybe check the update url '{url}'");
                exit(EXIT_CANNOT_UPDATE)
            }
        },
        CommitSpecification::ConventionalEmojiCommits => {
            match update_conventional_emoji_commits(config).await {
                Ok(config) => config,
                Err(err) => {
                    println!("{:?}", err);
                    warn!("Oops, cannot update the config because {err}");
                    eprintln!("⚠️  Configuration not updated, maybe check the update url '{url}'");
                    exit(EXIT_CANNOT_UPDATE)
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct CommitTitleDescription {
    title: String,
    description: Option<String>,
}

#[tracing::instrument(skip(term))]
async fn ask_commit_title_description(
    config: &GitmojiConfig,
    term: &Term,
) -> Result<CommitTitleDescription> {
    match config.specification() {
        CommitSpecification::Default => {
            let DefaultCommitParams {
                gitmoji,
                scope,
                title,
                description,
            } = get_default_commit_params(config, term)?;
            let gitmoji = match config.format() {
                EmojiFormat::UseCode => gitmoji.code(),
                EmojiFormat::UseEmoji => gitmoji.emoji(),
            };

            let title = match config.specification() {
                CommitSpecification::Default => scope.map_or_else(
                    || format!("{gitmoji} {title}"),
                    |scope| format!("{gitmoji} {scope}{title}"),
                ),
                CommitSpecification::ConventionalEmojiCommits => scope.map_or_else(
                    || format!("{gitmoji} {title}"),
                    |scope| format!("{gitmoji} {scope}{title}"),
                ),
            };

            let result = CommitTitleDescription { title, description };
            Ok(result)
        }
        CommitSpecification::ConventionalEmojiCommits => {
            let ConventionalEmojiCommitParams {
                description,
                emoji,
                scope,
                title,
                type_name,
            } = get_conventional_emoji_commit_params(config, term)?;
            let emoji = match config.format() {
                EmojiFormat::UseCode => emoji.code(),
                EmojiFormat::UseEmoji => emoji.emoji(),
            };
            let title = match scope {
                Some(value) if !value.is_empty() => {
                    format!("{emoji}{type_name}({value}): {title}")
                }
                Some(_) | None => {
                    format!("{emoji}{type_name}: {title}")
                }
            };
            let result = CommitTitleDescription { title, description };

            Ok(result)
        }
    }
}

/// Commit using Gitmoji
#[tracing::instrument(skip(term))]
pub async fn commit(all: bool, amend: bool, term: &Term) -> Result<()> {
    let config = get_config_or_stop().await;

    if !amend && !has_staged_changes().await? {
        eprintln!("No change to commit");
        return Ok(());
    }

    let CommitTitleDescription { title, description } =
        ask_commit_title_description(&config, term).await?;

    // Add before commit
    let all = all || config.auto_add();

    // Commit
    let status = git::commit(all, amend, config.signed(), &title, description.as_deref()).await?;
    status.success().then_some(()).ok_or(Error::FailToCommit)
}

/// Configure Gitmoji
#[tracing::instrument(skip(term))]
pub async fn config(default: bool, term: &Term) -> Result<()> {
    let config = if default {
        GitmojiConfig::default()
    } else {
        create_config(term)?
    };
    info!("Loading gitmojis from {}", config.update_url());
    update_config_or_stop(config).await;

    Ok(())
}

/// Search a gitmoji
#[tracing::instrument]
pub async fn search(text: &str) -> Result<()> {
    let config = get_config_or_stop().await;
    let result = filter(config.gitmojis(), text);
    print_gitmojis(&result);
    Ok(())
}

/// List all Gitmojis
#[tracing::instrument]
pub async fn list() -> Result<()> {
    let config = get_config_or_stop().await;
    match config.specification() {
        CommitSpecification::ConventionalEmojiCommits => {
            print_conventional_commit_emojis(config.conventional_commit_emojis());
        }
        CommitSpecification::Default => {
            print_gitmojis(config.gitmojis());
        }
    }
    Ok(())
}

/// Update the configuration with the URL
#[tracing::instrument]
pub async fn update_config(url: Option<Url>) -> Result<()> {
    let mut config = read_config_or_default().await;
    if let Some(url) = url {
        config.set_update_url(url);
    }
    let result = update_config_or_stop(config).await;
    print_gitmojis(result.gitmojis());

    Ok(())
}

/// Create hook
#[cfg(feature = "hook")]
#[tracing::instrument]
pub async fn create_hook() -> Result<()> {
    hook::create().await
}

/// Remove hook
#[tracing::instrument]
#[cfg(feature = "hook")]
pub async fn remove_hook() -> Result<()> {
    hook::remove().await
}

/// Apply hook
#[cfg(feature = "hook")]
#[tracing::instrument(skip(term))]
pub async fn apply_hook(
    dest: std::path::PathBuf,
    source: Option<String>,
    term: &Term,
) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

    let config = get_config_or_stop().await;

    let CommitTitleDescription { title, description } =
        ask_commit_title_description(&config, term).await?;

    info!("Write commit message to {dest:?} with source: {source:?}");
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(dest)
        .await?;

    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;
    file.seek(std::io::SeekFrom::Start(0)).await?;

    file.write_all(title.as_bytes()).await?;
    file.write_all(b"\n\n").await?;

    if let Some(description) = description {
        file.write_all(description.as_bytes()).await?;
        file.write_all(b"\n").await?;
    }
    file.write_all(&contents).await?;

    Ok(())
}
