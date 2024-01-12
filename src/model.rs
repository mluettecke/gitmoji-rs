use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use url::Url;

/// The default URL used for update
pub const DEFAULT_URL: &str = "https://gitmoji.dev/api/gitmojis";
/// The default url for conventional commits data
pub const CONVENTIONAL_EMOJI_COMMITS_DEFAULT_URL: &str = "https://gist.githubusercontent.com/mluettecke/3f84a6a5c1c53ff6412828e601cd60ca/raw/9390c31c733ded384f65d37e063e041075016cdc/conventional-emoji-commits-types.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// The commit specification
pub enum CommitSpecification {
    /// The default gitmoji format
    Default,
    /// Conventional Emoji Format
    ConventionalEmojiCommits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// The emoji format
pub enum EmojiFormat {
    /// Use the code mode, like ':smile:'
    UseCode,
    /// Use the emoji mode, like '😄'
    UseEmoji,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
/// The Gitmojis configuration
pub struct GitmojiConfig {
    auto_add: bool,
    specification: CommitSpecification,
    format: EmojiFormat,
    signed: bool,
    scope: bool,
    update_url: Url,
    #[serde(with = "time::serde::iso8601::option")]
    last_update: Option<OffsetDateTime>,
    gitmojis: Vec<Gitmoji>,
    conventional_commit_emojis: Vec<ConventionalEmojiCommit>,
}

impl GitmojiConfig {
    /// Create a new `GitmojiConfig`
    #[must_use]
    pub const fn new(
        auto_add: bool,
        specification: CommitSpecification,
        format: EmojiFormat,
        signed: bool,
        scope: bool,
        update_url: Url,
    ) -> Self {
        Self {
            auto_add,
            specification,
            format,
            signed,
            scope,
            update_url,
            last_update: None,
            gitmojis: vec![],
            conventional_commit_emojis: vec![],
        }
    }

    /// Merge with a local configuration
    pub fn merge(&mut self, local_config: &LocalGitmojiConfig) {
        if let Some(auto_add) = local_config.auto_add() {
            self.auto_add = auto_add;
        }
        if let Some(format) = local_config.format() {
            self.format = format;
        }
        if let Some(signed) = local_config.signed() {
            self.signed = signed;
        }
        if let Some(gitmojis) = local_config.gitmojis() {
            self.gitmojis = gitmojis.to_vec();
        }
        if let Some(conventional_commit_emoji) = local_config.conventional_commit_emojis() {
            self.conventional_commit_emojis = conventional_commit_emoji.to_vec();
        }
    }

    /// If the "--all" is added to commit command
    #[must_use]
    pub const fn auto_add(&self) -> bool {
        self.auto_add
    }

    /// The commit specification
    #[must_use]
    pub const fn specification(&self) -> CommitSpecification {
        self.specification
    }

    /// The format of gitmoji (code or emoji)
    #[must_use]
    pub const fn format(&self) -> &EmojiFormat {
        &self.format
    }

    /// If the signed commits is enabled
    #[must_use]
    pub const fn signed(&self) -> bool {
        self.signed
    }

    /// If we add a scope
    #[must_use]
    pub const fn scope(&self) -> bool {
        self.scope
    }

    /// The URL used for update
    #[must_use]
    pub fn update_url(&self) -> &str {
        self.update_url.as_ref()
    }

    /// Set the URL used for update
    pub fn set_update_url(&mut self, update_url: Url) {
        self.update_url = update_url;
    }

    /// The last time the gitmoji list was updated
    #[must_use]
    pub const fn last_update(&self) -> Option<OffsetDateTime> {
        self.last_update
    }

    /// The gitmoji list
    #[must_use]
    pub fn gitmojis(&self) -> &[Gitmoji] {
        self.gitmojis.as_ref()
    }

    /// Set the gitmojis list
    pub fn set_gitmojis(&mut self, gitmojis: Vec<Gitmoji>) {
        self.last_update = Some(OffsetDateTime::now_utc());
        self.gitmojis = gitmojis;
    }

    /// The gitmoji list
    #[must_use]
    pub fn conventional_commit_emojis(&self) -> &[ConventionalEmojiCommit] {
        self.conventional_commit_emojis.as_ref()
    }

