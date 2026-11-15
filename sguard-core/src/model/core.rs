use std::ops::{Deref, DerefMut};
use bytes::Bytes;
use http::{request::Parts, HeaderMap, Request, Response, StatusCode};
use hyper::body::Incoming;
use pin_project_lite::pin_project;

pub struct HttpRequest {
    pub parts: Parts,
    pub incoming: Option<Incoming>,
}

impl HttpRequest {
    pub fn new() -> Self {
        let (parts, _) = Request::new(()).into_parts();
        HttpRequest {
            parts,
            incoming: None,
        }
    }

    pub fn take_parts(&mut self) -> Parts {
        let (empty, _) = Request::new(()).into_parts();
        std::mem::replace(&mut self.parts, empty)
    }

    pub async fn from_hyper_request(req: Request<Incoming>) -> Result<HttpRequest, hyper::Error> {
        let (parts, incoming) = req.into_parts();
        Ok(HttpRequest {
            parts,
            incoming: Some(incoming),
        })
    }
}

impl Default for HttpRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for HttpRequest {
    type Target = Parts;
    fn deref(&self) -> &Self::Target {
        &self.parts
    }
}

impl DerefMut for HttpRequest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parts
    }
}

pin_project! {
    pub struct HttpResponse {
        pub status: StatusCode,
        pub headers: HeaderMap,
        #[pin]
        body: Option<Response<Incoming>>,
        is_end: bool,
        buffer: Option<Bytes>,
    }
}

impl Default for HttpResponse {
    fn default() -> Self {
        HttpResponse {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: None,
            is_end: false,
            buffer: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{header::AUTHORIZATION, HeaderValue, Method, Request};

    #[test]
    fn test_http_request_deref_and_take() {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/test")
            .header(AUTHORIZATION, "Bearer test_token")
            .body(())
            .unwrap();
        let (parts, _) = req.into_parts();
        let mut http_req = HttpRequest {
            parts,
            incoming: None,
        };

        // Deref access
        assert_eq!(http_req.method, Method::POST);
        assert_eq!(http_req.uri.path(), "/api/test");
        assert_eq!(
            http_req.headers.get(AUTHORIZATION),
            Some(&HeaderValue::from_static("Bearer test_token"))
        );

        // Take parts without cloning
        let taken = http_req.take_parts();
        assert_eq!(taken.method, Method::POST);
        assert_eq!(taken.uri.path(), "/api/test");

        // http_req.parts is now replaced with an empty parts
        assert_eq!(http_req.headers.len(), 0);
    }
}

// impl HttpResponse {
//     pub fn new(status: StatusCode, headers: HeaderMap, body: Response<Incoming>) -> Self {
//         HttpResponse {
//             status,
//             headers,
//             body: Some(body),
//             is_end: false,
//             buffer: None,
//         }
//     }
//     pub fn empty(status: StatusCode) -> Self {
//         HttpResponse {
//             status,
//             headers: HeaderMap::new(),
//             body: None,
//             is_end: true,
//             buffer: None,
//         }
//     }
//     pub fn with_bytes(status: StatusCode, headers: HeaderMap, bytes: Bytes) -> Self {
//         HttpResponse {
//             status,
//             headers,
//             body: None,
//             is_end: false,
//             buffer: Some(bytes),
//         }
//     }
//     pub async fn aggregate_body(&mut self) -> Result<Bytes, std::io::Error> {
//         if let Some(response) = self.body.take() {
//             let (_, body) = response.into_parts();
//             let mut bytes = Vec::new();
//             let mut body = Pin::new(Box::new(body));

//             loop {
//                 let mut cx = std::task::Context::from_waker(
//                     futures::task::noop_waker_ref()
//                 );

//                 match Body::poll_frame(Pin::new(&mut body), &mut cx) {
//                     Poll::Ready(Some(Ok(frame))) => {
//                         if let Some(data) = frame.into_data() {
//                             bytes.extend_from_slice(&data);
//                         }
//                     }
//                     Poll::Ready(None) => break,
//                     Poll::Ready(Some(Err(e))) => {
//                         return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
//                     }
//                     Poll::Pending => {
//                         // In a real async context, we would yield here
//                         tokio::task::yield_now().await;
//                     }
//                 }
//             }

//             Ok(Bytes::from(bytes))
//         } else if let Some(bytes) = self.buffer.take() {
//             Ok(bytes)
//         } else {
//             Ok(Bytes::new())
//         }
//     }
// }

// // Implementing `hyper::body::Body` for `HttpResponse`
// impl Body for HttpResponse {
//     type Data = Bytes;
//     type Error = std::io::Error;

//     fn poll_frame(
//         self: Pin<&mut Self>,
//         cx: &mut Context<'_>,
//     ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
//         let mut this = self.project();

//         if *this.is_end {
//             return Poll::Ready(None);
//         }

//         // If we have buffered data, send it
//         if let Some(bytes) = this.buffer.take() {
//             *this.is_end = true;
//             return Poll::Ready(Some(Ok(Frame::data(bytes))));
//         }

//         // If we have a streaming body, forward its frames
//         if let Some(response) = this.body.as_mut().get_mut() {
//             let incoming = response.body();
//             let result = match Pin::new(incoming).poll_frame(cx) {
//                 Poll::Ready(Some(Ok(frame))) => {
//                     if let Some(data) = frame.data_ref() {
//                         // Clone the data since we can't take ownership of data_ref
//                         let bytes = data.clone();
//                         Poll::Ready(Some(Ok(Frame::data(bytes))))
//                     } else if let Some(trailers) = frame.trailers_ref() {
//                         // Handle trailers if present
//                         Poll::Ready(Some(Ok(Frame::trailers(trailers.clone()))))
//                     } else {
//                         Poll::Ready(None)
//                     }
//                 }
//                 Poll::Ready(Some(Err(e))) => {
//                     Poll::Ready(Some(Err(std::io::Error::new(std::io::ErrorKind::Other, e))))
//                 }
//                 Poll::Ready(None) => {
//                     *this.is_end = true;
//                     Poll::Ready(None)
//                 }
//                 Poll::Pending => Poll::Pending,
//             };
//             result
//         } else {
//             *this.is_end = true;
//             Poll::Ready(None)
//         }
//     }

//     fn size_hint(&self) -> SizeHint {
//         if let Some(bytes) = &self.buffer {
//             SizeHint::with_exact(bytes.len() as u64)
//         } else if let Some(response) = &self.body {
//             response.body().size_hint()
//         } else {
//             SizeHint::with_exact(0)
//         }
//     }
// }
