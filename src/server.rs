use std::sync::{mpsc, Arc};

use core::convert::TryInto;

use embedded_svc::{
    http::{Headers, Method},
    io::{Read, Write},
};
use esp_idf_svc::http::server::EspHttpServer;

use serde::{Deserialize, Serialize};

use crate::audio;
use crate::global;

#[derive(Debug, Deserialize)]
struct TTSRequest {
    text: String, // 文本内容
}

#[derive(Debug, Deserialize)]
struct VolumeRequest {
    op: String, // 操作类型: "inc" 或 "dec"
}

pub fn server(ui_tx: mpsc::Sender<String>, tts_tx: mpsc::Sender<String>) -> anyhow::Result<()> {
    log::info!("starting server");

    let mut server = create_server()?;

    server.fn_handler("/", Method::Get, |req| {
        req.into_ok_response()?
            .write(global::INDEX_HTML.as_bytes())
            .map(|_| ())
    })?;

    server.fn_handler::<anyhow::Error, _>("/api/tts", Method::Post, |mut req| {
        let len = req.content_len().unwrap_or(0) as usize;
        if len > MAX_LEN {
            req.into_status_response(413)?
                .write_all("Request too big".as_bytes())?;
            return Ok(());
        }

        let mut buf = vec![0; len];
        req.read_exact(&mut buf)?;
        let mut resp = req.into_ok_response()?;

        if let Ok(request) = serde_json::from_slice::<TTSRequest>(&buf) {
            log::info!("request: {:?}", request);
            ui_tx.send(request.text.clone());
            tts_tx.send(request.text);
        } else {
            resp.write_all("JSON error".as_bytes())?;
            return Ok(());
        }

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

        let mut buf = vec![0; len];
        req.read_exact(&mut buf)?;
        let mut resp = req.into_ok_response()?;

        if let Ok(request) = serde_json::from_slice::<VolumeRequest>(&buf) {
            log::info!("request: {:?}", request);
            match request.op {
                "dec" => {
                    audio::volume_down();
                }
                "inc" => {
                    audio::volume_up();
                }
                _ => {
                    log::warn!("unkonw op: {:?}", request.op);
                }
            }
        } else {
            resp.write_all("JSON error".as_bytes())?;
            return Ok(());
        }

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
