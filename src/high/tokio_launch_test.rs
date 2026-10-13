use super::*;
use crate::http::{Endpoint, ParseError, TextResponse, client::RequestError};
use pem::Pem;
use std::sync::{Arc, Mutex as StdMutex};

#[derive(Debug, thiserror::Error)]
enum FakeError {
    #[error("host busy")]
    Busy,
    #[error("{0}")]
    Parse(ParseError),
}
impl TryFrom<FakeError> for ParseError {
    type Error = FakeError;
    fn try_from(error: FakeError) -> Result<Self, FakeError> {
        match error {
            FakeError::Parse(error) => Ok(error),
            other => Err(other),
        }
    }
}
impl RequestError for FakeError {
    fn is_connect(&self) -> bool {
        false
    }
    fn is_encryption(&self) -> bool {
        false
    }
}
#[derive(Clone, Default)]
struct FakeClient {
    paths: Arc<StdMutex<Vec<&'static str>>>,
    busy: bool,
}
impl RequestClient for FakeClient {
    type Error = FakeError;
    fn with_defaults() -> Result<Self, Self::Error> {
        Ok(Self::default())
    }
    fn with_defaults_long_timeout() -> Result<Self, Self::Error> {
        Self::with_defaults()
    }
    fn with_certificates(_: &Pem, _: &Pem, _: &Pem) -> Result<Self, Self::Error> {
        Self::with_defaults()
    }
    async fn send_http<E>(
        &self,
        _: ClientInfo<'_>,
        _: &str,
        _: &E::Request,
    ) -> Result<E::Response, Self::Error>
    where
        E: Endpoint,
        E::Request: Sync,
        E::Response: TextResponse<Err = ParseError>,
    {
        panic!("unexpected plaintext request")
    }
    async fn send_https<E>(
        &self,
        _: ClientInfo<'_>,
        _: &str,
        _: &E::Request,
    ) -> Result<E::Response, Self::Error>
    where
        E: Endpoint,
        E::Request: Sync,
        E::Response: TextResponse<Err = ParseError>,
    {
        self.paths.lock().unwrap().push(E::path());
        if self.busy && E::path() == "/launch" {
            return Err(FakeError::Busy);
        }
        r#"<root status_code="200"><gamesession>1</gamesession><resume>1</resume><sessionUrl0>rtsp://fake/session</sessionUrl0></root>"#.parse().map_err(FakeError::Parse)
    }
    async fn send_https_with_bytes<E>(
        &self,
        _: ClientInfo<'_>,
        _: &str,
        _: &E::Request,
    ) -> Result<E::Response, Self::Error>
    where
        E: Endpoint<Response = Vec<u8>>,
        E::Request: Sync,
    {
        panic!("unexpected bytes request")
    }
}
fn request() -> ClientStreamRequest {
    ClientStreamRequest {
        app_id: 7,
        mode_width: 1280,
        mode_height: 720,
        mode_fps: 30,
        sops: true,
        hdr: false,
        local_audio_play_mode: false,
        gamepads_attached_mask: 0,
        gamepads_persist_after_disconnect: false,
        ri_key: AesKey([0; 16]),
        ri_key_id: AesIv(0),
        additional_query_parameters: String::new(),
    }
}
#[tokio::test]
async fn launch_only_never_resumes_either_observed_or_racing_busy_host() {
    // current_game=0 plus a refused launch represents a host becoming busy after
    // observation; current_game=42 represents an already occupied host.
    for current_game in [0, 42] {
        let client = FakeClient {
            busy: true,
            ..Default::default()
        };
        let result = send_stream_request(
            &client,
            ClientInfo {
                unique_id: "signage",
                uuid: Uuid::new_v4(),
            },
            "fake:47984",
            &request(),
            current_game,
            false,
        )
        .await;
        assert!(result.is_err());
        assert_eq!(*client.paths.lock().unwrap(), vec!["/launch"]);
    }
}
#[tokio::test]
async fn exclusive_launch_and_legacy_resume_choose_expected_endpoint() {
    for (game, resume, path) in [
        (0, false, "/launch"),
        (42, false, "/launch"),
        (0, true, "/launch"),
        (42, true, "/resume"),
    ] {
        let client = FakeClient::default();
        let result = send_stream_request(
            &client,
            ClientInfo {
                unique_id: "test",
                uuid: Uuid::new_v4(),
            },
            "fake:47984",
            &request(),
            game,
            resume,
        )
        .await
        .unwrap();
        assert_eq!(result.as_deref(), Some("rtsp://fake/session"));
        assert_eq!(*client.paths.lock().unwrap(), vec![path]);
    }
}
