//! A window as it is drawn, for the agent (spec 2026-11-23-app-mcp-design.md, decision 3). Tauri 2.12.5 has no capture
//! of a webview; `with_webview` hands the platform's own, and each platform draws a picture of it:
//! - Linux: WebKitGTK's `webkit_web_view_get_snapshot` (webkit2gtk 2.0.2), written as PNG by cairo; the 3D view's WebGL
//!   is in it (measured 2026-11-23: 1440 × 900 in 117 ms under Xvfb) (`screenshot_webkit.rs`);
//! - Windows: WebView2's `ICoreWebView2::CapturePreview` (webview2-com 0.39.1) into a memory stream, as PNG
//!   (`screenshot_webview2.rs`).

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
#[path = "screenshot_webkit.rs"]
mod webkit;
#[cfg(target_os = "linux")]
use webkit::capture;

#[cfg(windows)]
#[path = "screenshot_webview2.rs"]
mod webview2;
#[cfg(windows)]
use webview2::capture;

#[cfg(not(any(target_os = "linux", windows)))]
fn capture(_: tauri::webview::PlatformWebview, sender: oneshot::Sender<Result<Vec<u8>, String>>) {
    let _ = sender.send(Err(
        "a window's picture is taken on Linux and Windows only".into()
    ));
}

#[cfg(test)]
#[path = "screenshot_tests.rs"]
mod tests;
