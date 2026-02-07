use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::timeout;

use crate::config::EngineConfig;
use crate::engine::EngineSpec;

#[derive(Debug, Clone)]
pub struct UciEngineInfo {
    pub name: String,
    pub author: String,
}

#[derive(Debug)]
pub enum UciError {
    Io(std::io::Error),
    Timeout(&'static str),
    UnexpectedEof,
    InvalidResponse(String),
}

impl std::fmt::Display for UciError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UciError::Io(err) => write!(f, "io error: {err}"),
            UciError::Timeout(stage) => write!(f, "timeout waiting for {stage}"),
            UciError::UnexpectedEof => write!(f, "unexpected EOF"),
            UciError::InvalidResponse(line) => write!(f, "invalid response: {line}"),
        }
    }
}

impl From<std::io::Error> for UciError {
    fn from(err: std::io::Error) -> Self {
        UciError::Io(err)
    }
}

pub struct UciProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl UciProcess {
    pub async fn spawn(path: &PathBuf, args: &[String], working_dir: Option<&PathBuf>) -> Result<Self, UciError> {
        let mut command = Command::new(path);
        command.args(args);
        command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
        if let Some(dir) = working_dir {
            command.current_dir(dir);
        }

        let mut child = command.spawn()?;
        let stdin = child.stdin.take().ok_or(UciError::UnexpectedEof)?;
        let stdout = child.stdout.take().ok_or(UciError::UnexpectedEof)?;
        let stdout = BufReader::new(stdout);

        Ok(Self { child, stdin, stdout })
    }

    pub async fn send_line(&mut self, line: &str) -> Result<(), UciError> {
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }

    async fn read_line(&mut self) -> Result<String, UciError> {
        let mut buf = String::new();
        let bytes = self.stdout.read_line(&mut buf).await?;
        if bytes == 0 {
            return Err(UciError::UnexpectedEof);
        }
        Ok(buf.trim().to_string())
    }

    pub async fn handshake(&mut self) -> Result<UciEngineInfo, UciError> {
        self.send_line("uci").await?;

        let mut name = None;
        let mut author = None;

        loop {
            let line = self.read_line().await?;
            if let Some(rest) = line.strip_prefix("id name ") {
                name = Some(rest.trim().to_string());
            } else if let Some(rest) = line.strip_prefix("id author ") {
                author = Some(rest.trim().to_string());
            } else if line == "uciok" {
                break;
            }
        }

        Ok(UciEngineInfo {
            name: name.unwrap_or_else(|| "".to_string()),
            author: author.unwrap_or_else(|| "".to_string()),
        })
    }

    pub async fn is_ready(&mut self) -> Result<(), UciError> {
        self.send_line("isready").await?;
        loop {
            let line = self.read_line().await?;
            if line == "readyok" {
                break;
            }
        }
        Ok(())
    }

    pub async fn ucinewgame(&mut self) -> Result<(), UciError> {
        self.send_line("ucinewgame").await
    }

    pub async fn bestmove(&mut self, wtime: u64, btime: u64, timeout_ms: u64) -> Result<String, UciError> {
        self.send_line(&format!("go wtime {wtime} btime {btime}")).await?;

        let deadline = Duration::from_millis(timeout_ms);
        let line = timeout(deadline, async {
            loop {
                let line = self.read_line().await?;
                if let Some(rest) = line.strip_prefix("bestmove ") {
                    return Ok::<String, UciError>(rest.trim().to_string());
                }
            }
        })
        .await
        .map_err(|_| UciError::Timeout("bestmove"))??;

        Ok(line)
    }

    pub async fn quit(mut self) -> Result<(), UciError> {
        let _ = self.send_line("quit").await;
        let _ = self.child.wait().await;
        Ok(())
    }
}

pub async fn discover_engines(configs: &[EngineConfig]) -> Result<Vec<EngineSpec>, UciError> {
    let mut engines = Vec::new();

    for entry in configs {
        let mut process = match UciProcess::spawn(&entry.path, &entry.args, entry.working_dir.as_ref()).await {
            Ok(process) => process,
            Err(err) => {
                eprintln!("failed to spawn engine {}: {err}", entry.id);
                continue;
            }
        };

        let info = match timeout(Duration::from_secs(5), process.handshake()).await {
            Ok(Ok(info)) => info,
            Ok(Err(err)) => {
                eprintln!("uci handshake failed for {}: {err}", entry.id);
                let _ = process.quit().await;
                continue;
            }
            Err(_) => {
                eprintln!("uci handshake timeout for {}", entry.id);
                let _ = process.quit().await;
                continue;
            }
        };

        let _ = process.is_ready().await;
        let _ = process.quit().await;

        engines.push(EngineSpec {
            id: entry.id.clone(),
            name: if info.name.is_empty() { entry.id.clone() } else { info.name },
            author: info.author,
            path: entry.path.clone(),
            args: entry.args.clone(),
            working_dir: entry.working_dir.clone(),
        });
    }

    Ok(engines)
}
