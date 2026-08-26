//! Length-prefixed Unix-domain IPC. Local-only; never TCP.

use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use clipl_core::error::{Error, Result};
use clipl_core::paths;

use crate::{Envelope, Message, Request, Response};

/// Current request/response vocabulary version.
pub const PROTOCOL_VERSION: u32 = 1;

const MAX_FRAME: u32 = 8 * 1024 * 1024;

/// Write a JSON envelope as `u32le` length + bytes.
pub fn write_frame<W: Write>(mut writer: W, envelope: &Envelope) -> Result<()> {
    let bytes = envelope.to_json_bytes()?;
    let len = u32::try_from(bytes.len()).map_err(|_| Error::Protocol("frame too large".into()))?;
    if len > MAX_FRAME {
        return Err(Error::Protocol("frame exceeds 8 MiB".into()));
    }
    writer
        .write_all(&len.to_le_bytes())
        .map_err(|err| Error::Io(err.to_string()))?;
    writer
        .write_all(&bytes)
        .map_err(|err| Error::Io(err.to_string()))?;
    writer.flush().map_err(|err| Error::Io(err.to_string()))?;
    Ok(())
}

/// Read one length-prefixed envelope.
pub fn read_frame<R: Read>(mut reader: R) -> Result<Envelope> {
    let mut len_buf = [0u8; 4];
    reader
        .read_exact(&mut len_buf)
        .map_err(|err| Error::Io(err.to_string()))?;
    let len = u32::from_le_bytes(len_buf);
    if len > MAX_FRAME {
        return Err(Error::Protocol("frame exceeds 8 MiB".into()));
    }
    let mut buf = vec![0u8; len as usize];
    reader
        .read_exact(&mut buf)
        .map_err(|err| Error::Io(err.to_string()))?;
    Envelope::from_json_bytes(&buf)
}

/// Client connected to a running daemon.
pub struct IpcClient {
    stream: UnixStream,
}

impl IpcClient {
    /// Connect to the default XDG runtime socket.
    pub fn connect() -> Result<Self> {
        Self::connect_path(&paths::socket_path())
    }

    /// Connect to an explicit socket path.
    pub fn connect_path(path: &Path) -> Result<Self> {
        let stream = UnixStream::connect(path).map_err(|err| {
            Error::Io(format!(
                "clipl-daemon is not running (socket {}): {err}",
                path.display()
            ))
        })?;
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|err| Error::Io(err.to_string()))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .map_err(|err| Error::Io(err.to_string()))?;
        Ok(Self { stream })
    }

    /// Send a request and wait for the matching response.
    pub fn request(&mut self, request: Request) -> Result<Response> {
        let outgoing = Envelope::new(Message::Request(request));
        write_frame(&mut self.stream, &outgoing)?;
        let incoming = read_frame(&mut self.stream)?;
        if incoming.id != outgoing.id {
            return Err(Error::Protocol("response correlation id mismatch".into()));
        }
        match incoming.payload {
            Message::Response(response) => Ok(response),
            _ => Err(Error::Protocol("expected a response".into())),
        }
    }
}

/// Restrictive permissions for a listening socket.
pub fn set_socket_mode(path: &Path) -> Result<()> {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .map_err(|err| Error::Io(format!("{}: {err}", path.display())))
}

/// Remove a stale socket if nothing is listening.
pub fn cleanup_stale_socket(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    match UnixStream::connect(path) {
        Ok(_) => Err(Error::Io(format!(
            "clipl-daemon already running at {}",
            path.display()
        ))),
        Err(_) => {
            std::fs::remove_file(path)
                .map_err(|err| Error::Io(format!("{}: {err}", path.display())))?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Message, Request};
    use std::os::unix::net::UnixListener;
    use std::thread;

    #[test]
    fn request_response_over_unix_socket() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.sock");
        let listener = UnixListener::bind(&path).unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let env = read_frame(&mut stream).unwrap();
            assert!(matches!(env.payload, Message::Request(Request::Ping)));
            let reply = Envelope {
                id: env.id,
                payload: Message::Response(Response::Pong),
            };
            write_frame(&mut stream, &reply).unwrap();
        });
        let mut client = IpcClient::connect_path(&path).unwrap();
        let response = client.request(Request::Ping).unwrap();
        assert!(matches!(response, Response::Pong));
        server.join().unwrap();
    }
}
