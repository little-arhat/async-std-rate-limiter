use crate::rate_limiter;
use crate::symbols;

use core::num::NonZeroU8;

use async_std::net::TcpListener;
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::task;
use futures::stream::StreamExt;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

const SUCCESS: &[u8] = "0\n".as_bytes();
const FAILURE: &[u8] = "1\n".as_bytes();

pub async fn serve(partitions: Vec<Vec<usize>>, address: String, limit: NonZeroU8) -> Result<()> {
    let listener = TcpListener::bind(address).await.unwrap();
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        println!("Accepted client from: {}", stream.peer_addr()?);
        safe_spawn(for_client(stream, partitions.clone(), limit));
    }

    Ok(())
}

async fn for_client(
    mut stream: TcpStream,
    partitions: Vec<Vec<usize>>,
    limit: NonZeroU8,
) -> Result<()> {
    let inp_stream = stream.clone();
    let peer_addr = stream.peer_addr()?;
    let reader = async_std::io::BufReader::new(&inp_stream);
    let mut lines = reader.lines();
    let rls = rate_limiter::init::<{ symbols::N }>(partitions, limit);

    while let Some(line) = lines.next().await {
        let line = line?;
        println!("<= message from {}: {};", peer_addr, line);
        let response = match line
            .chars()
            .nth(0)
            .and_then(symbols::to_index)
            .and_then(|ix| rls.check_key(&ix).ok())
        {
            Some(_) => SUCCESS,
            None => FAILURE,
        };
        println!(">= message to {}: {:?};", peer_addr, response);
        stream.write_all(response).await?
    }
    println!("Client from {} disconnected", peer_addr);

    Ok(())
}

fn safe_spawn<F>(fut: F) -> task::JoinHandle<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    task::spawn(async move {
        if let Err(e) = fut.await {
            eprintln!("{}", e)
        }
    })
}
