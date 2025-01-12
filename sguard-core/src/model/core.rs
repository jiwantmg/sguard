use http::{HeaderMap, Method, Request, StatusCode, Uri};
use http_body::{Frame, SizeHint};
use http_body_util::BodyExt;
use hyper::body::{Bytes, Incoming};
use std::{pin::Pin, task::{Context, Poll}};


#[derive(Default)]
pub struct HttpRequest {
    pub method: Method,
    pub uri: Uri,
    pub headers: HeaderMap,
    pub body: Bytes,
}

impl HttpRequest {
    pub async fn from_hyper_request(req: Request<Incoming>) -> Result<HttpRequest, hyper::Error> {
        let (parts, body) = req.into_parts(); // Split into parts and body 
        let request = HttpRequest {
            method: parts.method,
            uri: parts.uri,
            headers: parts.headers,
            body: body.collect().await?.to_bytes(),
        };
        Ok(request)
    }
}


#[derive(Default)]
pub struct HttpResponse {
    pub status: http::StatusCode,
    pub headers: http::HeaderMap,
    pub body: Vec<u8>, 
}


impl HttpResponse {
    pub fn new(status: StatusCode, headers: HeaderMap, body: Vec<u8>) -> Self {
        HttpResponse { status, headers, body }
    }
}

// Implementing `hyper::body::Body` for `HttpResponse`
impl http_body::Body for HttpResponse {
    type Data = Bytes;
    type Error = std::io::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        // let this = self.project();

        // if *this.is_end {
        //     return Poll::Ready(None);
        // }

        // if let Some(data) = this.data.take() {
        //     *this.is_end = true;
        //     Poll::Ready(Some(Ok(Frame::data(data))))
        // } else {
        //     *this.is_end = true;
        //     Poll::Ready(None)
        // }
        Poll::Ready(None)
    }

    fn size_hint(&self) -> SizeHint {
        // match &self.data {
        //     Some(data) => {
        //         let size = data.len();
        //         SizeHint::with_exact(size as u64)
        //     }
        //     None => SizeHint::with_exact(0),
        // }
        SizeHint::with_exact(0)
    }
}