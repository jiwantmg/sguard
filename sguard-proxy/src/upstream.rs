use std::io::Error;

use bytes::Bytes;
use hyper::Response;
use hyper_util::rt::TokioIo;
use sguard_core::model::context::RequestContext;
use sguard_error::{Error as AppError, ErrorType};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

#[derive(Clone, Default)]
pub struct UpstreamService {}

impl UpstreamService {
    pub async fn call_upstream_service(
        &self,
        req: &mut RequestContext,
    ) -> Result<Response<hyper::body::Incoming>, AppError> {
        let uri_result = req.route_definition.uri.parse::<hyper::Uri>();
        if uri_result.is_err() {
            return Err(AppError::new(ErrorType::Custom("Invalid url"))); // Early return on error
        }

        let uri = uri_result.unwrap();
        let host = uri.host().expect("uri has no host");
        let port = uri.port_u16().unwrap_or(80);
        let address = format!("{}:{}", host, port);
        // Open a TCP connection to the remote host
        let stream = TcpStream::connect(address).await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);
        // Create the Hyper client
        log::trace!("Handshaking to remote server");
        let handshake = hyper::client::conn::http1::handshake(io);
        let (mut sender, conn) =
            timeout(Duration::from_secs(3), handshake)
                .await?
                .map_err(|_| {
                    Error::new(
                        std::io::ErrorKind::TimedOut,
                        "Upstream service not available",
                    )
                })?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });
        // if let (Some(incoming), Some(parts)) = (req.request.incoming.take(), req.request.parts) {

        // }

        let body = req.request.incoming.take().unwrap();
        let parts = req.request.parts.take().unwrap();
        let request_body = hyper::Request::from_parts(parts, body);
        let response = sender.send_request(request_body).await;
        match response {
            Ok(response) => {
                //let (parts, body) = response.into_parts();

                // Create a new body from the bytes

                // Return response with the reconstructed body
                Ok(response)
            }
            Err(err) => Err(AppError::new(ErrorType::ConnectTimeout)),
        }
    }
}
