use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, Label, ListBox,
    ListBoxRow, Orientation, PolicyType, ScrolledWindow,
    STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use gtk4::{gdk::Display, pango};
use gtk4::style_context_add_provider_for_display;
use glib::{self, clone, ControlFlow};

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Receiver;

use crate::db;

pub fn launch_gui(rx: Receiver<()>) {
    let rx = Rc::new(RefCell::new(rx));

    let app = Application::builder()
        .application_id("com.khai.cliptrack")
        .build();

    app.connect_activate({
        let rx = rx.clone();
        move |app| {
            // Load CSS
            let provider = CssProvider::new();
            provider.load_from_path("style.css");
            style_context_add_provider_for_display(
                &Display::default().unwrap(),
                &provider,
                STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            let _conn = db::init_db().unwrap();

            let window = ApplicationWindow::builder()
                .application(app)
                .title("📋 ClipTrack")
                .default_width(420)
                .default_height(500)
                .build();

            let vbox = GtkBox::new(Orientation::Vertical, 8);
            vbox.set_margin_top(10);
            vbox.set_margin_bottom(10);
            vbox.set_margin_start(10);
            vbox.set_margin_end(10);

            let clear_button = Button::with_label("🧹 Clear History");
            clear_button.set_halign(gtk4::Align::End);

            let list_box = ListBox::new();
            list_box.set_selection_mode(gtk4::SelectionMode::None);
            let list_box_ref = Rc::new(RefCell::new(list_box));

            let scroller = ScrolledWindow::builder()
                .child(&*list_box_ref.borrow())
                .vexpand(true)
                .build();
            scroller.set_policy(PolicyType::Automatic, PolicyType::Never);

            vbox.append(&clear_button);
            vbox.append(&scroller);
            window.set_child(Some(&vbox));
            window.show();

            let update_list = {
                let list_box_ref = list_box_ref.clone();
                move || {
                    let conn = db::init_db().unwrap();
                    let clips = db::get_all_clips(&conn);
                    let list_box = &*list_box_ref.borrow();

                    while let Some(child) = list_box.first_child() {
                        list_box.remove(&child);
                    }

                    for (content, timestamp) in clips {
                        let row = ListBoxRow::new();
                        row.set_css_classes(&["clip-row"]);
                        row.set_hexpand(true);

                        let row_box = GtkBox::new(Orientation::Vertical, 4);
                        row_box.set_margin_top(8);
                        row_box.set_margin_bottom(8);
                        row_box.set_margin_start(8);
                        row_box.set_margin_end(8);
                        row_box.set_hexpand(true);

                        let label = Label::new(Some(&content));
                        label.set_wrap(true);
                        label.set_wrap_mode(pango::WrapMode::WordChar);
                        label.set_max_width_chars(60);
                        label.set_ellipsize(pango::EllipsizeMode::None);
                        label.set_xalign(0.0);
                        label.set_hexpand(true);

                        let ts = Label::new(Some(&timestamp));
                        ts.set_xalign(0.0);
                        ts.add_css_class("timestamp");

                        row_box.append(&label);
                        row_box.append(&ts);
                        row.set_child(Some(&row_box));

                        let content_clone = content.clone();
                        row.connect_activate(move |_| {
                            if let Some(display) = Display::default() {
                                let clipboard = display.clipboard();
                                clipboard.set_text(&content_clone);
                            }
                        });

                        list_box.append(&row);
                    }
                }
            };

            update_list();

            clear_button.connect_clicked({
                let list_box_ref = list_box_ref.clone();
                move |_| {
                    let conn = db::init_db().unwrap();
                    db::clear_history(&conn);
                    let list_box = &*list_box_ref.borrow();
                    while let Some(row) = list_box.first_child() {
                        list_box.remove(&row);
                    }
                }
            });

            // Poll every few seconds for updates
            glib::timeout_add_local(std::time::Duration::from_secs(3), {
                let update_list = update_list.clone();
                move || {
                    update_list();
                    ControlFlow::Continue
                }
            });

            // Update when receiver signals change
            let update_list_clone = update_list.clone();
            let rx_clone = rx.clone();
            glib::source::idle_add_local(move || {
                while rx_clone.borrow().try_recv().is_ok() {
                    update_list_clone();
                }
                ControlFlow::Continue
            });
        }
    });

    app.run();
}
