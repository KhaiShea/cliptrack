mod gui;
mod db;
mod clipboard;

use std::sync::{Arc, Mutex, mpsc};
use notify_rust::Notification;

fn main() {
    println!("📋 ClipTrack started...");

    let conn = db::init_db().unwrap();
    let conn = Arc::new(Mutex::new(conn));

    let (tx, rx) = mpsc::channel();

    clipboard::start_polling({
        let conn = Arc::clone(&conn);
        let tx = tx.clone();

        move |text| {
            println!("📎 Copied: {}", text);

            let db = conn.lock().unwrap();
            db::insert_clip(&db, &text);

            // Notify GUI to update
            let _ = tx.send(());

            // Optional system tray notification
            let _ = Notification::new()
                .summary("📋 ClipTrack")
                .body(&text)
                .icon("edit-copy")
                .show();
        }
    });

    gui::launch_gui(rx);
}
