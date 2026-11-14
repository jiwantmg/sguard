use sguard_http::app::AppBuilder;

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default())
        .expect("failed to load log4rs.yaml");

    let mut app = AppBuilder::new();
    app.app_builder();

    let Err(err) = app.run().await;
    log::error!("gateway stopped: {}", err);
    std::process::exit(1);
}