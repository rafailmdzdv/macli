use macli::app::{Application, Macli};

#[tokio::main]
async fn main() {
    Macli::new().run_cmd().await.unwrap();
}
