//! The app's side: a listener on the loopback, and each connection served line by line after its token.

use std::future::Future;
use std::io;
use std::ops::RangeInclusive;
use std::pin::Pin;
use std::sync::Arc;

use serde_json::Value;
use subtle::ConstantTimeEq;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use crate::protocol::{Hello, Reply, Request};

/// The ports the app tries, the first free one taken (spec, decision 1).
pub const PORTS: RangeInclusive<u16> = 27650..=27659;

/// What the app does with a request: its method and parameters to an answer.
pub type Handler = Arc<
    dyn Fn(String, Value) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>
        + Send
        + Sync,
>;

/// A launch's token: 256 bits from the operating system, as hexadecimal.
pub fn token() -> io::Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| io::Error::other(e.to_string()))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// A listener on the first free port of `ports`, on the loopback only.
pub async fn bind(ports: RangeInclusive<u16>) -> io::Result<TcpListener> {
    let mut last = io::Error::new(io::ErrorKind::AddrInUse, "no port to try");
    for port in ports {
        match TcpListener::bind(("127.0.0.1", port)).await {
            Ok(listener) => return Ok(listener),
            Err(e) => last = e,
        }
    }
    Err(last)
}

/// Serves every connection to `listener` until the app quits.
pub async fn serve(listener: TcpListener, token: String, handler: Handler) {
    let token = Arc::new(token);
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(connection(stream, Arc::clone(&token), Arc::clone(&handler)));
    }
}

async fn send(writer: &mut (impl AsyncWriteExt + Unpin), reply: &Reply) -> io::Result<()> {
    let mut line = serde_json::to_vec(reply).map_err(io::Error::other)?;
    line.push(b'\n');
    writer.write_all(&line).await
}

/// One client: its hello with the token, then its requests in order, each answered before the next is read.
async fn connection(stream: TcpStream, token: Arc<String>, handler: Handler) -> io::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();
    let hello = lines.next_line().await?.unwrap_or_default();
    let given = serde_json::from_str::<Hello>(&hello)
        .map(|h| h.token)
        .unwrap_or_default();
    if !bool::from(given.as_bytes().ct_eq(token.as_bytes())) {
        return send(&mut writer, &Reply::of(0, Err("wrong token".into()))).await;
    }
    send(
        &mut writer,
        &Reply::of(0, Ok(serde_json::json!({ "app": "twinova" }))),
    )
    .await?;
    while let Some(line) = lines.next_line().await? {
        let reply = match serde_json::from_str::<Request>(&line) {
            Ok(request) => Reply::of(request.id, handler(request.method, request.params).await),
            Err(e) => Reply::of(0, Err(format!("not a request: {e}"))),
        };
        send(&mut writer, &reply).await?;
    }
    Ok(())
}
