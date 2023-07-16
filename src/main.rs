mod service;

#[warn(clippy::all)]
#[warn(clippy::pedantic)]
#[warn(clippy::nursery)]
#[tokio::main]
async fn main() -> zbus::Result<()> {
    env_logger::init();

    let _conn = zbus::ConnectionBuilder::session()?
        .name("org.freedesktop.impl.portal.desktop.rs")?
        .serve_at("/org/freedesktop/portal/desktop", service::FileChooser {})?
        .build()
        .await?;

    std::future::pending::<()>().await;

    Ok(())
}
