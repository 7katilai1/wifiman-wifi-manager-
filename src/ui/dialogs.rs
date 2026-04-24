use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{
    ApplicationWindow, Box as GtkBox, Button, Label, ListBox, PasswordEntry, Spinner,
};

use crate::models::Network;
use crate::nmcli;
use crate::ui::network_list::update_network_list;

/// Refresh the list
pub async fn refresh_list(list_box: &ListBox, networks_data: &Rc<RefCell<Vec<Network>>>) {
    if let Ok(networks) = nmcli::get_networks().await {
        let filtered = update_network_list(list_box, networks);
        *networks_data.borrow_mut() = filtered;
    }
}

/// Password entry dialog
pub fn show_password_dialog(
    parent: &ApplicationWindow,
    ssid: &str,
    spinner: Spinner,
    list_box: ListBox,
    networks_data: Rc<RefCell<Vec<Network>>>,
) {
    let dialog = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(format!("Connect to {}", ssid))
        .default_width(300)
        .build();

    let vbox = GtkBox::new(gtk::Orientation::Vertical, 10);
    vbox.set_margin_start(10);
    vbox.set_margin_end(10);
    vbox.set_margin_top(10);
    vbox.set_margin_bottom(10);

    let label = Label::new(Some(&format!("Password for {}:", ssid)));
    vbox.append(&label);

    let password_entry = PasswordEntry::new();
    password_entry.set_show_peek_icon(true);
    vbox.append(&password_entry);

    let hbox = GtkBox::new(gtk::Orientation::Horizontal, 10);
    hbox.set_halign(gtk::Align::End);
    let cancel_btn = Button::with_label("Cancel");
    let connect_btn = Button::with_label("Connect");
    hbox.append(&cancel_btn);
    hbox.append(&connect_btn);
    vbox.append(&hbox);

    dialog.set_child(Some(&vbox));

    let ssid_clone = ssid.to_string();

    cancel_btn.connect_clicked({
        let dialog = dialog.clone();
        move |_| dialog.close()
    });

    connect_btn.connect_clicked({
        let dialog = dialog.clone();
        let password_entry = password_entry.clone();
        let spinner = spinner.clone();
        let ssid = ssid_clone.clone();
        move |_| {
            let password = password_entry.text().to_string();
            if password.is_empty() {
                return;
            }
            dialog.close();
            spinner.start();

            let ssid_local = ssid.clone();
            let spinner_local = spinner.clone();
            let list_box2 = list_box.clone();
            let nd2 = networks_data.clone();

            let dialog_clone = dialog.clone();
            gtk::glib::MainContext::default().spawn_local(async move {
                if let Err(e) = nmcli::connect_to_network(&ssid_local, Some(&password), None).await {
                    show_error_dialog(Some(&dialog_clone.into()), &format!("Failed to connect:\n{}", e));
                }
                refresh_list(&list_box2, &nd2).await;
                spinner_local.stop();
            });
        }
    });

    // Enter → Connect
    password_entry.connect_activate({
        let connect_btn = connect_btn.clone();
        move |_| connect_btn.emit_clicked()
    });

    dialog.present();
}

/// Confirmation dialog (for deleting connections etc.)
pub fn show_confirm_dialog(
    parent: &impl IsA<gtk::Window>,
    title: &str,
    message: &str,
    on_confirm: impl Fn() + 'static,
) {
    let dialog = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(title)
        .default_width(300)
        .build();

    let vbox = GtkBox::new(gtk::Orientation::Vertical, 10);
    vbox.set_margin_start(16);
    vbox.set_margin_end(16);
    vbox.set_margin_top(16);
    vbox.set_margin_bottom(16);

    let label = Label::new(Some(message));
    label.set_wrap(true);
    vbox.append(&label);

    let hbox = GtkBox::new(gtk::Orientation::Horizontal, 10);
    hbox.set_halign(gtk::Align::End);
    let cancel_btn = Button::with_label("Cancel");
    let confirm_btn = Button::with_label("Confirm");
    confirm_btn.add_css_class("destructive-action");
    hbox.append(&cancel_btn);
    hbox.append(&confirm_btn);
    vbox.append(&hbox);

    dialog.set_child(Some(&vbox));

    cancel_btn.connect_clicked({
        let dialog = dialog.clone();
        move |_| dialog.close()
    });

    confirm_btn.connect_clicked({
        let dialog = dialog.clone();
        move |_| {
            dialog.close();
            on_confirm();
        }
    });

    dialog.present();
}

/// Error message display dialog
pub fn show_error_dialog(parent: Option<&gtk::Window>, message: &str) {
    let dialog = gtk::Window::builder()
        .modal(true)
        .title("Error")
        .default_width(320)
        .build();
    
    if let Some(p) = parent {
        dialog.set_transient_for(Some(p));
    }

    let vbox = GtkBox::new(gtk::Orientation::Vertical, 12);
    vbox.set_margin_start(16);
    vbox.set_margin_end(16);
    vbox.set_margin_top(16);
    vbox.set_margin_bottom(16);

    let error_label = Label::new(None);
    error_label.set_markup(&format!(
        "<span foreground=\"#e74c3c\"><b>{}</b></span>",
        gtk::glib::markup_escape_text(message)
    ));
    error_label.set_wrap(true);
    error_label.set_max_width_chars(40);
    vbox.append(&error_label);

    let hbox = GtkBox::new(gtk::Orientation::Horizontal, 0);
    hbox.set_halign(gtk::Align::End);
    let close_btn = Button::with_label("Close");
    hbox.append(&close_btn);
    vbox.append(&hbox);

    dialog.set_child(Some(&vbox));

    close_btn.connect_clicked({
        let dialog = dialog.clone();
        move |_| dialog.close()
    });

    dialog.present();
}
