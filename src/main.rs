use github_stats_rs::{
    algebra::{GithubExt, ImageGenExt},
    service::{Configuration, Github, ImageGen, Telemetry},
};
use reqwest::Client;
use secrecy::ExposeSecret;
use tracing_subscriber::Registry;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    Telemetry::<Registry>::new("github_stats_rs".into(), "info".into(), std::io::stdout).init();

    let configuration = Configuration::load_or_die();
    tracing::info!("{configuration:#?}");
    let app = octocrab::OctocrabBuilder::new()
        .app(configuration.app_id().into(), configuration.app_key()?)
        .build()?;
    let (app, access_token) = app.installation_and_token(configuration.installation_id().into()).await?;

    let mut header_map = reqwest::header::HeaderMap::new();
    header_map.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(&format!(
        "Bearer {}",
        access_token.expose_secret()
    ))?);
    header_map.insert(reqwest::header::HeaderName::from_static("x-github-api-version"), reqwest::header::HeaderValue::from_static("2022-11-28"));

    let client = Client::builder()
        .user_agent("graphql-rust")
        .default_headers(header_map)
        .build()?;

    let github = Github::new(configuration.clone(), app, client);
    let stats = github.get_stats().await?;
    let lines_changed = stats.lines_changed();
    let total_contributions = stats.total_contributions();

    // Generate the images
    let image_gen = ImageGen::new(
        configuration.template_folder().to_string(),
        configuration.output_folder().to_string(),
    );
    image_gen.generate_overview(&stats)?;
    image_gen.generate_languages(&stats)?;
    // image_gen.generate_contributions_grid(&stats)?;

    tracing::info!("Total contributions: {}", total_contributions);
    tracing::info!("Lines changed: {}, {}", lines_changed.0, lines_changed.1);
    Ok(())
}
