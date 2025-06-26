use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, CssProvider, GestureClick, Label, ListBox, ListBoxRow,
    Orientation, ScrolledWindow, StyleContext, STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use gtk4::{glib, pango};
use gtk4::{Align, SelectionMode};
use gtk4::gdk::Display;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Receiver;

use glib::clone;
use crate::db;

/// Launches the GTK GUI clipboard history viewer
pub fn launch_gui(rx: Receiver<()>) {
    let rx = Rc::new(RefCell::new(rx));

    let app = Application::builder()
        .application_id("com.khaishea.cliptrack")
        .build();

    app.connect_activate(move |app| {
        // Load custom CSS (does not return Result)
        let provider = CssProvider::new();
        provider.load_from_path("style.css");
        StyleContext::add_provider_for_display(
            &Display::default().unwrap(),
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let win = ApplicationWindow::builder()
            .application(app)
            .title("ClipTrack Clipboard History")
            .default_width(500)
            .default_height(600)
            .build();

        let vbox = Box::new(Orientation::Vertical, 5);

        let list_box = ListBox::new();
        list_box.set_selection_mode(SelectionMode::None);
        let list_box_ref = Rc::new(RefCell::new(list_box));

        // Function to refresh the clipboard list
        let update_list = {
            let list_box_ref = list_box_ref.clone();
            move || {
                let conn = db::init();
                let mut list_box = list_box_ref.borrow_mut();

                while let Some(child) = list_box.first_child() {
                    list_box.remove(&child);
                }

                let mut stmt = conn
                    .prepare("SELECT content, copied_at FROM clipboard ORDER BY copied_at DESC LIMIT 100")
                    .unwrap();

                let entries = stmt
                    .query_map([], |row| {
                        let content: String = row.get(0)?;
                        let copied_at: String = row.get(1)?;
                        Ok((content, copied_at))
                    })
                    .unwrap();

                for item in entries {
                    if let Ok((content, timestamp)) = item {
                        let row = ListBoxRow::new();
                        row.set_margin_top(6);
                        row.set_margin_bottom(6);
                        row.set_margin_start(6);
                        row.set_margin_end(6);
                        row.set_css_classes(&["entry-row"]);

                        let entry_box = Box::new(Orientation::Vertical, 2);
                        entry_box.set_halign(Align::Start);
                        entry_box.set_valign(Align::Center);

                        let timestamp_label = Label::new(Some(&timestamp));
                        timestamp_label.set_css_classes(&["timestamp"]);
                        timestamp_label.set_halign(Align::Start);

                        let content_label = Label::new(Some(&content));
                        content_label.set_wrap(true);
                        content_label.set_wrap_mode(pango::WrapMode::WordChar);
                        content_label.set_selectable(true);
                        content_label.set_halign(Align::Start);

                        entry_box.append(&timestamp_label);
                        entry_box.append(&content_label);
                        row.set_child(Some(&entry_box));

                        // Copy to clipboard when row is clicked
                        let copy_text = content.clone();
                        let gesture = GestureClick::new();
                        gesture.connect_pressed(move |_, _, _, _| {
                            if let Some(display) = Display::default() {
                                let clipboard = display.clipboard();
                                clipboard.set_text(&copy_text);
                            }
                        });
                        row.add_controller(gesture);

                        list_box.append(&row);
                        row.show();
                    }
                }
            }
        };

        // First render
        update_list();

        // Watch for refresh signal
        {
            let list_box_ref = list_box_ref.clone();
            glib::source::idle_add_local(clone!(@strong rx => move || {
                while rx.borrow_mut().try_recv().is_ok() {
                    update_list();
                }
                glib::ControlFlow::Continue
            }));
        }

        // Scroller wrapping the list
        let scroller = ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .min_content_height(400)
            .build();

        scroller.set_child(Some(&*list_box_ref.borrow()));
        vbox.append(&scroller);
        win.set_child(Some(&vbox));
        win.show();
    });

    app.run();
}
