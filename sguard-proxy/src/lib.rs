pub mod proxy_trait;
pub mod state_machine;

pub struct Session {
    // /// the HTTP session to downstream (the client)
    // pub downstream_session: Box<HttpSession>,
    // /// The interface to control HTTP caching
    // pub cache: HttpCache,
    // /// (de)compress responses coming into the proxy (from upstream)
    // pub upstream_compression: ResponseCompressionCtx,
    // /// (de)compress responses leaving the proxy (to downstream)
    // pub downstream_compression: ResponseCompressionCtx,
    // /// ignore downstream range (skip downstream range filters)
    // pub ignore_downstream_range: bool,
    // // the context from parent request
    // subrequest_ctx: Option<Box<SubReqCtx>>,
}

impl Session {
    //     fn new(downstream_session: impl Into<Box<HttpSession>>) -> Self {
    //         Session {
    //             downstream_session: downstream_session.into(),
    //             cache: HttpCache::new(),
    //             upstream_compression: ResponseCompressionCtx::new(0, false), // disable both
    //             downstream_compression: ResponseCompressionCtx::new(0, false), // disable both
    //             ignore_downstream_range: false,
    //             subrequest_ctx: None,
    //         }
    //     }

    //     /// Create a new [Session] from the given [Stream]
    //     ///
    //     /// This function is mostly used for testing and mocking.
    //     pub fn new_h1(stream: Stream) -> Self {
    //         Self::new(Box::new(HttpSession::new_http1(stream)))
    //     }

    //     pub fn as_downstream_mut(&mut self) -> &mut HttpSession {
    //         &mut self.downstream_session
    //     }

    //     pub fn as_downstream(&self) -> &HttpSession {
    //         &self.downstream_session
    //     }
    // }

    // impl Session {
    //     async fn write_response_tasks(&mut self, mut tasks: Vec<HttpTask>) -> Result<bool> {
    //         // all built-in downstream response filters goes here
    //         // NOTE: if downstream_session is written directly (error page), the filters will be
    //         // bypassed.
    //         tasks
    //             .iter_mut()
    //             .for_each(|t| self.downstream_compression.response_filter(t));
    //         self.downstream_session.response_duplex_vec(tasks).await
    //     }
    // }

    // impl AsRef<HttpSession> for Session {
    //     fn as_ref(&self) -> &HttpSession {
    //         &self.downstream_session
    //     }
    // }

    // impl AsMut<HttpSession> for Session {
    //     fn as_mut(&mut self) -> &mut HttpSession {
    //         &mut self.downstream_session
    //     }
    // }

    // use std::ops::{Deref, DerefMut};

    // impl Deref for Session {
    //     type Target = HttpSession;

    //     fn deref(&self) -> &Self::Target {
    //         &self.downstream_session
    //     }
    // }

    // impl DerefMut for Session {
    //     fn deref_mut(&mut self) -> &mut Self::Target {
    //         &mut self.downstream_session
    //     }
    // }

    // // generic HTTP 502 response sent when proxy_upstream_filter refuses to connect to upstream
    // static BAD_GATEWAY: Lazy<ResponseHeader> = Lazy::new(|| {
    //     let mut resp = ResponseHeader::build(http::StatusCode::BAD_GATEWAY, Some(3)).unwrap();
    //     resp.insert_header(header::SERVER, &SERVER_NAME[..])
    //         .unwrap();
    //     resp.insert_header(header::CONTENT_LENGTH, 0).unwrap();
    //     resp.insert_header(header::CACHE_CONTROL, "private, no-store")
    //         .unwrap();

    //     resp
    // });

    // impl<SV> HttpProxy<SV> {
    //     async fn process_request(
    //         self: &Arc<Self>,
    //         mut session: Session,
    //         mut ctx: <SV as ProxyHttp>::CTX,
    //     ) -> Option<Stream>
    //     where
    //         SV: ProxyHttp + Send + Sync + 'static,
    //         <SV as ProxyHttp>::CTX: Send + Sync,
    //     {
    //         match self.inner.request_filter(&mut session, &mut ctx).await {
    //             Ok(response_sent) => {
    //                 if response_sent {
    //                     // TODO: log error
    //                     self.inner.logging(&mut session, None, &mut ctx).await;
    //                     return session.downstream_session.finish().await.ok().flatten();
    //                 }
    //                 /* else continue */
    //             }
    //             Err(e) => {
    //                 if !self.inner.suppress_error_log(&session, &ctx, &e) {
    //                     error!(
    //                         "Fail to filter request: {}, {}",
    //                         e,
    //                         self.inner.request_summary(&session, &ctx)
    //                     );
    //                 }
    //                 self.inner.fail_to_proxy(&mut session, &e, &mut ctx).await;
    //                 self.inner.logging(&mut session, Some(&e), &mut ctx).await;
    //                 return None;
    //             }
    //         }

