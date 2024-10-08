use reqwest::{header::{self, HeaderMap, HeaderValue}, Response};
use tokio::time::{interval, Duration};
use std::fs::File;
use std::error::Error;
use std::collections::HashMap;

mod source_query;
use source_query::{SourceQuery, A2SInfoResult};

#[derive(Debug, serde::Deserialize, PartialEq)]
struct Config {
    influxdb: InfluxDBConfig,
    servers: Vec<ServerConfig>,
    frequency_secs: u64
}

#[derive(Debug, serde::Deserialize, PartialEq)]
struct InfluxDBConfig {
    host: String,
    bucket: String,
    token: String,
    organization: String
}

#[derive(Debug, serde::Deserialize, PartialEq)]
struct ServerConfig {
    host: String,
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_unknown")]
    community: String
}

#[tokio::main]
async fn main() {

    let config = match read_config() {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Unable to open config.yaml: {}", err);
            std::process::exit(1);
        }
    };

    let queries: Vec<SourceQuery> = config.servers.into_iter().map(
        |s|{SourceQuery::new(s.host, s.port, s.community)}
    ).collect();
    
    let mut interval = interval(Duration::from_secs(config.frequency_secs));

    let mut headers = HeaderMap::new();
    let token = format!("Token {}", config.influxdb.token);
    headers.insert(header::AUTHORIZATION, HeaderValue::from_str(&token).unwrap());
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/plain; charset=utf-8"));
    headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
    let influx_url = format!("{}/api/v2/write?org={}&bucket={}", config.influxdb.host,
        config.influxdb.organization, config.influxdb.bucket);
    let client = reqwest::Client::new();

    println!("Connector started. Querying every {} second(s).", config.frequency_secs);

    let mut last_success_queries: HashMap<String, A2SInfoResult> = HashMap::new();

    loop {
        interval.tick().await;

        let mut influx_data = String::new();

        for q in &queries {
            match q.query_a2s_info() {
                Ok(value) => {
                    last_success_queries.insert(q.full_host.clone(), value.clone());
                    influx_data.push_str(&format!("a2sinfo,host={},community={},game_folder={},game_name={},server_name={},map={} online=true,ping={},num_players={},num_bots={},max_players={}\n",
                        q.full_host, q.community, value.folder, clean_string(&value.game), clean_string(&value.server_name),
                        clean_string(&value.map), value.ping, value.num_players, value.num_bots, value.max_players));
                }
                Err(err) => {
                    eprintln!("Unable to query {}: {}", q.full_host, err);

                    match last_success_queries.get(&q.full_host) {
                        Some(value) => {
                            influx_data.push_str(&format!("a2sinfo,host={},community={},game_folder={},game_name={},server_name={},map={} online=false,ping=5000,num_players=0,num_bots=0,max_players=0\n",
                                q.full_host, q.community, value.folder, clean_string(&value.game), clean_string(&value.server_name), clean_string(&value.map)));
                        }
                        None => {}
                    }
                }
            };
        }

        let response: Response;
        match client.post(influx_url.as_str())
            .headers(headers.clone())
            .body(influx_data)
            .send().await {

            Ok(r) => response = r,
            Err(err) => {
                eprintln!("Unable to save results: {}", err);
                continue;
            }
        }

        let response_status = response.status();
        let response_body: String;

        match response.text().await {
            Ok(body) => response_body = body,
            Err(err) => {
                eprintln!("Unable to read InfluxDB response: {}", err);
                continue;
            }
        }

        if !response_status.is_success() {
            eprintln!("InfluxDB returned error code: {} {}", response_status, response_body);
            continue;
        }
    }

}

fn clean_string(input: &String) -> String {
    input.chars().filter(|&c| c.is_ascii() && c >= ' ').collect::<String>().trim()
        .replace(",", "\\,")
        .replace(" ", "\\ ")
        .replace("=", "\\=")
        .to_string()
}

fn read_config() -> Result<Config, Box<dyn Error>> {
    let file = File::open("config.yaml")?;

    let config: Config = serde_yaml::from_reader(file)?;
    Ok(config)
}

fn default_port() -> u16 {
    27015
}

fn default_unknown() -> String {
    "unknown".to_string()
}