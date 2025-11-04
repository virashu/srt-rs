use std::{
    fmt::Write,
    fs,
    sync::{Arc, Mutex},
};

use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    http::{
        Response, StatusCode,
        header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE},
    },
    response::IntoResponse,
    routing::get,
};

const PLAYLIST_HEADER_VOD: &str = "#EXTM3U
#EXT-X-VERSION:3
#EXT-X-PLAYLIST-TYPE:VOD
#EXT-X-TARGETDURATION:2
#EXT-X-MEDIA-SEQUENCE:0

";

const PLAYLIST_HEADER_EVENT: &str = "#EXTM3U
#EXT-X-VERSION:3
#EXT-X-PLAYLIST-TYPE:EVENT
#EXT-X-TARGETDURATION:2
";

fn mock_playlist(n_segments: u32) -> String {
    let segments: String = (0..n_segments)
        .map(|n| format!("#EXTINF:2.000,\n/api/hls/segment/segment_{n}.mpg\n"))
        .collect();
    format!("{PLAYLIST_HEADER_VOD}{segments}\n#EXT-X-ENDLIST")
}

fn read_playlist(current_segment: u64, is_ended: bool) -> String {
    let mut res = String::from(PLAYLIST_HEADER_EVENT);
    writeln!(res, "#EXT-X-MEDIA-SEQUENCE:{current_segment}").unwrap();

    let mut ents: Vec<_> = fs::read_dir("_local")
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    ents.sort_by_key(|x| (x.file_name().len(), x.file_name()));

    for ent in ents.iter().skip(current_segment as usize) {
        res += "#EXTINF:2.000,\n";
        res += "/api/hls/segment/";
        res += ent.file_name().to_str().unwrap();
        res += "\n";
    }

    if is_ended {
        res += "#EXT-X-ENDLIST";
    }

    res
}

async fn get_playlist(State(state): State<AppState>) -> impl IntoResponse {
    let current_segment = { *state.current_segment.lock().unwrap() };
    let is_ended = { *state.is_ended.lock().unwrap() };

    Response::builder()
        .header(CONTENT_TYPE, "application/vnd.apple.mpegurl")
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(read_playlist(current_segment, is_ended))
        .unwrap()
}

async fn get_segment(Path(segment): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    let file_res = fs::read(format!("_local/{segment}"));

    if let Ok(file) = file_res {
        Ok(Response::builder()
            .header(CONTENT_TYPE, "application/octet-stream")
            .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .body(Body::from(file))
            .unwrap())
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[derive(Clone, Debug)]
struct AppState {
    pub current_segment: Arc<Mutex<u64>>,
    pub is_ended: Arc<Mutex<bool>>,
}

pub fn run(current_segment: Arc<Mutex<u64>>, is_ended: Arc<Mutex<bool>>) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let app = Router::new()
        .route("/api/hls/stream.m3u8", get(get_playlist))
        .route("/api/hls/segment/{segment}", get(get_segment))
        .with_state(AppState {
            current_segment,
            is_ended,
        });

    rt.block_on(async {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
}
