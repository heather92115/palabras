use aws_config::{self, BehaviorVersion, Region};
use aws_sdk_secretsmanager;
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
pub struct DbLink {
    pub username: String,
    pub password: String,
    pub dbname: String,
    pub host: String,
    pub port: String,
}

async fn lookup_url(db_link: String, region_str: String) -> Result<Option<String>, String> {
    let region = Region::new(region_str.clone());

    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region(region)
        .load()
        .await;

    let asm = aws_sdk_secretsmanager::Client::new(&config);

    let response = asm
        .get_secret_value()
        .secret_id(db_link.clone())
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.secret_string().is_none() {
        return Err(format!(
            "Failed to find {} at region {}",
            db_link, region_str
        ));
    }

    let db_link: DbLink = serde_json::from_str(response.secret_string().unwrap_or_default())
        .map_err(|err| err.to_string())?;

    let url = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_link.username, db_link.password, db_link.host, db_link.port, db_link.dbname
    );

    Ok(Some(url))
}

fn fallback_database_url() -> String {
    println!("!!!USING FALLBACK DATABASE URL!!!");

    env::var("PAL_DATABASE_URL").expect("fallback env var DATABASE_URL was hoped for but not found")
}

pub async fn find_the_database() -> String {
    let db_link = env::var("PAL_DB_LINK").unwrap_or_default();
    let region = env::var("PAL_REGION").unwrap_or_default();

    if db_link.is_empty() || region.is_empty() {
        fallback_database_url()
    } else {
        if let Ok(Some(url)) = lookup_url(db_link, region).await {
            url
        } else {
            fallback_database_url()
        }
    }
}
