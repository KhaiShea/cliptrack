mod clipboard;
mod db;
mod gui;

use std::sync::{Arc, Mutex, mpsc};
use notify_rust::Notification;

fn main() {
    println!("📋 ClipTrack started...");

    let conn = db::init();
    let conn = Arc::new(Mutex::new(conn));

    // Channel to send clipboard update notifications to the GUI
    let (tx, rx) = mpsc::channel();

    // Start clipboard polling
    clipboard::start_polling({
        let conn = Arc::clone(&conn);
        let tx = tx.clone(); // move a clone into the thread

        move |text| {
            println!("New clipboard text: {}", text);
            let db = conn.lock().unwrap();
            db::save_clip(&db, &text);

            // Send update signal to GUI
            let _ = tx.send(());

            // Optional desktop notification
            Notification::new()
                .summary("Copied to Clipboard")
                .body(&text)
                .icon("/home/khaishea/cliptrack/assets/cliptrack-icon.png")
                .show()
                .unwrap();
        }
    });

    // Launch the GUI and pass the receiver
    gui::launch_gui(rx);
}
