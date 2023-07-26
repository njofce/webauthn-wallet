use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use env_logger::Env;
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::fs::File;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
struct PrecomputedBytecode {
    bytecode: String,
}

#[get("/precompute/{x}/{y}")]
async fn precompute(public_key_coords: web::Path<(String, String)>) -> impl Responder {
    let (x, y) = public_key_coords.into_inner();
    format!("Precomputing tables for public key coordinates: x = {x}, y = {y}!");

    let script_base_folder = std::env::var("BASE_FOLDER").unwrap();

    let mut owned_coord: String = x.to_owned();
    let owned_y: String = y.to_owned();
    owned_coord.push_str(&owned_y);

    let mut file_name: String = script_base_folder.to_owned();
    file_name.push_str(&digest(owned_coord));

    let mut script_name: String = script_base_folder.to_owned();
    script_name.push_str(&"precompute.sage");

    let _output = Command::new("sh")
        .arg("-C")
        .arg("./precompute.sh")
        .arg(x)
        .arg(y)
        .arg(file_name.to_owned())
        .arg(script_name.to_owned())
        .status();

    let mut owned_filename: String = file_name.to_owned();
    owned_filename.push_str(&".json");

    let file_open = File::open(owned_filename);

    match file_open {
        Ok(file) => {
            let bytecode: serde_json::Value =
                serde_json::from_reader(file).expect("JSON was not well-formatted");

            let output = bytecode.get("bytecode").unwrap().to_string().to_owned();

            let _delete_command_output = Command::new("sh")
                .arg("-C")
                .arg("./rm.sh")
                .arg(file_name.to_owned())
                .output()
                .unwrap();

            HttpResponse::Ok().json(PrecomputedBytecode { bytecode: output })
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let host = std::env::var("HOST").unwrap();
    let port = std::env::var("PORT").unwrap().parse().unwrap();

    HttpServer::new(|| App::new().service(precompute))
        .bind((host, port))?
        .run()
        .await
}
