mod capabilities;
mod config;
mod convert;
mod handler;
mod semantic_tokens;
mod server;
mod vfs;

use anyhow::{anyhow, Result};
use ide::VfsPath;
use lsp_server::{Connection, ErrorCode};
use lsp_types::{InitializeParams, Url};
use std::fmt;

pub(crate) use server::{Server, StateSnapshot};
pub(crate) use vfs::{LineMap, Vfs};

#[derive(Debug)]
pub(crate) struct LspError {
    code: ErrorCode,
    message: String,
}

impl fmt::Display for LspError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NB. This will be displayed in the editor.
        self.message.fmt(f)
    }
}

impl std::error::Error for LspError {}

pub(crate) trait UrlExt {
    fn to_vfs_path(&self) -> Result<VfsPath>;
}

impl UrlExt for Url {
    fn to_vfs_path(&self) -> Result<VfsPath> {
        let path = self
            .to_file_path()
            .map_err(|()| anyhow!("Non-file URI: {self}"))?;
        Ok(path.try_into()?)
    }
}

pub fn main_loop(conn: Connection) -> Result<()> {
    let init_params =
        conn.initialize(serde_json::to_value(capabilities::server_capabilities()).unwrap())?;
    tracing::info!("Init params: {}", init_params);

    let init_params = serde_json::from_value::<InitializeParams>(init_params)?;

    let root_path = match init_params
        .root_uri
        .as_ref()
        .and_then(|uri| uri.to_file_path().ok())
    {
        Some(path) => path,
        None => std::env::current_dir()?,
    };

    let mut server = Server::new(conn.sender.clone(), root_path);
    server.run(conn.receiver, init_params)?;

    tracing::info!("Leaving main loop");
    Ok(())
}
