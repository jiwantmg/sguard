use sguard_http::app::AppBuilder;

#[tokio::main]
async fn main() {
    let mut app = AppBuilder::new();
    app.app_builder();
    app.run().await;
}
