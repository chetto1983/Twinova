//! The window's picture on Windows (screenshot.rs): WebView2's `ICoreWebView2::CapturePreview` (webview2-com 0.39.1)
//! into a memory stream, as PNG. Compiled on Windows only, where no mutation runs (`.cargo/mutants.toml`); e2e/mcp.mjs
//! takes the picture on windows-2022 and checks its size.

use tokio::sync::oneshot;

pub(super) fn capture(
    webview: tauri::webview::PlatformWebview,
    sender: oneshot::Sender<Result<Vec<u8>, String>>,
) {
    capture_webview2(&webview.controller(), sender);
}

/// WebView2's picture of the webview `controller` holds, as PNG bytes through `sender`.
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
