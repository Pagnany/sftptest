use std::sync::Arc;

use log::{LevelFilter, info};
use russh::keys::*;
use russh::*;
use russh_sftp::client::SftpSession;
use russh_sftp::protocol::OpenFlags;
use std::env;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct Client;

impl client::Handler for Client {
    type Error = anyhow::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        // info!("check_server_key: {_server_public_key:?}");
        Ok(true)
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();

    // Wenn der Pfad nicht übergeben wurde.
    if args.len() != 4 {
        println!("Falsche Anzahl an Argumenten übergeben.");
        println!("Beispiel: {} localhost:22 login pw", args[0]);
        panic!("Falsche Anzahl an Argumenten übergeben.");
    }
    let server = args[1].split(':').next().unwrap_or("localhost").to_string();
    let port = args[1]
        .split(':')
        .nth(1)
        .unwrap_or("22")
        .parse::<u16>()
        .unwrap_or(22);
    let username = args[2].clone();
    let password = args[3].clone();

    env_logger::builder().filter_level(LevelFilter::Info).init();

    let config = russh::client::Config::default();
    let sh = Client {};
    let mut session = russh::client::connect(Arc::new(config), (server, port), sh).await?;

    if !session
        .authenticate_password(username, password)
        .await?
        .success()
    {
        panic!("authentication failed");
    }

    // open sftp session
    let channel = session.channel_open_session().await?;
    channel.request_subsystem(true, "sftp").await?;
    let sftp = SftpSession::new(channel.into_stream()).await?;

    // scanning directory
    let mut count = 0;
    for entry in sftp.read_dir("media/produktbilder").await? {
        info!("file in directory: {:?}", entry.file_name());
        count += 1;
    }
    info!("total files: {}", count);

    if false {
        // upload file
        let remote_file_name = "/media/produktbilder/_bild.jpg";
        let local_file_name = "C:/Users/eric.schmale/Desktop/bild.jpg";

        let mut local_file = File::open(local_file_name).await?;
        let mut buffer = Vec::new();
        local_file.read_to_end(&mut buffer).await?;

        let mut file = sftp
            .open_with_flags(
                remote_file_name,
                OpenFlags::CREATE | OpenFlags::TRUNCATE | OpenFlags::WRITE | OpenFlags::READ,
            )
            .await?;
        file.write_all(&buffer).await?;
        file.shutdown().await?;
    }

    Ok(())
}
