use zbus::{dbus_interface, zvariant};

// Enable clippy lints.
#[warn(clippy::all, clippy::pedantic)]

/// `StrMap` is a `HashMap` of &str to `zvariant::Value`.
type StrMap<'a> = std::collections::HashMap<&'a str, zvariant::Value<'a>>;

/// FileChooser implements the org.freedesktop.impl.portal.FileChooser interface.
struct FileChooser {}

#[dbus_interface(name = "org.freedesktop.impl.portal.FileChooser")]
impl FileChooser {
    /// Presents a file chooser dialog to the user to open one or more files.
    #[dbus_interface(out_args("response", "results"))]
    async fn open_file(
        &self,
        handle: zvariant::ObjectPath<'_>,
        _app_id: &str,
        _parent_window: &str,
        title: &str,
        options: StrMap<'_>,
    ) -> zbus::fdo::Result<(u32, StrMap<'_>)> {
        log::info!("open_file({}, {})", handle, title);

        let multiple = matches!(options.get("multiple"), Some(zvariant::Value::Bool(true)));

        let directory = matches!(options.get("directory"), Some(zvariant::Value::Bool(true)));

        let dialog = rfd::FileDialog::new().set_title(title);

        if multiple {
            let choices = match directory {
                false => dialog.pick_files(),
                true => dialog.pick_folders(),
            };

            match choices {
                Some(paths) => {
                    let uris = pathbuf_to_file_uri(paths)
                        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

                    let mut results = StrMap::new();

                    results.insert("uris", zvariant::Array::from(uris).into());

                    zbus::fdo::Result::Ok((0, results))
                }

                None => zbus::fdo::Result::Ok((1, StrMap::new())),
            }
        } else {
            let choice = match directory {
                false => dialog.pick_file(),
                true => dialog.pick_folder(),
            };

            match choice {
                Some(path) => {
                    let uris = pathbuf_to_file_uri(vec![path])
                        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

                    let mut results = StrMap::new();

                    results.insert("uris", zvariant::Array::from(uris).into());

                    zbus::fdo::Result::Ok((0, results))
                }

                None => zbus::fdo::Result::Ok((1, StrMap::new())),
            }
        }
    }

    /// Presents a file chooser dialog to the user to save a file.
    #[dbus_interface(out_args("response", "results"))]
    async fn save_file(
        &self,
        handle: zvariant::ObjectPath<'_>,
        _app_id: &str,
        _parent_window: &str,
        title: &str,
        options: StrMap<'_>,
    ) -> zbus::fdo::Result<(u32, StrMap<'_>)> {
        log::info!("save_file({}, {})", handle, title);

        if let Some(zvariant::Value::Bool(true)) = options.get("multiple") {
            return zbus::fdo::Result::Err(zbus::fdo::Error::NotSupported(String::from(
                "multiple save not supported",
            )));
        };

        let mut dialog = rfd::FileDialog::new().set_title(title);

        if let Some(zvariant::Value::Str(current_name)) = options.get("current_name") {
            dialog = dialog.set_file_name(current_name);
        }

        match dialog.save_file() {
            Some(path) => {
                let uris = pathbuf_to_file_uri(vec![path])
                    .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

                let mut results = StrMap::new();

                results.insert("uris", zvariant::Array::from(uris).into());

                zbus::fdo::Result::Ok((0, results))
            }

            None => zbus::fdo::Result::Ok((1, StrMap::new())),
        }
    }

    /// Asks for a folder as a location to save one or more files.
    #[dbus_interface(out_args("response", "results"))]
    async fn save_files(
        &self,
        handle: zvariant::ObjectPath<'_>,
        _app_id: &str,
        _parent_window: &str,
        title: &str,
        _options: StrMap<'_>,
    ) -> zbus::fdo::Result<(u32, StrMap<'_>)> {
        log::info!("save_files({}, {})", handle, title);

        match rfd::FileDialog::new().set_title(title).pick_folder() {
            Some(path) => {
                let uris = pathbuf_to_file_uri(vec![path])
                    .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

                let mut results = StrMap::new();

                results.insert("uris", zvariant::Array::from(uris).into());

                zbus::fdo::Result::Ok((0, results))
            }

            None => zbus::fdo::Result::Ok((1, StrMap::new())),
        }
    }
}

/// Convert one or more PathBuf to URI file strings.
fn pathbuf_to_file_uri(paths: Vec<std::path::PathBuf>) -> Result<Vec<String>, http::Error> {
    log::debug!("pathbuf_to_uri({:?})", paths);

    paths
        .iter()
        .map(|path| {
            http::Uri::builder()
                .scheme("file")
                .authority("localhost")
                .path_and_query(path.to_string_lossy().as_ref())
                .build()
                .map(|uri| uri.to_string())
        })
        .collect()
}

#[tokio::main]
async fn main() -> zbus::Result<()> {
    env_logger::init();

    let _conn = zbus::ConnectionBuilder::session()?
        .name("org.freedesktop.impl.portal.desktop.rs")?
        .serve_at("/org/freedesktop/portal/desktop", FileChooser {})?
        .build()
        .await?;

    std::future::pending::<()>().await;

    Ok(())
}
