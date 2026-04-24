use std::collections::HashSet;

use gtk::prelude::*;
use gtk::{Box as GtkBox, Image, Label, ListBox, ListBoxRow};

use crate::models::*;

pub fn filter_and_sort_networks(mut networks: Vec<Network>) -> Vec<Network> {
    networks.sort_by(|a, b| b.in_use.cmp(&a.in_use).then(b.signal.cmp(&a.signal)));

    let mut seen_ssids = HashSet::new();
    let mut filtered = Vec::new();

    for net in networks {
        if net.ssid.is_empty() {
            continue;
        }
        if seen_ssids.insert(net.ssid.clone()) {
            filtered.push(net);
        }
    }
    filtered
}

pub fn update_network_list(list_box: &ListBox, networks: Vec<Network>) -> Vec<Network> {
    while let Some(row) = list_box.row_at_index(0) {
        list_box.remove(&row);
    }

    let filtered = filter_and_sort_networks(networks);

    for net in &filtered {
        let row = ListBoxRow::new();
        row.set_activatable(false);
        let hbox = GtkBox::new(gtk::Orientation::Horizontal, 10);
        hbox.set_margin_start(10);
        hbox.set_margin_end(10);
        hbox.set_margin_top(10);
        hbox.set_margin_bottom(10);

        // Signal/type icon
        let icon_name = if net.net_type == NetworkType::Ethernet {
            "network-wired-symbolic"
        } else if net.signal > 80 {
            "network-wireless-signal-excellent-symbolic"
        } else if net.signal > 55 {
            "network-wireless-signal-good-symbolic"
        } else if net.signal > 30 {
            "network-wireless-signal-ok-symbolic"
        } else {
            "network-wireless-signal-weak-symbolic"
        };
        let icon = Image::from_icon_name(icon_name);
        hbox.append(&icon);

        // SSID
        let label = Label::new(Some(&net.ssid));
        label.set_hexpand(true);
        label.set_halign(gtk::Align::Start);
        if net.in_use {
            label.set_markup(&format!(
                "<b>{}</b>",
                gtk::glib::markup_escape_text(&net.ssid)
            ));
        }
        hbox.append(&label);

        // Lock icon
        if !net.security.is_empty() && net.security != "--" {
            let lock_icon = Image::from_icon_name("network-wireless-encrypted-symbolic");
            hbox.append(&lock_icon);
        }

        // Signal percentage (colored)
        if net.net_type == NetworkType::WiFi && net.signal > 0 {
            let color = if net.signal > 70 {
                "#4caf50"
            } else if net.signal > 40 {
                "#ff9800"
            } else {
                "#f44336"
            };
            let sig_label = Label::new(None);
            sig_label.set_markup(&format!(
                "<span foreground=\"{}\" size=\"small\"><b>{}%</b></span>",
                color, net.signal
            ));
            sig_label.set_halign(gtk::Align::End);
            sig_label.set_width_chars(5);
            hbox.append(&sig_label);
        }

        // Connection checkmark
        if net.in_use {
            let check_icon = Image::from_icon_name("object-select-symbolic");
            hbox.append(&check_icon);
        }

        row.set_child(Some(&hbox));
        list_box.append(&row);
    }

    filtered
}

