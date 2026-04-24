use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{
    Box as GtkBox, Button, DropDown, Entry, Label, ListBox, Notebook, ScrolledWindow, Spinner,
    StringList, Switch,
};

use crate::models::Network;
use crate::nmcli;
use crate::ui::dialogs::refresh_list;
use crate::ui::widgets::{form_row, section_label};

const IPV4_METHODS: &[&str] = &["auto", "manual", "disabled", "link-local", "shared"];
const IPV6_METHODS: &[&str] = &["auto", "manual", "ignore", "disabled", "link-local"];

/// Connection editing window
pub async fn show_editor_window(
    parent: Option<&gtk::Window>,
    uuid: &str,
    spinner: Spinner,
    list_box: ListBox,
    networks_data: Rc<RefCell<Vec<Network>>>,
) {
    let details = match nmcli::get_connection_details(uuid).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to load connection: {}", e);
            return;
        }
    };

    let window = gtk::Window::builder()
        .title(format!("Edit — {}", details.name))
        .default_width(480)
        .default_height(560)
        .modal(true)
        .build();
    if let Some(p) = parent {
        window.set_transient_for(Some(p));
    }

    let notebook = Notebook::new();

    // ════════════════════════════════════════════════════════════
    // TAB 1 — General
    // ════════════════════════════════════════════════════════════
    let general_box = GtkBox::new(gtk::Orientation::Vertical, 4);
    general_box.set_margin_start(12);
    general_box.set_margin_end(12);
    general_box.set_margin_top(12);
    general_box.set_margin_bottom(12);

    let name_entry = Entry::new();
    name_entry.set_text(&details.name);
    general_box.append(&form_row("Connection Name", &name_entry));

    let autoconnect_switch = Switch::new();
    autoconnect_switch.set_active(details.autoconnect);
    autoconnect_switch.set_halign(gtk::Align::Start);
    general_box.append(&form_row("Autoconnect", &autoconnect_switch));

    let iface_entry = Entry::new();
    iface_entry.set_text(&details.interface_name);
    iface_entry.set_placeholder_text(Some("e.g. wlan0, eth0"));
    general_box.append(&form_row("Interface", &iface_entry));

    general_box.append(&section_label("Device"));

    let mtu_entry = Entry::new();
    mtu_entry.set_text(&details.mtu);
    mtu_entry.set_placeholder_text(Some("auto"));
    general_box.append(&form_row("MTU", &mtu_entry));

    let mac_entry = Entry::new();
    mac_entry.set_text(&details.cloned_mac);
    mac_entry.set_placeholder_text(Some("AA:BB:CC:DD:EE:FF"));
    general_box.append(&form_row("Cloned MAC", &mac_entry));

    notebook.append_page(&general_box, Some(&Label::new(Some("General"))));

    // ════════════════════════════════════════════════════════════
    // TAB 2 — IPv4
    // ════════════════════════════════════════════════════════════
    let (ipv4_page, ipv4_method_dd, ipv4_addr_entry, ipv4_gw_entry, ipv4_dns_entry) =
        build_ip_tab("IPv4", IPV4_METHODS, &details.ipv4_method,
            &details.ipv4_addresses, &details.ipv4_gateway, &details.ipv4_dns);
    notebook.append_page(&ipv4_page, Some(&Label::new(Some("IPv4"))));

    // ════════════════════════════════════════════════════════════
    // TAB 3 — IPv6
    // ════════════════════════════════════════════════════════════
    let (ipv6_page, ipv6_method_dd, ipv6_addr_entry, ipv6_gw_entry, ipv6_dns_entry) =
        build_ip_tab("IPv6", IPV6_METHODS, &details.ipv6_method,
            &details.ipv6_addresses, &details.ipv6_gateway, &details.ipv6_dns);
    notebook.append_page(&ipv6_page, Some(&Label::new(Some("IPv6"))));

    // ════════════════════════════════════════════════════════════
    // Bottom buttons
    // ════════════════════════════════════════════════════════════
    let main_box = GtkBox::new(gtk::Orientation::Vertical, 0);
    main_box.append(&notebook);

    let btn_box = GtkBox::new(gtk::Orientation::Horizontal, 10);
    btn_box.set_halign(gtk::Align::End);
    btn_box.set_margin_start(12);
    btn_box.set_margin_end(12);
    btn_box.set_margin_top(8);
    btn_box.set_margin_bottom(12);

    let cancel_btn = Button::with_label("Cancel");
    let save_btn = Button::with_label("Save");
    save_btn.add_css_class("suggested-action");
    btn_box.append(&cancel_btn);
    btn_box.append(&save_btn);
    main_box.append(&btn_box);

    window.set_child(Some(&main_box));

    cancel_btn.connect_clicked({
        let window = window.clone();
        move |_| window.close()
    });

    let uuid_owned = uuid.to_string();
    let con_type = details.con_type.clone();

    save_btn.connect_clicked({
        let window = window.clone();
        move |_| {
            let uuid = uuid_owned.clone();
            let con_type = con_type.clone();
            let name = name_entry.text().to_string();
            let autoconnect = autoconnect_switch.is_active();
            let iface = iface_entry.text().to_string();
            let mtu = mtu_entry.text().to_string();
            let mac = mac_entry.text().to_string();

            let ipv4_method = get_dropdown_value(&ipv4_method_dd, IPV4_METHODS);
            let ipv4_addr = ipv4_addr_entry.text().to_string();
            let ipv4_gw = ipv4_gw_entry.text().to_string();
            let ipv4_dns = ipv4_dns_entry.text().to_string();

            let ipv6_method = get_dropdown_value(&ipv6_method_dd, IPV6_METHODS);
            let ipv6_addr = ipv6_addr_entry.text().to_string();
            let ipv6_gw = ipv6_gw_entry.text().to_string();
            let ipv6_dns = ipv6_dns_entry.text().to_string();

            let spinner = spinner.clone();
            let list_box = list_box.clone();
            let networks_data = networks_data.clone();

            window.close();
            spinner.start();

            gtk::glib::MainContext::default().spawn_local(async move {
                let mut settings: Vec<(&str, String)> = Vec::new();

                settings.push(("connection.id", name));
                settings.push((
                    "connection.autoconnect",
                    if autoconnect { "yes" } else { "no" }.to_string(),
                ));
                if !iface.is_empty() {
                    settings.push(("connection.interface-name", iface));
                }

                // MTU
                let mtu_key = if con_type.contains("wireless") {
                    "802-11-wireless.mtu"
                } else {
                    "802-3-ethernet.mtu"
                };
                settings.push((mtu_key, if mtu.is_empty() { "0".into() } else { mtu }));

                // Cloned MAC
                let mac_key = if con_type.contains("wireless") {
                    "802-11-wireless.cloned-mac-address"
                } else {
                    "802-3-ethernet.cloned-mac-address"
                };
                if !mac.is_empty() {
                    settings.push((mac_key, mac));
                }

                // IPv4
                settings.push(("ipv4.method", ipv4_method));
                if !ipv4_addr.is_empty() {
                    settings.push(("ipv4.addresses", ipv4_addr));
                } else {
                    settings.push(("ipv4.addresses", "".into()));
                }
                if !ipv4_gw.is_empty() {
                    settings.push(("ipv4.gateway", ipv4_gw));
                } else {
                    settings.push(("ipv4.gateway", "".into()));
                }
                if !ipv4_dns.is_empty() {
                    settings.push(("ipv4.dns", ipv4_dns));
                } else {
                    settings.push(("ipv4.dns", "".into()));
                }

                // IPv6
                settings.push(("ipv6.method", ipv6_method));
                if !ipv6_addr.is_empty() {
                    settings.push(("ipv6.addresses", ipv6_addr));
                } else {
                    settings.push(("ipv6.addresses", "".into()));
                }
                if !ipv6_gw.is_empty() {
                    settings.push(("ipv6.gateway", ipv6_gw));
                } else {
                    settings.push(("ipv6.gateway", "".into()));
                }
                if !ipv6_dns.is_empty() {
                    settings.push(("ipv6.dns", ipv6_dns));
                } else {
                    settings.push(("ipv6.dns", "".into()));
                }

                let refs: Vec<(&str, &str)> =
                    settings.iter().map(|(k, v)| (*k, v.as_str())).collect();
                if let Err(e) = nmcli::modify_connection(&uuid, &refs).await {
                    crate::ui::dialogs::show_error_dialog(None, &format!("Failed to save connection:\n{}", e));
                }
                refresh_list(&list_box, &networks_data).await;
                spinner.stop();
            });
        }
    });

    window.present();
}

