pub trait ProxyHttp {
    /// The per request object to share state across the different filters
    type CTX;
    /// Define how the `ctx` should be created.
    fn new_ctx(&self) -> Self::CTX;
    // Define where the proxy should send the request to.
    //
    // The returned [HttpPeer] contains the information regarding where and how this request should
    // be forwarded to.
    // async fn upstream_peer(
    //     &self,
    //     session: &mut Session,
    //     ctx: &mut Self::CTX,
    // ) -> Result<Box<HttpPeer>>;
}
