// Use the arboard crate to access the system clipboard
use arboard::Clipboard;

// For timing and thread sleeping
use std::time::Duration;
use std::thread;

/// Starts a loop in a separate thread that checks the clipboard every second
/// If it finds new text, it calls the given callback function with that text
pub fn start_polling<F>(mut callback: F)
where
    // The callback must be a function that takes a String, can be sent to a thread, and lives forever
    F: FnMut(String) + Send + 'static,
{
    // Spawn a new thread so we don't block the main program
    thread::spawn(move || {
        // Try to access the system clipboard
        let mut clipboard = Clipboard::new().expect("Failed to access clipboard");

        // Store the last seen clipboard text so we don't save duplicates
        let mut last = String::new();

        loop {
            // Try to read the current clipboard text
            if let Ok(current) = clipboard.get_text() {
                // If it's different from the last one, it's new!
                if current != last {
                    // Call the callback with the new text
                    last = current.clone();
                    callback(current);
                }
            }

            // Sleep for 100ms (0.1 second) before checking again
            thread::sleep(Duration::from_millis(100));
        }
    });
}