    /// Set conventional commit emojis
    pub fn set_conventional_commit_emojis(
        &mut self,
        conventional_commit_emojis: HashMap<String, ConventionalEmojiCommit>,
    ) {
        self.last_update = Some(OffsetDateTime::now_utc());
        self.conventional_commit_emojis = conventional_commit_emojis
            .into_iter()
            .map(|(r#type, value)| ConventionalEmojiCommit {
                r#type: r#type,
                code: value.code,
                description: value.description,
                emoji: value.emoji,
            })
            .collect();
    }
}

impl Default for GitmojiConfig {
    fn default() -> Self {
        Self {
            auto_add: false,
            specification: CommitSpecification::Default,
            format: EmojiFormat::UseCode,
            signed: false,
            scope: false,
            update_url: DEFAULT_URL.parse().expect("It's a valid URL"),
            last_update: None,
            gitmojis: vec![],
            conventional_commit_emojis: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// The local gitmoji configuration
pub struct LocalGitmojiConfig {
    auto_add: Option<bool>,
    specification: Option<CommitSpecification>,
    format: Option<EmojiFormat>,
    signed: Option<bool>,
    scope: Option<bool>,
    gitmojis: Option<Vec<Gitmoji>>,
    conventional_commit_emojis: Option<Vec<ConventionalEmojiCommit>>,
}

impl LocalGitmojiConfig {
    /// If the "--all" is added to commit command
    #[must_use]
    pub fn auto_add(&self) -> Option<bool> {
        self.auto_add
    }

    /// The specification of gitmoji
    #[must_use]
    pub fn specification(&self) -> Option<CommitSpecification> {
        self.specification
    }

    /// The format of gitmoji (code or emoji)
    #[must_use]
    pub fn format(&self) -> Option<EmojiFormat> {
        self.format
    }

    /// If the signed commits is enabled
    #[must_use]
    pub fn signed(&self) -> Option<bool> {
        self.signed
    }

    /// If we add a scope
    #[must_use]
    pub fn scope(&self) -> Option<bool> {
        self.scope
    }

    /// The gitmoji list
    #[must_use]
    pub fn gitmojis(&self) -> Option<&[Gitmoji]> {
        self.gitmojis.as_deref()
    }

    /// The conventional emoji commits list
    #[must_use]
    pub fn conventional_commit_emojis(&self) -> Option<&[ConventionalEmojiCommit]> {
        self.conventional_commit_emojis.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// A Gitmoji
pub struct Gitmoji {
    emoji: String,
    code: String,
    name: Option<String>,
    description: Option<String>,
}

impl Gitmoji {
    /// Create a gitmoji
    #[must_use]
    pub fn new(
        emoji: String,
        code: String,
        name: Option<String>,
        description: Option<String>,
    ) -> Self {
        Self {
            emoji,
            code,
            name,
            description,
        }
    }

    /// The emoji
    #[must_use]
    pub fn emoji(&self) -> &str {
        self.emoji.as_ref()
    }

    /// The associated code
    #[must_use]
    pub fn code(&self) -> &str {
        self.code.as_ref()
    }

    /// The name
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// The description
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

impl Display for Gitmoji {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Gitmoji {
            emoji,
            code,
            name,
            description,
            ..
        } = self;
        write!(
            f,
            "{emoji} {code} {} - {}",
            name.as_deref().unwrap_or_default(),
            description.as_deref().unwrap_or_default()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// A Conventional Commit Emoji
pub struct ConventionalEmojiCommit {
    emoji: String,
    code: String,
    r#type: String,
    description: Option<String>,
}

impl ConventionalEmojiCommit {
    /// Create a conventional commit emoji
    #[must_use]
    pub fn new(emoji: String, code: String, r#type: String, description: Option<String>) -> Self {
        Self {
            emoji,
            code,
            r#type,
            description,
        }
    }

    /// The emoji
    #[must_use]
    pub fn emoji(&self) -> &str {
        self.emoji.as_ref()
    }

    /// The associated code
    #[must_use]
    pub fn code(&self) -> &str {
        self.code.as_ref()
    }

    /// The type
    #[must_use]
    pub fn r#type(&self) -> &str {
        self.r#type.as_str()
    }

    /// The description
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

impl Display for ConventionalEmojiCommit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ConventionalEmojiCommit {
            emoji,
            r#type,
            description,
            ..
        } = self;
        write!(
            f,
            "{emoji} {} - {}",
            r#type.as_str(),
            description.as_deref().unwrap_or_default()
        )
    }
}

#[cfg(test)]
#[allow(clippy::ignored_unit_patterns)]
mod tests {
    use assert2::*;

    use super::*;

    #[test]
    fn should_serde_gitmoji() {
        let gitmoji = Gitmoji {
            emoji: String::from("🚀"),
            code: String::from("rocket"),
            name: Some(String::from("Initialize")),
            description: Some(String::from("Bla bla")),
        };

        // Serialize
        let toml = toml_edit::ser::to_string(&gitmoji);
        let_assert!(Ok(toml) = toml);

        // Deserialize
        let result = toml_edit::de::from_str::<Gitmoji>(&toml);
        let_assert!(Ok(result) = result);

        check!(result == gitmoji);
    }

    #[test]
    fn should_serde_config() {
        let mut config = GitmojiConfig::default();
        config.gitmojis.push(Gitmoji {
            emoji: String::from("🚀"),
            code: String::from("rocket"),
            name: Some(String::from("Initialize")),
            description: Some(String::from("Bla bla")),
        });

        // Serialize
        let toml = toml_edit::ser::to_string(&config);
        let_assert!(Ok(toml) = toml);

        // Deserialize
        let result = toml_edit::de::from_str::<GitmojiConfig>(&toml);
        let_assert!(Ok(result) = result);

        check!(result == config);
    }
}
