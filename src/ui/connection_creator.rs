use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{
    Box as GtkBox, Button, DropDown, Entry, Label, ListBox, Notebook, PasswordEntry,
    ScrolledWindow, Spinner, StringList, Switch,
};

use crate::models::Network;
use crate::nmcli;
use crate::ui::dialogs::refresh_list;
use crate::ui::widgets::{form_row, section_label};

const CON_TYPES: &[&str] = &["ethernet", "wifi", "bond", "bridge", "vlan"];
const CON_TYPE_LABELS: &[&str] = &["Ethernet", "Wi-Fi", "Bond", "Bridge", "VLAN"];
const BOND_MODES: &[&str] = &[
    "balance-rr",
    "active-backup",
    "balance-xor",
    "broadcast",
    "802.3ad",
    "balance-tlb",
    "balance-alb",
];
const IPV4_METHODS: &[&str] = &["auto", "manual", "disabled", "link-local", "shared"];
const IPV6_METHODS: &[&str] = &["auto", "manual", "ignore", "disabled", "link-local"];

/// New connection creation window
pub fn show_creator_window(
    parent: Option<&gtk::Window>,
    spinner: Spinner,
    list_box: ListBox,
    networks_data: Rc<RefCell<Vec<Network>>>,
) {
    let window = gtk::Window::builder()
        .title("New Connection")
        .default_width(480)
        .default_height(560)
        .modal(true)
        .build();
    if let Some(p) = parent {
        window.set_transient_for(Some(p));
    }

    let main_box = GtkBox::new(gtk::Orientation::Vertical, 0);

    // ─── Type & Name ───
    let top_box = GtkBox::new(gtk::Orientation::Vertical, 4);
    top_box.set_margin_start(12);
    top_box.set_margin_end(12);
    top_box.set_margin_top(12);

    let type_list = StringList::new(CON_TYPE_LABELS);
    let type_dd = DropDown::new(Some(type_list), gtk::Expression::NONE);
    type_dd.set_selected(0);
    top_box.append(&form_row("Type", &type_dd));

    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("Connection Name"));
    top_box.append(&form_row("Name", &name_entry));

    main_box.append(&top_box);

    // ─── Type-specific fields (dynamic) ───
    let dynamic_box = GtkBox::new(gtk::Orientation::Vertical, 4);
    dynamic_box.set_margin_start(12);
    dynamic_box.set_margin_end(12);
    main_box.append(&dynamic_box);

    // Wi-Fi fields
    let wifi_section = GtkBox::new(gtk::Orientation::Vertical, 4);
    wifi_section.append(&section_label("Wi-Fi"));
    let ssid_entry = Entry::new();
    ssid_entry.set_placeholder_text(Some("Network SSID"));
    wifi_section.append(&form_row("SSID", &ssid_entry));
    let wifi_pass = PasswordEntry::new();
    wifi_pass.set_show_peek_icon(true);
    wifi_section.append(&form_row("Password", &wifi_pass));

    // Bond fields
    let bond_section = GtkBox::new(gtk::Orientation::Vertical, 4);
    bond_section.append(&section_label("Bond"));
    let bond_mode_list = StringList::new(BOND_MODES);
    let bond_mode_dd = DropDown::new(Some(bond_mode_list), gtk::Expression::NONE);
    bond_section.append(&form_row("Mode", &bond_mode_dd));

    // VLAN fields
    let vlan_section = GtkBox::new(gtk::Orientation::Vertical, 4);
    vlan_section.append(&section_label("VLAN"));
    let vlan_parent_entry = Entry::new();
    vlan_parent_entry.set_placeholder_text(Some("eth0"));
    vlan_section.append(&form_row("Parent Device", &vlan_parent_entry));
    let vlan_id_entry = Entry::new();
    vlan_id_entry.set_placeholder_text(Some("100"));
    vlan_section.append(&form_row("VLAN ID", &vlan_id_entry));

    // Bridge fields (minimal, nmcli handles defaults)
    let bridge_section = GtkBox::new(gtk::Orientation::Vertical, 4);
    bridge_section.append(&section_label("Bridge"));
    let bridge_stp_switch = Switch::new();
    bridge_stp_switch.set_active(true);
    bridge_stp_switch.set_halign(gtk::Align::Start);
    bridge_section.append(&form_row("STP", &bridge_stp_switch));

    // Show/hide type sections
    dynamic_box.append(&wifi_section);
    dynamic_box.append(&bond_section);
    dynamic_box.append(&vlan_section);
    dynamic_box.append(&bridge_section);

    let update_visibility = {
        let wifi_section = wifi_section.clone();
        let bond_section = bond_section.clone();
        let vlan_section = vlan_section.clone();
        let bridge_section = bridge_section.clone();
        move |idx: u32| {
            wifi_section.set_visible(idx == 1);
            bond_section.set_visible(idx == 2);
            bridge_section.set_visible(idx == 3);
            vlan_section.set_visible(idx == 4);
        }
    };
    update_visibility(0); // Ethernet default

    type_dd.connect_selected_notify({
        let update_visibility = update_visibility.clone();
        move |dd| {
            update_visibility(dd.selected());
        }
    });

    // ─── IP Configuration (Notebook) ───
    let notebook = Notebook::new();
    notebook.set_margin_start(12);
    notebook.set_margin_end(12);
    notebook.set_margin_top(8);

    // IPv4 tab
    let ipv4_box = GtkBox::new(gtk::Orientation::Vertical, 4);
    ipv4_box.set_margin_start(8);
    ipv4_box.set_margin_end(8);
    ipv4_box.set_margin_top(8);
    ipv4_box.set_margin_bottom(8);

    let ipv4_method_list = StringList::new(IPV4_METHODS);
    let ipv4_method_dd = DropDown::new(Some(ipv4_method_list), gtk::Expression::NONE);
    ipv4_box.append(&form_row("Method", &ipv4_method_dd));
    let ipv4_addr = Entry::new();
    ipv4_addr.set_placeholder_text(Some("192.168.1.100/24"));
    ipv4_box.append(&form_row("Address", &ipv4_addr));
    let ipv4_gw = Entry::new();
    ipv4_gw.set_placeholder_text(Some("192.168.1.1"));
    ipv4_box.append(&form_row("Gateway", &ipv4_gw));
    let ipv4_dns = Entry::new();
    ipv4_dns.set_placeholder_text(Some("8.8.8.8, 1.1.1.1"));
    ipv4_box.append(&form_row("DNS", &ipv4_dns));

    notebook.append_page(&ipv4_box, Some(&Label::new(Some("IPv4"))));

    // IPv6 tab
    let ipv6_box = GtkBox::new(gtk::Orientation::Vertical, 4);
    ipv6_box.set_margin_start(8);
    ipv6_box.set_margin_end(8);
    ipv6_box.set_margin_top(8);
    ipv6_box.set_margin_bottom(8);

    let ipv6_method_list = StringList::new(IPV6_METHODS);
    let ipv6_method_dd = DropDown::new(Some(ipv6_method_list), gtk::Expression::NONE);
    ipv6_box.append(&form_row("Method", &ipv6_method_dd));
    let ipv6_addr = Entry::new();
    ipv6_addr.set_placeholder_text(Some("fe80::1/64"));
    ipv6_box.append(&form_row("Address", &ipv6_addr));
    let ipv6_gw = Entry::new();
    ipv6_gw.set_placeholder_text(Some("fe80::1"));
    ipv6_box.append(&form_row("Gateway", &ipv6_gw));
    let ipv6_dns = Entry::new();
    ipv6_dns.set_placeholder_text(Some("2001:4860:4860::8888"));
    ipv6_box.append(&form_row("DNS", &ipv6_dns));

    notebook.append_page(&ipv6_box, Some(&Label::new(Some("IPv6"))));

    main_box.append(&notebook);

    // ─── Autoconnect ───
    let auto_box = GtkBox::new(gtk::Orientation::Vertical, 4);
    auto_box.set_margin_start(12);
    auto_box.set_margin_end(12);
    auto_box.set_margin_top(8);
    let autoconnect_switch = Switch::new();
    autoconnect_switch.set_active(true);
    autoconnect_switch.set_halign(gtk::Align::Start);
    auto_box.append(&form_row("Autoconnect", &autoconnect_switch));
    main_box.append(&auto_box);

    // ─── Buttons ───
    let btn_box = GtkBox::new(gtk::Orientation::Horizontal, 10);
    btn_box.set_halign(gtk::Align::End);
    btn_box.set_margin_start(12);
    btn_box.set_margin_end(12);
    btn_box.set_margin_top(8);
    btn_box.set_margin_bottom(12);

    let cancel_btn = Button::with_label("Cancel");
    let create_btn = Button::with_label("Create");
    create_btn.add_css_class("suggested-action");
    btn_box.append(&cancel_btn);
    btn_box.append(&create_btn);
    main_box.append(&btn_box);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .child(&main_box)
        .build();
    window.set_child(Some(&scrolled));

    cancel_btn.connect_clicked({
        let window = window.clone();
        move |_| window.close()
    });

    create_btn.connect_clicked({
        let window = window.clone();
        move |_| {
            let type_idx = type_dd.selected() as usize;
            let con_type = CON_TYPES.get(type_idx).unwrap_or(&"ethernet");
            let name = name_entry.text().to_string();
            if name.is_empty() {
                return;
            }

            let mut settings: Vec<(String, String)> = Vec::new();

            // Autoconnect
            settings.push((
                "connection.autoconnect".into(),
                if autoconnect_switch.is_active() {
                    "yes"
                } else {
                    "no"
                }
                .into(),
            ));

            // Type-specific
            match type_idx {
                1 => {
                    // Wi-Fi
                    let ssid = ssid_entry.text().to_string();
                    if !ssid.is_empty() {
                        settings.push(("ssid".into(), ssid));
                    }
                    let pass = wifi_pass.text().to_string();
                    if !pass.is_empty() {
                        settings.push(("wifi-sec.key-mgmt".into(), "wpa-psk".into()));
                        settings.push(("wifi-sec.psk".into(), pass));
                    }
                }
                2 => {
                    // Bond
                    let mode_idx = bond_mode_dd.selected() as usize;
                    let mode = BOND_MODES.get(mode_idx).unwrap_or(&"balance-rr");
                    settings.push(("bond.options".into(), format!("mode={}", mode)));
                }
                3 => {
                    // Bridge
                    settings.push((
                        "bridge.stp".into(),
                        if bridge_stp_switch.is_active() {
                            "yes"
                        } else {
                            "no"
                        }
                        .into(),
                    ));
                }
                4 => {
                    // VLAN
                    let parent = vlan_parent_entry.text().to_string();
                    if !parent.is_empty() {
                        settings.push(("dev".into(), parent));
                    }
                    let id = vlan_id_entry.text().to_string();
                    if !id.is_empty() {
                        settings.push(("id".into(), id));
                    }
                }
                _ => {}
            }

            // IPv4
            let ipv4_m_idx = ipv4_method_dd.selected() as usize;
            let ipv4_method = IPV4_METHODS.get(ipv4_m_idx).unwrap_or(&"auto");
            settings.push(("ipv4.method".into(), ipv4_method.to_string()));
            let addr_text = ipv4_addr.text().to_string();
            if !addr_text.is_empty() {
                settings.push(("ipv4.addresses".into(), addr_text));
            }
            let gw_text = ipv4_gw.text().to_string();
            if !gw_text.is_empty() {
                settings.push(("ipv4.gateway".into(), gw_text));
            }
            let dns_text = ipv4_dns.text().to_string();
            if !dns_text.is_empty() {
                settings.push(("ipv4.dns".into(), dns_text));
            }

            // IPv6
            let ipv6_m_idx = ipv6_method_dd.selected() as usize;
            let ipv6_method = IPV6_METHODS.get(ipv6_m_idx).unwrap_or(&"auto");
            settings.push(("ipv6.method".into(), ipv6_method.to_string()));
            let addr6 = ipv6_addr.text().to_string();
            if !addr6.is_empty() {
                settings.push(("ipv6.addresses".into(), addr6));
            }
            let gw6 = ipv6_gw.text().to_string();
            if !gw6.is_empty() {
                settings.push(("ipv6.gateway".into(), gw6));
            }
            let dns6 = ipv6_dns.text().to_string();
            if !dns6.is_empty() {
                settings.push(("ipv6.dns".into(), dns6));
            }

            let con_type = con_type.to_string();
            let spinner = spinner.clone();
            let list_box = list_box.clone();
            let networks_data = networks_data.clone();

            window.close();
            spinner.start();

            gtk::glib::MainContext::default().spawn_local(async move {
                match nmcli::add_connection(&con_type, &name, &settings).await {
                    Ok(_uuid) => {}
                    Err(e) => {
                        crate::ui::dialogs::show_error_dialog(None, &format!("Failed to create connection:\n{}", e));
                    }
                }
                refresh_list(&list_box, &networks_data).await;
                spinner.stop();
            });
        }
    });

    window.present();
}
