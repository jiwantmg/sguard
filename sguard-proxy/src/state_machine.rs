use crate::upstream::{FailureKind, UpstreamService};
use hyper::{body::Incoming, Response};
use sguard_core::model::context::RequestContext;
use sguard_error::{Error as AppError, ErrorType};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::{Semaphore, SemaphorePermit};

/// Max requests in the "connect + send + wait for response headers" phase per route at once.
const ROUTE_INFLIGHT_UPSTREAM: usize = 4096;
/// Max requests allowed to wait for a permit per route. Beyond this the gateway sheds load (503)
/// immediately instead of queueing without limit.
const MAX_QUEUED: usize = 8192;
/// Max time a request may wait for a permit before it is shed.
const QUEUE_TIMEOUT: Duration = Duration::from_secs(2);

struct Admission {
    sem: Semaphore,
    waiting: AtomicUsize,
    rejected: AtomicU64,
}

struct WaitGuard<'a>(&'a AtomicUsize);
impl Drop for WaitGuard<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Admission {
    fn new(inflight: usize) -> Self {
        Self {
            sem: Semaphore::new(inflight),
            waiting: AtomicUsize::new(0),
            rejected: AtomicU64::new(0),
        }
    }

    async fn acquire(&self) -> Result<SemaphorePermit<'_>, AppError> {
        // Fast path: no waiting, no counters touched
        if let Ok(permit) = self.sem.try_acquire() {
            return Ok(permit);
        }
        let queued = self.waiting.fetch_add(1, Ordering::Relaxed);
        let _guard = WaitGuard(&self.waiting);

        if queued >= MAX_QUEUED {
            self.rejected.fetch_add(1, Ordering::Relaxed);
            return Err(overloaded());
        }
        match tokio::time::timeout(QUEUE_TIMEOUT, self.sem.acquire()).await {
            Ok(Ok(permit)) => Ok(permit),
            _ => {
                self.rejected.fetch_add(1, Ordering::Relaxed);
                Err(overloaded())
            }
        }
    }
}

fn overloaded() -> AppError {
    AppError::new(ErrorType::Custom(String::from("Upstream overloaded"))) // -> 503
}

pub struct StateMachineManager {
    /// One client and connection pool for the whole gateway.
    upstream: Arc<UpstreamService>,
    bulkheads: RwLock<HashMap<String, Arc<Admission>>>,
    default_admission: Arc<Admission>,
}

impl StateMachineManager {
    /// Create ONCE at startup (inside the tokio runtime).
    pub fn new() -> Self {
        let manager = Self {
            upstream: Arc::new(UpstreamService::new()),
            bulkheads: RwLock::new(HashMap::new()),
            default_admission: Arc::new(Admission::new(ROUTE_INFLIGHT_UPSTREAM)),
        };
        manager.spawn_reporter();
        manager
    }

    fn get_admission(&self, route_id: &str) -> Arc<Admission> {
        if route_id.is_empty() {
            return self.default_admission.clone();
        }
        {
            let read = self.bulkheads.read().unwrap();
            if let Some(adm) = read.get(route_id) {
                return adm.clone();
            }
        }
        let mut write = self.bulkheads.write().unwrap();
        write
            .entry(route_id.to_string())
            .or_insert_with(|| Arc::new(Admission::new(ROUTE_INFLIGHT_UPSTREAM)))
            .clone()
    }

    pub async fn forward(&self, req: &mut RequestContext) -> Result<Response<Incoming>, AppError> {
        let admission = self.get_admission(&req.route_definition.id);
        let _permit = admission.acquire().await?;
        log::trace!("Forwarding to upstream for route {}", req.route_definition.id);
        self.upstream.call_upstream_service(req).await
        // permit released here, when the response headers have arrived
    }

    /// Logs one summary line every 10 s, but only when something went wrong.
    /// This is what tells you the real root cause without a debugger.
    fn spawn_reporter(&self) {
        let Ok(handle) = tokio::runtime::Handle::try_current() else {
            return;
        };
        let upstream = self.upstream.clone();
        handle.spawn(async move {
            let mut last = upstream.stats.snapshot();
            let mut last_retries = 0u64;
            let mut tick = tokio::time::interval(Duration::from_secs(10));
            tick.tick().await;
            loop {
                tick.tick().await;
                let now = upstream.stats.snapshot();
                let retries = upstream.stats.retries.load(Ordering::Relaxed);

                let mut parts: Vec<String> = Vec::new();
                for kind in FailureKind::ALL {
                    let delta = now[kind as usize] - last[kind as usize];
                    if delta > 0 {
                        parts.push(format!("{:?}={}", kind, delta));
                    }
                }
                if retries > last_retries {
                    parts.push(format!("retried={}", retries - last_retries));
                }
                if !parts.is_empty() {
                    log::warn!("upstream last 10s: {}", parts.join(" "));
                }
                last = now;
                last_retries = retries;
            }
        });
    }
}

impl Default for StateMachineManager {
    fn default() -> Self {
        Self::new()
    }
}