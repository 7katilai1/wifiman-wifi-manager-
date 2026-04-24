use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, Entry, Label};

use crate::nmcli;

/// Hostname settings dialog
pub fn show_hostname_dialog(parent: Option<&gtk::Window>) {
    let window = gtk::Window::builder()
        .title("Set Hostname")
        .default_width(360)
        .default_height(160)
        .modal(true)
        .build();
    if let Some(p) = parent {
        window.set_transient_for(Some(p));
    }

    let vbox = GtkBox::new(gtk::Orientation::Vertical, 10);
    vbox.set_margin_start(16);
    vbox.set_margin_end(16);
    vbox.set_margin_top(16);
    vbox.set_margin_bottom(16);

    let current_label = Label::new(Some("Loading..."));
    current_label.set_halign(gtk::Align::Start);
    vbox.append(&current_label);

    let entry = Entry::new();
    entry.set_placeholder_text(Some("New hostname"));
    vbox.append(&entry);

    let btn_box = GtkBox::new(gtk::Orientation::Horizontal, 10);
    btn_box.set_halign(gtk::Align::End);
    let cancel_btn = Button::with_label("Cancel");
    let save_btn = Button::with_label("Set");
    save_btn.add_css_class("suggested-action");
    btn_box.append(&cancel_btn);
    btn_box.append(&save_btn);
    vbox.append(&btn_box);

    window.set_child(Some(&vbox));

    // Load current hostname
    let current_label_clone = current_label.clone();
    let entry_clone = entry.clone();
    gtk::glib::MainContext::default().spawn_local(async move {
        if let Ok(hostname) = nmcli::get_hostname().await {
            current_label_clone.set_markup(&format!(
                "Current hostname: <b>{}</b>",
                gtk::glib::markup_escape_text(&hostname)
            ));
            entry_clone.set_text(&hostname);
        }
    });

    cancel_btn.connect_clicked({
        let window = window.clone();
        move |_| window.close()
    });

    save_btn.connect_clicked({
        let window = window.clone();
        let entry = entry.clone();
        let current_label = current_label.clone();
        move |_| {
            let new_hostname = entry.text().to_string();
            if new_hostname.is_empty() {
                return;
            }
            // RFC 1123 hostname validation
            let is_valid = new_hostname.len() <= 253
                && new_hostname
                    .split('.')
                    .all(|label| {
                        !label.is_empty()
                            && label.len() <= 63
                            && !label.starts_with('-')
                            && !label.ends_with('-')
                            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                    });
            if !is_valid {
                current_label.set_markup(
                    "<span foreground=\"#e74c3c\">Invalid hostname (a-z, 0-9, - only, max 253 chars)</span>",
                );
                return;
            }
            let current_label = current_label.clone();
            let window = window.clone();
            gtk::glib::MainContext::default().spawn_local(async move {
                match nmcli::set_hostname(&new_hostname).await {
                    Ok(_) => {
                        current_label.set_markup(&format!(
                            "Current hostname: <b>{}</b>",
                            gtk::glib::markup_escape_text(&new_hostname)
                        ));
                        window.close();
                    }
                    Err(e) => {
                        current_label.set_markup(&format!(
                            "<span foreground=\"#e74c3c\">Error: {}</span>",
                            gtk::glib::markup_escape_text(&e.to_string())
                        ));
                    }
                }
            });
        }
    });

    // Enter → Set
    entry.connect_activate({
        let save_btn = save_btn.clone();
        move |_| save_btn.emit_clicked()
    });

    window.present();
}
