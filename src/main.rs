mod config;
mod packet;

use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter, ReadHalf, WriteHalf, split},
    net::{TcpListener, TcpStream},
};

use crate::{
    config::{Config, Server},
    packet::{Packet, State},
};

const MAX_PACKET_SIZE: usize = 1024 * 2048;
const ALLOCATE_SIZE: usize = MAX_PACKET_SIZE / 8;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let conf = Config::read_config()?;
    let listener = match TcpListener::bind(conf.listen).await {
        Ok(value) => value,
        Err(error) => Err(error)?,
    };

    log::info!("Simple MCProxy successfully initialized!");

    loop {
        let servers = conf.servers.clone();
        let default_server = conf.default_server.clone();
        let stream = match listener.accept().await {
            Ok((stream, addr)) => {
                log::info!("Client connected from {addr}");
                stream
            }
            Err(error) => {
                log::error!("An error occurred while accepting client: {error}");
                continue;
            }
        };

        tokio::spawn(async move {
            match connection(stream, servers.clone(), default_server).await {
                Ok(_) => {}
                Err(error) => log::error!("{error}"),
            }
        });
    }
}

async fn connection(
    client_stream: TcpStream,
    servers: Vec<Server>,
    default_server: String,
) -> Result<()> {
    let state = State::Handshaking;
    let mut buf = Vec::with_capacity(ALLOCATE_SIZE);
    let (client_rx, client_tx) = split(client_stream);
    let mut client_reader = BufReader::new(client_rx);
    let client_writer = BufWriter::new(client_tx);

    client_reader.read_buf(&mut buf).await?;
    let Packet::Handshake(packet) = Packet::parse(state, &buf)?;
    let server = match servers
        .into_iter()
        .find(|server| server.hostname == packet.server)
    {
        Some(server) => server.dest.clone(),
        None => default_server,
    };

    let server_stream = match TcpStream::connect(&server).await {
        Ok(stream) => stream,
        Err(error) => {
            log::error!("An error occurred while connectiong to backend server({server}): {error}");
            Err(error)?
        }
    };
    let (server_rx, server_tx) = split(server_stream);
    let server_reader = BufReader::new(server_rx);
    let mut server_writer = BufWriter::new(server_tx);

    server_writer.write_all(&buf).await?;
    server_writer.flush().await?;

    tokio::spawn(proxy(client_reader, server_writer));
    tokio::spawn(proxy(server_reader, client_writer));

    Ok(())
}

async fn proxy(
    mut recv: BufReader<ReadHalf<TcpStream>>,
    mut send: BufWriter<WriteHalf<TcpStream>>,
) -> Result<()> {
    let mut buf = Vec::with_capacity(ALLOCATE_SIZE);
    loop {
        recv.read_buf(&mut buf).await?;
        if buf.is_empty() {
            break;
        }
        send.write_all(&buf).await?;
        send.flush().await?;
        buf.clear();
    }
    Ok(())
}
