use atuin_client::{api_client, settings::Settings};
use eyre::{Result, bail};
use std::io;

fn get_input() -> Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim_end_matches(&['\r', '\n'][..]).to_string())
}

pub async fn run(settings: &Settings) -> Result<()> {
    if !settings.logged_in().await? {
        bail!("You are not logged in");
    }

    eprint!(
        "Please enter 'DELETE-ACCOUNT-AND-DATA' (uppercase and without quotes) to delete your account: "
    );
    let confirmation = get_input().expect("Failed to read from input");

    if confirmation != "DELETE-ACCOUNT-AND-DATA" {
        println!("\nConfirmation failure. Account not deleted.");
        std::process::exit(1);
    }

    let client = api_client::Client::new(
        &settings.sync_address,
        settings.session_token().await?.as_str(),
        settings.network_connect_timeout,
        settings.network_timeout,
    )?;

    client.delete().await?;

    // Clean up session from meta store
    Settings::meta_store().await?.delete_session().await?;

    println!("Your account is deleted");

    Ok(())
}
