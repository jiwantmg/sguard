use crate::model::context::RequestContext;
use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::Response;
use sguard_error::Error;
use core::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;

/// A Filter represents a top level contract that must be shared by all the
/// filters in the application
pub trait Filter: Send + Sync {
    fn handle(&self, req: &mut RequestContext, next: FilterFn) -> FilterRs;
}

pub type FilterFn = Arc<dyn Fn(&mut RequestContext) -> FilterRs + Send + Sync>;

/// Same as FilterFn, but used for Response instead
pub type FilterRs = Pin<Box<dyn Future<Output = Result<Response<ResponseBody>, Error>> + Send>>;

// Generic Response Body Type
pub enum ResponseBody {
    Full(Full<Bytes>),
    Incoming(Response<Incoming>),
}

// Specify the associated types correctly
impl hyper::body::Body for ResponseBody {
    type Data = Bytes;
    type Error = hyper::Error;
    
    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        match self.get_mut() {
            ResponseBody::Incoming(incoming) => Pin::new(incoming).poll_frame(cx),
            ResponseBody::Full(full) => {
                 // Map `Infallible` error to `hyper::Error`
                match Pin::new(full).poll_frame(cx) {
                    Poll::Ready(Some(Ok(frame))) => Poll::Ready(Some(Ok(frame))),
                    Poll::Ready(None) => Poll::Ready(None),  // No more data
                    Poll::Pending => Poll::Pending,          // Still waiting for data
                    Poll::Ready(Some(Err(_))) => {
                        // `Full<Bytes>` doesn't error, so this case should be treated as handled
                        Poll::Ready(None)  // No more data, effectively complete
                    },
                }
            },
        }
    }
}

impl fmt::Debug for ResponseBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResponseBody::Full(_) => f.debug_tuple("Full").finish(),
            ResponseBody::Incoming(_) => f.debug_tuple("Incoming").finish(),
        }
    }
}

// Implement fmt::Display for ResponseBody
impl fmt::Display for ResponseBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResponseBody::Full(_) => write!(f, "Full<Bytes>"),
            ResponseBody::Incoming(_) => write!(f, "Incoming"),
        }
    }
}

// Implement the Error trait for ResponseBody
impl std::error::Error for ResponseBody {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}