#[macro_use]
extern crate dotenv_codegen;

use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand,
};

use dotenv::dotenv;
use futures::future::*;
use futures::{Future, Stream};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use reqwest::r#async::{Client, Decoder};
use reqwest::StatusCode;
use select::{document::Document, predicate::Name};
use std::mem;

fn main() {
    pretty_env_logger::init();
    dotenv().ok();

    let matches = build_app().get_matches();

    match matches.subcommand() {
        (CMD_LOGIN, Some(_matches)) => {
            tokio::run(lazy(|| {
                let client = Client::builder().cookie_store(true).build().unwrap();
                login(&client)
            }));
        }
        (CMD_PARSE_SUBJECT, Some(matches)) => {
            let url = matches.value_of(ARG_PARSE_SUBJECT_URL).unwrap();
            let format = matches.value_of(ARG_PARSE_FORMAT).unwrap();
            println!("URL: {}", url);
            println!("FORMAT: {}", format);
        }
        _ => {
            matches.usage(); // but unreachable
        }
    }
}

fn scrap_course(client: &Client, url: &str) -> impl Future<Item = (), Error = ()> {
    client
        .get(url)
        .send()
        .and_then(|mut res| {
            assert_eq!(res.status(), StatusCode::OK);

            println!("Status: {}", res.status());
            println!("Headers:\n{:?}", res.headers());

            let body = mem::replace(res.body_mut(), Decoder::empty());
            body.concat2()
        })
        .map_err(|err| println!("Request error: {}", err))
        .map(|mut body| {
            Document::from_read(body.as_ref())
                .unwrap();
        })
}

fn login(client: &Client) -> impl Future<Item = (), Error = ()> {
    let user = dotenv!("CANVAS_USER");
    let password = dotenv!("CANVAS_PASSWORD");
    let authenticity_token = dotenv!("CANVAS_AUTHENTICITY_TOKEN");
    let csrf_token = dotenv!("CSRF_TOKEN");

    println!("Log in with: {}", user);

    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, HeaderValue::from_static(csrf_token));

    let mut data: FormData = Vec::new();
    data.push(("authenticity_token", authenticity_token));
    data.push(("pseudonym_session[unique_id]", user));
    data.push(("pseudonym_session[password]", password));
    data.push(("pseudonym_session[remember_me]", "1"));

    client
        .post("https://micampus.unir.net/login/canvas")
        .headers(headers)
        .form(&data)
        .send()
        .and_then(|mut res| {
            assert_eq!(res.status(), StatusCode::OK);

            println!("Status: {}", res.status());
            println!("Headers:\n{:?}", res.headers());

            let body = mem::replace(res.body_mut(), Decoder::empty());
            body.concat2()
        })
        .map_err(|err| println!("Request error: {}", err))
        .map(|body| {
            println!("{:?}", body);
        })
}

const CMD_LOGIN: &str = "login";

const CMD_PARSE_SUBJECT: &str = "parse-subject";
const ARG_PARSE_FORMAT: &str = "format";
const ARG_PARSE_SUBJECT_URL: &str = "url";

fn build_app<'a>() -> App<'a, 'a> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name(CMD_LOGIN).about("Login to Canvas"))
        .subcommand(
            SubCommand::with_name(CMD_PARSE_SUBJECT)
                .about("Parse course's subject")
                .arg(
                    Arg::with_name(ARG_PARSE_SUBJECT_URL)
                        .help("Url where subject lives")
                        .long(ARG_PARSE_SUBJECT_URL)
                        .short("u")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name(ARG_PARSE_FORMAT)
                        .help("Format to parse it to")
                        .long(ARG_PARSE_FORMAT)
                        .short("f")
                        .default_value("markdown")
                        .required(false)
                        .takes_value(true),
                ),
        )
}

type FormData = Vec<(&'static str, &'static str)>;
