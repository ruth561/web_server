
use std::io::Read;
use log::{debug, error};
use pavao::{
    SmbClient, SmbCredentials, SmbOptions, SmbOpenOptions, SmbResult,
    SmbError
};
mod setting;

pub fn connect() -> SmbClient
{
    SmbClient::new(
        SmbCredentials::default()
            .server(setting::SERVER)
            .share(setting::SHARE)
            .password(setting::PASSWORD)
            .username(setting::USERNAME)
            .workgroup(setting::WORKGROUP), 
        SmbOptions::default().one_share_per_server(true),
    )
    .expect("[ ERROR ] Failed to connect to the NAS.")
}

pub fn get_file_b(conn: &SmbClient, path: &str) -> SmbResult<Vec<u8>>
{
    debug!("accsess to nas with this path {}.", path);
    let mut file = conn.open_with(path, SmbOpenOptions::default().read(true))?;
    let mut res = Vec::<u8>::new();
    if let Err(e) = file.read_to_end(&mut res) {
        error!("{:?}", e);
        return Err(SmbError::Io(e));
    }
    Ok(res)
}

pub fn get_file_s(conn: &SmbClient, path: &str) -> SmbResult<String>
{
    debug!("accsess to nas with this path {}.", path);
    let mut file = conn.open_with(path, SmbOpenOptions::default().read(true))?;
    let mut res = String::new();
    if let Err(e) = file.read_to_string(&mut res) {
        error!("[ ERROR ] nas::get_file | {:?}", e);
        return Err(SmbError::Io(e));
    }
    Ok(res)
}
