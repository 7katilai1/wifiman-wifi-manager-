use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, ScrolledWindow};

use crate::nmcli;

/// Window showing connection details
pub async fn show_details_window(parent: Option<&gtk::Window>, uuid: &str) {
    let details = match nmcli::get_connection_details(uuid).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to get details: {}", e);
            return;
        }
    };

    let window = gtk::Window::builder()
        .title(format!("Details — {}", details.name))
        .default_width(420)
        .default_height(500)
        .modal(true)
        .build();
    if let Some(p) = parent {
        window.set_transient_for(Some(p));
    }

    let vbox = GtkBox::new(gtk::Orientation::Vertical, 0);
    vbox.set_margin_start(16);
    vbox.set_margin_end(16);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);

    // ─── General ───
    add_section(&vbox, "General");
    add_detail_row(&vbox, "Name", &details.name);
    add_detail_row(&vbox, "UUID", &details.uuid);
    add_detail_row(&vbox, "Type", &details.con_type);
    add_detail_row(&vbox, "Interface", &details.interface_name);
    add_detail_row(
        &vbox,
        "Autoconnect",
        if details.autoconnect { "Yes" } else { "No" },
    );

    if !details.mtu.is_empty() {
        add_detail_row(&vbox, "MTU", &details.mtu);
    }
    if !details.cloned_mac.is_empty() {
        add_detail_row(&vbox, "Cloned MAC", &details.cloned_mac);
    }

    // ─── IPv4 ───
    add_section(&vbox, "IPv4");
    add_detail_row(&vbox, "Method", &details.ipv4_method);
    if !details.ipv4_addresses.is_empty() {
        add_detail_row(&vbox, "Addresses", &details.ipv4_addresses.join(", "));
    }
    if !details.ipv4_gateway.is_empty() {
        add_detail_row(&vbox, "Gateway", &details.ipv4_gateway);
    }
    if !details.ipv4_dns.is_empty() {
        add_detail_row(&vbox, "DNS", &details.ipv4_dns.join(", "));
    }

    // ─── IPv6 ───
    add_section(&vbox, "IPv6");
    add_detail_row(&vbox, "Method", &details.ipv6_method);
    if !details.ipv6_addresses.is_empty() {
        add_detail_row(&vbox, "Addresses", &details.ipv6_addresses.join(", "));
    }
    if !details.ipv6_gateway.is_empty() {
        add_detail_row(&vbox, "Gateway", &details.ipv6_gateway);
    }
    if !details.ipv6_dns.is_empty() {
        add_detail_row(&vbox, "DNS", &details.ipv6_dns.join(", "));
    }

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .child(&vbox)
        .build();

    window.set_child(Some(&scrolled));
    window.present();
}

fn add_section(container: &GtkBox, title: &str) {
    let label = Label::new(None);
    label.set_markup(&format!("<b>{}</b>", gtk::glib::markup_escape_text(title)));
    label.set_halign(gtk::Align::Start);
    label.set_margin_top(12);
    label.set_margin_bottom(4);
    label.add_css_class("detail-section");
    container.append(&label);

    let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
    container.append(&sep);
}

fn add_detail_row(container: &GtkBox, key: &str, value: &str) {
    let row = GtkBox::new(gtk::Orientation::Horizontal, 12);
    row.set_margin_start(8);
    row.set_margin_top(4);
    row.set_margin_bottom(4);

    let key_label = Label::new(Some(key));
    key_label.set_halign(gtk::Align::Start);
    key_label.set_width_chars(14);
    key_label.set_xalign(0.0);
    key_label.add_css_class("detail-key");
    row.append(&key_label);

    let val_label = Label::new(Some(if value.is_empty() { "—" } else { value }));
    val_label.set_halign(gtk::Align::Start);
    val_label.set_hexpand(true);
    val_label.set_wrap(true);
    val_label.set_selectable(true);
    row.append(&val_label);

    container.append(&row);
}
