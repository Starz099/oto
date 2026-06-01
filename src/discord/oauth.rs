use crate::core::DiscordError;

pub const DISCORD_TOKEN_URL: &str = "https://discord.com/api/v10/oauth2/token";

pub async fn exchange_code(
    client_id: &str,
    client_secret: &str,
    code: &str,
) -> Result<String, DiscordError> {
    let client = reqwest::Client::new();
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", "http://127.0.0.1"),
    ];

    let res = client.post(DISCORD_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| DiscordError::Auth(e.to_string()))?;

    let status = res.status();
    let token_data: serde_json::Value = res.json().await
        .map_err(|e| DiscordError::Auth(e.to_string()))?;

    if !status.is_success() {
        return Err(DiscordError::Auth(format!("Token API error {}: {}", status, token_data)));
    }

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| DiscordError::Auth("No access_token found".to_string()))?
        .to_string();

    Ok(access_token)
}
