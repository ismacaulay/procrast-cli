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

    // TODO: We should store the credentials in the credentials storage for the os (ie. Keychain)
    // and then reuse them as necessary

    // prompt for email
    print!("Email: ");
    let email = input::get_stdin_input();

    // prompt for password
    let pass = rpassword::read_password_from_tty(Some("Password: ")).unwrap();

    let request = LoginRequest {
        email: email,
        password: pass,
    };

    let url = format!("{}/auth/v1/login", ctx.config.base_url);
    let resp: TokenResponse = network::send_post_request(&ctx.client, &url, &request, None)?;
    ctx.config.token = resp.token;
    config::save(&ctx.config)?;

    Ok(())
}
