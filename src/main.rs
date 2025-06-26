// Import our two modules from clipboard.rs and db.rs
mod clipboard;
mod db;

// Allows multiple threads to safely share access to the database
use std::sync::{Arc, Mutex};

// Import the notify_rust crate for desktop notifications
use notify_rust::Notification;

// Import Path to reference your icon file
use std::path::Path;

fn main() {
    println!("📋 ClipTrack started...");

    // Initialise the database (creates the .db file if it doesn't exist)
    let conn = db::init();

    // Wrap the database connection in Arc+Mutex so it can be safely shared across threads
    let conn = Arc::new(Mutex::new(conn));

    // Start polling the clipboard in a separate thread
    clipboard::start_polling({
        // Clone the Arc so the thread gets access to the DB
        let conn = Arc::clone(&conn);

        // This function gets called every time new clipboard text is detected
        move |text| {
            println!("New clipboard text: {}", text);

            // Lock the DB connection for writing, then save the clipboard entry
            let db = conn.lock().unwrap();
            db::save_clip(&db, &text);

            // Use an absolute path to your PNG or SVG icon
            let icon_path = Path::new("/home/khaishea/cliptrack/assets/cliptrack-icon.svg");

            // Show a desktop notification with the copied text and custom icon
            Notification::new()
                .summary("Copied to Clipboard")
                .body(&text)
                .icon(icon_path.to_str().unwrap())
                .show()
                .unwrap();
        }
    });

    // Keep the main thread alive forever (the polling runs in its own thread)
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}