use actix_files::NamedFile;
use actix_web::Result;

pub async fn network_settings() -> Result<NamedFile> {
    Ok(NamedFile::open("network_settings.nws")?)
}