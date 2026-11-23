//! A window as it is drawn, for the agent (spec 2026-11-23-app-mcp-design.md, decision 3). Tauri 2.12.5 has no capture
//! of a webview; `with_webview` hands the platform's own, and each platform draws a picture of it:
//! - Linux: WebKitGTK's `webkit_web_view_get_snapshot` (webkit2gtk 2.0.2), written as PNG by cairo; the 3D view's WebGL
//!   is in it (measured 2026-11-23: 1440 × 900 in 117 ms under Xvfb);
//! - Windows: WebView2's `ICoreWebView2::CapturePreview` (webview2-com 0.39.1) into a memory stream, as PNG.

use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::oneshot;

/// The picture `screenshot` answers: PNG bytes as base64, and the size its header gives.
#[derive(Debug, Clone, Serialize)]
pub struct Picture {
    pub window: String,
    pub png: String,
    pub width: u32,
    pub height: u32,
}

/// A PNG's width and height, from its IHDR chunk (PNG 1.2, section 11.2.2: right after the 8-byte signature and the
/// chunk's length and type).
pub fn png_size(png: &[u8]) -> Result<(u32, u32), String> {
    const SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
    if png.len() < 24 || !png.starts_with(SIGNATURE) || &png[12..16] != b"IHDR" {
        return Err("the window's picture is not a PNG".into());
    }
    let word = |at: usize| u32::from_be_bytes([png[at], png[at + 1], png[at + 2], png[at + 3]]);
    Ok((word(16), word(20)))
}

/// Window `label` drawn as it is now.
pub async fn take<R: Runtime>(app: &AppHandle<R>, label: &str) -> Result<Picture, String> {
    let window = app
        .get_webview_window(label)
        .ok_or_else(|| format!("no {label} window open"))?;
    let (sender, picture) = oneshot::channel::<Result<Vec<u8>, String>>();
    window
        .with_webview(move |webview| capture(webview, sender))
        .map_err(|e| e.to_string())?;
    let png = picture
        .await
        .map_err(|_| "the window gave no picture".to_string())??;
    let (width, height) = png_size(&png)?;
    use base64::Engine;
    Ok(Picture {
        window: label.to_string(),
        png: base64::engine::general_purpose::STANDARD.encode(&png),
        width,
        height,
    })
}

#[cfg(target_os = "linux")]
fn capture(
    webview: tauri::webview::PlatformWebview,
    sender: oneshot::Sender<Result<Vec<u8>, String>>,
) {
    use webkit2gtk::WebViewExt;
    webview.inner().snapshot(
        webkit2gtk::SnapshotRegion::Visible,
        webkit2gtk::SnapshotOptions::NONE,
        None::<&webkit2gtk::gio::Cancellable>,
        move |surface| {
            let png = surface.map_err(|e| e.to_string()).and_then(|surface| {
                let image = cairo::ImageSurface::try_from(surface)
                    .map_err(|_| "WebKit's picture is not an image surface".to_string())?;
                let mut png = Vec::new();
                image.write_to_png(&mut png).map_err(|e| e.to_string())?;
                Ok(png)
            });
            let _ = sender.send(png);
        },
    );
}

#[cfg(windows)]
fn capture(
    webview: tauri::webview::PlatformWebview,
    sender: oneshot::Sender<Result<Vec<u8>, String>>,
) {
    capture_webview2(&webview.controller(), sender);
}

/// WebView2's picture of the webview `controller` holds, as PNG bytes through `sender`.
#[cfg(windows)]
fn capture_webview2(
    controller: &webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller,
    sender: oneshot::Sender<Result<Vec<u8>, String>>,
) {
    use webview2_com::CapturePreviewCompletedHandler;
    use webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_CAPTURE_PREVIEW_IMAGE_FORMAT_PNG;
    use windows::Win32::Foundation::HGLOBAL;
    use windows::Win32::System::Com::StructuredStorage::CreateStreamOnHGlobal;
    use windows::Win32::System::Com::{IStream, STREAM_SEEK_SET};

    /// The whole of `stream`, from its start.
    fn read(stream: &IStream) -> windows::core::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        let mut chunk = vec![0u8; 1 << 16];
        unsafe {
            stream.Seek(0, STREAM_SEEK_SET, None)?;
            loop {
                let mut got = 0u32;
                stream
                    .Read(
                        chunk.as_mut_ptr().cast(),
                        chunk.len() as u32,
                        Some(&mut got),
                    )
                    .ok()?;
                if got == 0 {
                    return Ok(bytes);
                }
                bytes.extend_from_slice(&chunk[..got as usize]);
            }
        }
    }

    let started = (|| -> windows::core::Result<(IStream, _)> {
        let core = unsafe { controller.CoreWebView2()? };
        let stream = unsafe { CreateStreamOnHGlobal(HGLOBAL::default(), true)? };
        Ok((stream, core))
    })();
    let (stream, core) = match started {
        Ok(started) => started,
        Err(e) => return drop(sender.send(Err(e.to_string()))),
    };
    let written = stream.clone();
    let handler = CapturePreviewCompletedHandler::create(Box::new(move |result| {
        let _ = sender.send(
            result
                .and_then(|()| read(&written))
                .map_err(|e| e.to_string()),
        );
        Ok(())
    }));
    if let Err(e) = unsafe {
        core.CapturePreview(
            COREWEBVIEW2_CAPTURE_PREVIEW_IMAGE_FORMAT_PNG,
            &stream,
            &handler,
        )
    } {
        eprintln!("the window's picture could not start: {e}");
    }
}

#[cfg(not(any(target_os = "linux", windows)))]
fn capture(_: tauri::webview::PlatformWebview, sender: oneshot::Sender<Result<Vec<u8>, String>>) {
    let _ = sender.send(Err(
        "a window's picture is taken on Linux and Windows only".into()
    ));
}

#[cfg(test)]
#[path = "screenshot_tests.rs"]
mod tests;
