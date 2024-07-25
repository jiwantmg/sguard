use log4rs;
use sguard_http::app::AppBuilder;
#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let mut app = AppBuilder::new();
    app.app_builder();
    app.run().await;
}
