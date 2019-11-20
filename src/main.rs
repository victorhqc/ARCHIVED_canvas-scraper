#[macro_use]
extern crate dotenv_codegen;

use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings, SubCommand,
};

use dotenv::dotenv;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};

fn main() {
    pretty_env_logger::init();
    dotenv().ok();

    let matches = build_app().get_matches();

    match matches.subcommand() {
        (CMD_LOGIN, Some(_matches)) => {
            let user = dotenv!("CANVAS_USER");
            let password = dotenv!("CANVAS_PASSWORD");
            let authenticity_token = dotenv!("CANVAS_AUTHENTICITY_TOKEN");
            let csrf_token = dotenv!("CSRF_TOKEN");

            println!("CSRF_TOKEN: {}", csrf_token);
            println!("Log in with: {}", user);

            let mut headers = HeaderMap::new();
            headers.insert(COOKIE, HeaderValue::from_static(csrf_token));

            let mut data: FormData = Vec::new();
            data.push(("authenticity_token", authenticity_token));
            data.push(("pseudonym_session[unique_id]", user));
            data.push(("pseudonym_session[password]", password));
            data.push(("pseudonym_session[remember_me]", "1"));

            let res = Client::new()
                .post("https://micampus.unir.net/login/canvas")
                .headers(headers)
                .form(&data)
                .send()
                .unwrap();

            println!("Status: {}", res.status());
            println!("Headers:\n{:?}", res.headers());
            // println!("Body:\n{:?}", res.text().unwrap());
        }
        _ => {
            matches.usage(); // but unreachable
        }
    }
}

const CMD_LOGIN: &str = "login";

fn build_app<'a>() -> App<'a, 'a> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name(CMD_LOGIN).about("Login to Canvas"))
}

type FormData = Vec<(&'static str, &'static str)>;
