use crate::{config, input, network, utils::Result, Context};
use rpassword;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct TokenResponse {
    token: String,
}

#[derive(Debug, Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

pub fn login(ctx: &mut Context) -> Result<()> {
    if ctx.config.base_url == "" {
        return Err(String::from("No base_url configured"));
    }

    let base_url = format!("{}/auth/v1", ctx.config.base_url);

    let mut url = format!("{}/token", base_url);
    // attempt to refresh token
    if let Ok(resp) =
        network::send_get_request::<TokenResponse>(&ctx.client, &url, Some(&ctx.config.token))
    {
        ctx.config.token = resp.token;
        config::save(&ctx.config)?;
    } else {
        // prompt for email
        print!("Email: ");
        let email = input::get_stdin_input();

        // prompt for password
        let pass = rpassword::read_password_from_tty(Some("Password: ")).unwrap();

        let request = LoginRequest {
            email: email,
            password: pass,
        };

        url = format!("{}/login", base_url);
        let resp: TokenResponse = network::send_post_request(&ctx.client, &url, &request, None)?;
        ctx.config.token = resp.token;
        config::save(&ctx.config)?;
    }

    Ok(())
}