    //         // all built-in downstream request filters go below

    //         session
    //             .downstream_compression
    //             .request_filter(session.downstream_session.req_header());

    //         if let Some((reuse, err)) = self.proxy_cache(&mut session, &mut ctx).await {
    //             // cache hit
    //             return self.finish(session, &mut ctx, reuse, err.as_deref()).await;
    //         }
    //         // either uncacheable, or cache miss

    //         // decide if the request is allowed to go to upstream
    //         match self
    //             .inner
    //             .proxy_upstream_filter(&mut session, &mut ctx)
    //             .await
    //         {
    //             Ok(proxy_to_upstream) => {
    //                 if !proxy_to_upstream {
    //                     // The hook can choose to write its own response, but if it doesn't, we respond
    //                     // with a generic 502
    //                     if session.response_written().is_none() {
    //                         match session.write_response_header_ref(&BAD_GATEWAY).await {
    //                             Ok(()) => {}
    //                             Err(e) => {
    //                                 if !self.inner.suppress_error_log(&session, &ctx, &e) {
    //                                     error!(
    //                                         "Error responding with Bad Gateway: {}, {}",
    //                                         e,
    //                                         self.inner.request_summary(&session, &ctx)
    //                                     );
    //                                 }
    //                                 self.inner.fail_to_proxy(&mut session, &e, &mut ctx).await;
    //                                 self.inner.logging(&mut session, Some(&e), &mut ctx).await;
    //                                 return None;
    //                             }
    //                         }
    //                     }

    //                     return self.finish(session, &mut ctx, false, None).await;
    //                 }
    //                 /* else continue */
    //             }
    //             Err(e) => {
    //                 if !self.inner.suppress_error_log(&session, &ctx, &e) {
    //                     error!(
    //                         "Error deciding if we should proxy to upstream: {}, {}",
    //                         e,
    //                         self.inner.request_summary(&session, &ctx)
    //                     );
    //                 }
    //                 self.inner.fail_to_proxy(&mut session, &e, &mut ctx).await;
    //                 self.inner.logging(&mut session, Some(&e), &mut ctx).await;
    //                 return None;
    //             }
    //         }

    //         let mut retries: usize = 0;

    //         let mut server_reuse = false;
    //         let mut proxy_error: Option<Box<Error>> = None;

    //         while retries < MAX_RETRIES {
    //             retries += 1;

    //             let (reuse, e) = self.proxy_to_upstream(&mut session, &mut ctx).await;
    //             server_reuse = reuse;

    //             match e {
    //                 Some(error) => {
    //                     let retry = error.retry();
    //                     proxy_error = Some(error);
    //                     if !retry {
    //                         break;
    //                     }
    //                     // only log error that will be retried here, the final error will be logged below
    //                     warn!(
    //                         "Fail to proxy: {}, tries: {}, retry: {}, {}",
    //                         proxy_error.as_ref().unwrap(),
    //                         retries,
    //                         retry,
    //                         self.inner.request_summary(&session, &ctx)
    //                     );
    //                 }
    //                 None => {
    //                     proxy_error = None;
    //                     break;
    //                 }
    //             };
    //         }

    //         // serve stale if error
    //         // Check both error and cache before calling the function because await is not cheap
    //         let serve_stale_result = if proxy_error.is_some() && session.cache.can_serve_stale_error() {
    //             self.handle_stale_if_error(&mut session, &mut ctx, proxy_error.as_ref().unwrap())
    //                 .await
    //         } else {
    //             None
    //         };

    //         let final_error = if let Some((reuse, stale_cache_error)) = serve_stale_result {
    //             // don't reuse server conn if serve stale polluted it
    //             server_reuse = server_reuse && reuse;
    //             stale_cache_error
    //         } else {
    //             proxy_error
    //         };

    //         if let Some(e) = final_error.as_ref() {
    //             let status = self.inner.fail_to_proxy(&mut session, e, &mut ctx).await;

    //             // final error will have > 0 status unless downstream connection is dead
    //             if !self.inner.suppress_error_log(&session, &ctx, e) {
    //                 error!(
    //                     "Fail to proxy: {}, status: {}, tries: {}, retry: {}, {}",
    //                     final_error.as_ref().unwrap(),
    //                     status,
    //                     retries,
    //                     false, // we never retry here
    //                     self.inner.request_summary(&session, &ctx)
    //                 );
    //             }
    //         }

    //         // logging() will be called in finish()
    //         self.finish(session, &mut ctx, server_reuse, final_error.as_deref())
    //             .await
    //     }
}
