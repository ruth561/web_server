use std::{io::Read};
use log::{debug, error, info, warn};
use pavao::{SmbClient, SmbOpenOptions};
use actix_web::{
    get, App, HttpResponse, HttpServer, Responder, HttpRequest
};

mod nas;



/// This function returns all metadata in the shared folder in nas.
#[get("/api/all")]
async fn get_all_metadata(req: HttpRequest) -> impl Responder
{
    info!("{:?} {:?}", req.method(), req.uri());
    let conn: SmbClient = nas::connect();
    match conn.list_dir("/") {
        Ok(dirents) => {
            let mut json = serde_json::json!({});
            for dirent in dirents {
                if dirent.name() == "#recycle" {
                    continue;
                }
                debug!("get_all_metadata | {:?}", dirent.name());                
                let path = format!("/{}/metadata.json", dirent.name());
                if let Ok(metadata) = nas::get_file_s(&conn, &path) {
                    json[dirent.name()] = serde_json::from_str(&metadata).unwrap();
                }
            }
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(json.to_string())
        }
        Err(e) => {
            error!("get_all_metadata | {:?}", e);
            HttpResponse::Ok()
                .body("There isn't file in nas.")
        }
    }
}

/// This function is invoked when requested like /api/seminars/01.
/// A return value is nas_shared_folder/{seminar_id}/metadata.json if it exists
/// ans "There isn't file in nas." if it doesn't exit.
#[get("/api/{seminar_id}")]
async fn get_metadata(req: HttpRequest) -> impl Responder
{
    info!("{:?} {:?}", req.method(), req.uri());
    let seminar_id = req.match_info().get("seminar_id")
        .expect("[ Error ] Pattern match doesn't work well.");

    let nas: SmbClient = nas::connect();
    match nas.list_dir("/") {
        Ok(dirents) => {
            for dirent in dirents {
                if dirent.name() == seminar_id {
                    let path = format!("/{}/metadata.json", seminar_id);
                    if let Ok(mut file) = nas.open_with(path, 
                        SmbOpenOptions::default().read(true)) {
                        let mut res = String::new();
                        if file.read_to_string(&mut res).is_ok() {
                            return HttpResponse::Ok()
                                .content_type("text/html; charset=utf-8")
                                .body(res);
                        }
                    } else {
                        warn!("get_metadata | failed to accsess to nas with {}.", seminar_id);
                    }
                }
            }
        }
        Err(e) => {
            error!("get_metadata | {:?}", e);
        }
    }
    HttpResponse::Ok().body("There isn't file in nas.")
}

/// A return value is nas_shared_folder/{seminar_id}/{filename} if it exists.
#[get("/nas/{seminar_id}/{filename}")]
async fn get_file_data(req: HttpRequest) -> impl Responder
{
    info!("{:?} {:?}", req.method(), req.uri());
    let seminar_id = req.match_info().get("seminar_id")
        .expect("[ Error ] Pattern match doesn't work well.");
    let filename = req.match_info().get("filename")
        .expect("[ Error ] Pattern match doesn't work well.");
    
    let conn: SmbClient = nas::connect();
    let path = format!("/{}/{}", seminar_id, filename);

    if let Ok(file_data) = nas::get_file_b(&conn, &path) {
        HttpResponse::Ok().body(file_data)
    } else {
        error!("get_file_data | failed to accsess to nas with path {}.", path);
        HttpResponse::Ok().body("There isn't file in nas.")
    }
}


/// Entry point
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .service(get_all_metadata)
            .service(get_metadata)
            .service(get_file_data)
    })
    .bind(("127.0.0.1", 5000))?
    .run()
    .await
}
