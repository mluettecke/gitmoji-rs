use std::collections::HashMap;

use serde::Deserialize;
use tracing::{debug, info};

use super::write_config;
use crate::model::GitmojiConfig;
use crate::{ConventionalEmojiCommit, Gitmoji, Result};

#[derive(Debug, Clone, Default, Deserialize)]
struct GetGitmojis {
    gitmojis: Vec<Gitmoji>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct GetConventionalEmojiCommitsTypes {
    types: HashMap<String, ConventionalEmojiCommit>,
}

async fn get_gitmojis(url: &str) -> Result<GetGitmojis> {
    info!("Update gitmojis with {url}");
    let result = reqwest::get(url).await?.json::<GetGitmojis>().await?;
    debug!("Found {} gitmojis", result.gitmojis.len());
    Ok(result)
}

async fn get_conventional_commit_emojis(url: &str) -> Result<GetConventionalEmojiCommitsTypes> {
    info!("Update conventional emoji commits with {url}");
    let result = reqwest::get(url)
        .await?
        .json::<GetConventionalEmojiCommitsTypes>()
        .await?;
    debug!("Found {} conventional emoji commits", result.types.len());
    Ok(result)
}

pub async fn update_gitmojis(mut config: GitmojiConfig) -> Result<GitmojiConfig> {
    let result = get_gitmojis(config.update_url()).await?;
    config.set_gitmojis(result.gitmojis);
    write_config(&config).await?;

    Ok(config)
}

pub async fn update_conventional_emoji_commits(mut config: GitmojiConfig) -> Result<GitmojiConfig> {
    let result = get_conventional_commit_emojis(config.update_url()).await?;
    config.set_conventional_commit_emojis(result.types);
    write_config(&config).await?;

    Ok(config)
}

#[cfg(test)]
#[allow(clippy::ignored_unit_patterns)]
mod tests {
    use assert2::{check, let_assert};
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    #[test_log::test(tokio::test)]
    async fn should_get_gitmojis() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{
        "gitmojis": [
            {
                "emoji": "🎨",
                "entity": "&#x1f3a8;",
                "code": ":art:",
                "description": "Improve structure / format of the code.",
                "name": "art",
                "semver": null
            }
        ]
    }"#,
                "application/json",
            ))
            .mount(&mock_server)
            .await;

        let result = get_gitmojis(&mock_server.uri()).await.unwrap();

        check!(result.gitmojis.len() == 1);
    }

    #[test_log::test(tokio::test)]
    async fn should_fail_on_bad_json() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{
    "💥 plaf!": []
}"#,
                "application/json",
            ))
            .mount(&mock_server)
            .await;

        let result = get_gitmojis(&mock_server.uri()).await;

        let_assert!(Err(_) = result);
    }

    #[test_log::test(tokio::test)]
    async fn should_fail_on_404() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let result = get_gitmojis(&mock_server.uri()).await;

        let_assert!(Err(_) = result);
    }
}
