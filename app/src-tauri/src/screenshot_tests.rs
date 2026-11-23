use super::*;

#[test]
fn a_png_s_size_comes_from_its_header() {
    let mut png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR".to_vec();
    png.extend_from_slice(&1440u32.to_be_bytes());
    png.extend_from_slice(&900u32.to_be_bytes());
    assert_eq!(png_size(&png), Ok((1440, 900)));
    assert!(png_size(&png[..23]).is_err());
    let mut jpeg = png.clone();
    jpeg[1] = b'J';
    assert_eq!(
        png_size(&jpeg),
        Err("the window's picture is not a PNG".into())
    );
    let mut chunk = png.clone();
    chunk[12] = b'X';
    assert!(png_size(&chunk).is_err());
}

#[tokio::test]
async fn a_window_that_is_not_open_is_said_so() {
    let app = tauri::test::mock_app();
    let refused = take(app.handle(), "plc").await.unwrap_err();
    assert_eq!(refused, "no plc window open");
}
