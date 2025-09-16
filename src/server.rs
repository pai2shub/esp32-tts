use std::sync::Arc;

use core::convert::TryInto;

use embedded_svc::{
    http::{Headers, Method},
    io::{Read, Write},
};
use esp_idf_svc::http::server::EspHttpServer;

use serde::{Deserialize, Serialize};

// Max payload length
const MAX_LEN: usize = 128;

// Need lots of stack to parse JSON
const STACK_SIZE: usize = 10240;

pub fn server() -> anyhow::Result<()> {
    log::info!("starting server");

    let mut server = create_server()?;

    server.fn_handler("/", Method::Get, |req| {
        req.into_ok_response()?
            .write(constant::INDEX_HTML.as_bytes())
            .map(|_| ())
    })?;

    server.fn_handler::<anyhow::Error, _>("/api/tts", Method::Post, |mut req| {
        let len = req.content_len().unwrap_or(0) as usize;
        if len > MAX_LEN {
            req.into_status_response(413)?
                .write_all("Request too big".as_bytes())?;
            return Ok(());
        }

        // todo

        resp.write_all("{}".as_bytes())?;
        Ok(())
    });

    server.fn_handler::<anyhow::Error, _>("/api/volume", Method::Put, |mut req| {
        let len = req.content_len().unwrap_or(0) as usize;

        if len > MAX_LEN {
            req.into_status_response(413)?
                .write_all("Request too big".as_bytes())?;
            return Ok(());
        }

        // todo

        resp.write_all("{}".as_bytes())?;
        Ok(())
    })?;

    core::mem::forget(server);

    Ok(())
}

fn create_server() -> anyhow::Result<EspHttpServer<'static>> {
    let server_configuration = esp_idf_svc::http::server::Configuration {
        stack_size: STACK_SIZE,
        ..Default::default()
    };

    Ok(EspHttpServer::new(&server_configuration)?)
}
