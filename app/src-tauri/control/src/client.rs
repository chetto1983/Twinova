//! The agent's side: a connection to the open app, found through its file or named by its port.

use std::path::Path;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use crate::file::ControlFile;
use crate::protocol::{Hello, Reply, Request};

pub struct Client {
    lines: Lines<BufReader<OwnedReadHalf>>,
    writer: OwnedWriteHalf,
    next: u64,
}

impl Client {
    /// Connects to the app on `port` with the launch's `token`.
    pub async fn connect(port: u16, token: &str) -> Result<Self, String> {
        let stream = TcpStream::connect(("127.0.0.1", port))
            .await
            .map_err(|e| format!("no Twinova app on port {port}: {e}"))?;
        let (reader, writer) = stream.into_split();
        let mut client = Self {
            lines: BufReader::new(reader).lines(),
            writer,
            next: 1,
        };
        client
            .write(&Hello {
                token: token.to_string(),
            })
            .await?;
        client.read().await?.answer()?;
        Ok(client)
    }

    /// Connects to the app whose file is in `data_dir`.
    pub async fn discover(data_dir: &Path) -> Result<Self, String> {
        let file = ControlFile::read(data_dir).map_err(|e| {
            format!(
                "no open Twinova app (no {}: {e})",
                crate::file::path(data_dir).display()
            )
        })?;
        Self::connect(file.port, &file.token).await
    }

    /// Runs `method` in the app, and its answer.
    pub async fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.next;
        self.next += 1;
        self.write(&Request {
            id,
            method: method.to_string(),
            params,
        })
        .await?;
        let reply = self.read().await?;
        if reply.id != id {
            return Err(format!("the app answered request {} for {id}", reply.id));
        }
        reply.answer()
    }

    async fn write(&mut self, message: &impl serde::Serialize) -> Result<(), String> {
        let mut line = serde_json::to_vec(message).map_err(|e| e.to_string())?;
        line.push(b'\n');
        self.writer
            .write_all(&line)
            .await
            .map_err(|e| format!("the app left: {e}"))
    }

    async fn read(&mut self) -> Result<Reply, String> {
        let line = (self.lines.next_line().await)
            .map_err(|e| format!("the app left: {e}"))?
            .ok_or("the app closed the connection")?;
        serde_json::from_str(&line).map_err(|e| format!("not an answer: {e}"))
    }
}
