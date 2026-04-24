use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Button, HeaderBar, Label, ListBox,
    ScrolledWindow, SelectionMode, Spinner,
};
use tokio::process::Command as AsyncCommand;

use crate::models::*;
use crate::nmcli;
use crate::style;
use crate::ui::context_menu::show_context_popover;
use crate::ui::dialogs::{refresh_list, show_password_dialog};
use crate::ui::network_list::update_network_list;

pub fn build_app() -> gtk::glib::ExitCode {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let _guard = rt.enter();

    let app = Application::builder()
        .application_id("com.wifiman")
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("WifiMan")
        .default_width(400)
        .default_height(600)
        .build();

    let header = HeaderBar::new();
    window.set_titlebar(Some(&header));

    // Load CSS
    style::load_css();

    // --- Header widgets ---
    let wifi_box = GtkBox::new(gtk::Orientation::Horizontal, 5);
    wifi_box.set_valign(gtk::Align::Center);
    wifi_box.append(&Label::new(Some("Wi-Fi")));
    let wifi_switch = gtk::Switch::new();
    wifi_box.append(&wifi_switch);
    header.pack_start(&wifi_box);

    let scan_button = Button::with_label("Scan");
    header.pack_start(&scan_button);

    // "+" button — create new connection
    let add_button = Button::from_icon_name("list-add-symbolic");
    add_button.set_tooltip_text(Some("New Connection"));
    header.pack_start(&add_button);

    let spinner = Spinner::new();
    header.pack_end(&spinner);

    // Hamburger menu — create button and popover, content populated later
    let menu_button = gtk::MenuButton::new();
    menu_button.set_icon_name("open-menu-symbolic");
    header.pack_end(&menu_button);

    // Read Wi-Fi radio status
    let wifi_switch_clone = wifi_switch.clone();
    gtk::glib::MainContext::default().spawn_local(async move {
        if let Ok(output) = AsyncCommand::new("nmcli")
            .args(&["-t", "radio", "wifi"])
            .output()
            .await
        {
            let state = String::from_utf8_lossy(&output.stdout);
            wifi_switch_clone.set_active(state.trim() == "enabled");
        }
    });

    // Wi-Fi toggle
    let spinner_clone = spinner.clone();
    wifi_switch.connect_active_notify(move |switch| {
        let state = switch.is_active();
        let spinner = spinner_clone.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            spinner.start();
            let arg = if state { "on" } else { "off" };
            let _ = AsyncCommand::new("nmcli")
                .args(&["radio", "wifi", arg])
                .output()
                .await;
            spinner.stop();
        });
    });

    // --- Network list ---
    let list_box = ListBox::new();
    list_box.set_selection_mode(SelectionMode::None);

    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .child(&list_box)
        .build();

    window.set_child(Some(&scrolled_window));

    let networks_data = Rc::new(RefCell::new(Vec::<Network>::new()));

    // --- Hamburger menu content (list_box and networks_data ready) ---
    {
        let menu_popover = gtk::Popover::new();
        let menu_vbox = GtkBox::new(gtk::Orientation::Vertical, 0);
        menu_vbox.add_css_class("wifiman-menu");

        let saved_conn_btn =
            crate::ui::widgets::make_menu_button("document-edit-symbolic", "Saved Connections");
        saved_conn_btn.connect_clicked({
            let menu_popover = menu_popover.clone();
            let window_weak = window.downgrade();
            let spinner = spinner.clone();
            let list_box = list_box.clone();
            let networks_data = networks_data.clone();
            move |_| {
                menu_popover.popdown();
                if let Some(win) = window_weak.upgrade() {
                    crate::ui::saved_connections::show_saved_connections_window(
                        Some(&win.into()),
                        spinner.clone(),
                        list_box.clone(),
                        networks_data.clone(),
                    );
                }
            }
        });
        menu_vbox.append(&saved_conn_btn);

        let hostname_btn =
            crate::ui::widgets::make_menu_button("network-server-symbolic", "Set Hostname");
        hostname_btn.connect_clicked({
            let menu_popover = menu_popover.clone();
            let window_weak = window.downgrade();
            move |_| {
                menu_popover.popdown();
                if let Some(win) = window_weak.upgrade() {
                    crate::ui::hostname::show_hostname_dialog(Some(&win.into()));
                }
            }
        });
        menu_vbox.append(&hostname_btn);
        menu_popover.set_child(Some(&menu_vbox));
        menu_button.set_popover(Some(&menu_popover));
    }

    // --- Load / Scan ---
    let load_networks = {
        let list_box = list_box.clone();
        let spinner = spinner.clone();
        let networks_data = networks_data.clone();
        move || {
            spinner.start();
            let list_box = list_box.clone();
            let spinner = spinner.clone();
            let networks_data = networks_data.clone();
            gtk::glib::MainContext::default().spawn_local(async move {
                if let Ok(networks) = nmcli::get_networks().await {
                    let filtered = update_network_list(&list_box, networks);
                    *networks_data.borrow_mut() = filtered;
                }
                spinner.stop();
            });
        }
    };

    scan_button.connect_clicked({
        let list_box = list_box.clone();
        let spinner = spinner.clone();
        let networks_data = networks_data.clone();
        move |_| {
            spinner.start();
            let list_box = list_box.clone();
            let spinner = spinner.clone();
            let networks_data = networks_data.clone();
            gtk::glib::MainContext::default().spawn_local(async move {
                let _ = nmcli::scan_networks().await;
                if let Ok(networks) = nmcli::get_networks().await {
                    let filtered = update_network_list(&list_box, networks);
                    *networks_data.borrow_mut() = filtered;
                }
                spinner.stop();
            });
        }
    });

    // Click on "+" button — create new connection
    add_button.connect_clicked({
        let window_weak = window.downgrade();
        let spinner = spinner.clone();
        let list_box = list_box.clone();
        let networks_data = networks_data.clone();
        move |_| {
            let parent = window_weak.upgrade().map(|w| -> gtk::Window { w.into() });
            crate::ui::connection_creator::show_creator_window(
                parent.as_ref(),
                spinner.clone(),
                list_box.clone(),
                networks_data.clone(),
            );
        }
    });

    // --- Mouse: left click = direct action, right click = context menu ---
    let gesture = gtk::GestureClick::new();
    gesture.set_button(0);
    gesture.connect_released({
        let window = window.clone();
        let spinner = spinner.clone();
        let list_box = list_box.clone();
        let networks_data = networks_data.clone();
        move |gesture, _n, x, y| {
            let button = gesture.current_button();
            let Some(row) = list_box.row_at_y(y as i32) else {
                return;
            };
            let index = row.index() as usize;

            let net = {
                let borrowed = networks_data.borrow();
                borrowed.get(index).cloned()
            };
            let Some(net) = net else { return };

            let spinner = spinner.clone();
            let list_box2 = list_box.clone();
            let nd2 = networks_data.clone();

            match button {
                1 => {
                    // ── Left click ──
                    if net.in_use {
                        spinner.start();
                        gtk::glib::MainContext::default().spawn_local(async move {
                            nmcli::disconnect_network(&net).await;
                            refresh_list(&list_box2, &nd2).await;
                            spinner.stop();
                        });
                    } else {
                        if net.net_type == NetworkType::Ethernet {
                            if let Some(uuid) = net.uuid.clone() {
                                spinner.start();
                                let win_clone = window.clone();
                                gtk::glib::MainContext::default().spawn_local(async move {
                                    if let Err(e) = nmcli::connect_ethernet(&uuid).await {
                                        crate::ui::dialogs::show_error_dialog(Some(&win_clone.into()), &format!("Failed to connect:\n{}", e));
                                    }
                                    refresh_list(&list_box2, &nd2).await;
                                    spinner.stop();
                                });
                            }
                        } else {
                            let is_open = net.security.is_empty() || net.security == "--";
                            let has_profile = net.uuid.is_some();

                            if is_open || has_profile {
                                let ssid = net.ssid.clone();
                                let uuid = net.uuid.clone();
                                spinner.start();
                                let win_clone = window.clone();
                                gtk::glib::MainContext::default().spawn_local(async move {
                                    if let Err(e) = nmcli::connect_to_network(
                                        &ssid,
                                        None,
                                        uuid.as_deref(),
                                    )
                                    .await {
                                        crate::ui::dialogs::show_error_dialog(Some(&win_clone.into()), &format!("Failed to connect:\n{}", e));
                                    }
                                    refresh_list(&list_box2, &nd2).await;
                                    spinner.stop();
                                });
                            } else {
                                show_password_dialog(
                                    &window,
                                    &net.ssid,
                                    spinner,
                                    list_box2,
                                    nd2,
                                );
                            }
                        }
                    }
                }
                3 => {
                    // ── Right click → context menu ──
                    let parent_win: gtk::Window = window.clone().into();
                    show_context_popover(
                        &list_box,
                        &net,
                        spinner,
                        x,
                        y,
                        list_box2,
                        nd2,
                        Some(parent_win),
                    );
                }
                _ => {}
            }
        }
    });
    list_box.add_controller(gesture);

    // Initial load
    load_networks();
    window.present();
}
