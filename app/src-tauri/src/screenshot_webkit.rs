//! The window's picture on Linux (screenshot.rs): WebKitGTK's snapshot of the visible view, written as PNG by cairo.
//! It needs a real window, so mutation leaves it out (`.cargo/mutants.toml`); e2e/mcp.mjs takes the picture under Xvfb
//! and checks its size.

use tokio::sync::oneshot;

pub(super) fn capture(
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