/// Create IP configuration tab (shared for IPv4/IPv6)
fn build_ip_tab(
    title: &str,
    methods: &[&str],
    current_method: &str,
    current_addresses: &[String],
    current_gateway: &str,
    current_dns: &[String],
) -> (ScrolledWindow, DropDown, Entry, Entry, Entry) {
    let vbox = GtkBox::new(gtk::Orientation::Vertical, 4);
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);

    vbox.append(&section_label(&format!("{} Configuration", title)));

    // Method dropdown
    let string_list = StringList::new(methods);
    let method_dd = DropDown::new(Some(string_list), gtk::Expression::NONE);
    let default_idx = methods
        .iter()
        .position(|m| *m == current_method)
        .unwrap_or(0);
    method_dd.set_selected(default_idx as u32);
    vbox.append(&form_row("Method", &method_dd));

    // Addresses
    let addr_entry = Entry::new();
    addr_entry.set_text(&current_addresses.join(", "));
    addr_entry.set_placeholder_text(Some("192.168.1.100/24"));
    vbox.append(&form_row("Addresses", &addr_entry));

    // Gateway
    let gw_entry = Entry::new();
    gw_entry.set_text(current_gateway);
    gw_entry.set_placeholder_text(Some("192.168.1.1"));
    vbox.append(&form_row("Gateway", &gw_entry));

    // DNS
    let dns_entry = Entry::new();
    dns_entry.set_text(&current_dns.join(", "));
    dns_entry.set_placeholder_text(Some("8.8.8.8, 1.1.1.1"));
    vbox.append(&form_row("DNS Servers", &dns_entry));

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .child(&vbox)
        .build();

    (scrolled, method_dd, addr_entry, gw_entry, dns_entry)
}

fn get_dropdown_value(dd: &DropDown, options: &[&str]) -> String {
    let idx = dd.selected() as usize;
    options.get(idx).unwrap_or(&"auto").to_string()
}
