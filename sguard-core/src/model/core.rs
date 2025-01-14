use http::{request::Parts, HeaderMap, Method, Request, Response, StatusCode, Uri};
use http_body::{Body,Frame, SizeHint};
use hyper::body::Incoming;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use pin_project_lite::pin_project;
use futures;
use bytes::Bytes;

pub struct HttpRequest {
    pub method: Method,
    pub uri: Uri,
    pub headers: HeaderMap,
    pub incoming: Option<Incoming>,
    pub parts: Option<Parts>,
}

impl HttpRequest {
    pub fn new() -> Self {
        HttpRequest {
            method: Method::GET,
            uri: Uri::default(),
            headers: HeaderMap::new(),
            incoming: None,
            parts: None,
        }
    }
    pub async fn from_hyper_request(req: Request<Incoming>) -> Result<HttpRequest, hyper::Error> {
        let (parts, incoming) = req.into_parts(); // Split into parts and body
        let parts_copy = parts.clone();
        let request = HttpRequest {
            method: parts.method,
            uri: parts.uri,
            headers: parts.headers,
            //body: incoming.collect().await?.to_bytes(),
            incoming: Some(incoming),
            parts: Some(parts_copy),
        };
        Ok(request)
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