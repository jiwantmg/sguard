use sguard_http::app::AppBuilder;

#[cfg(unix)]
fn raise_fd_limit() {
    unsafe {
        let mut rlim = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        if libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlim) == 0 {
            let target = rlim.rlim_max.min(1_048_576);
            if rlim.rlim_cur < target {
                let prev = rlim.rlim_cur;
                rlim.rlim_cur = target;
                if libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) == 0 {
                    log::info!("Raised file descriptor limit from {} to {}", prev, target);
                } else {
                    log::warn!("Failed to raise file descriptor limit from {} to {}", prev, target);
                }
            } else {
                log::info!("Current file descriptor limit is {}", rlim.rlim_cur);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default())
        .expect("failed to load log4rs.yaml");

    #[cfg(unix)]
    raise_fd_limit();

    let mut app = AppBuilder::new();
    app.app_builder();

    let Err(err) = app.run().await;
    log::error!("gateway stopped: {}", err);
    std::process::exit(1);
}