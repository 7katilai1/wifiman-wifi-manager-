use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{Box as GtkBox, ListBox, Spinner};

use crate::models::*;
use crate::nmcli;
use crate::ui::dialogs::refresh_list;
use crate::ui::widgets::make_menu_button;

pub fn show_context_popover(
    list_box: &ListBox,
    net: &Network,
    spinner: Spinner,
    x: f64,
    y: f64,
    list_box_ref: ListBox,
    networks_data: Rc<RefCell<Vec<Network>>>,
    parent_window: Option<gtk::Window>,
) {
    let popover = gtk::Popover::new();
    popover.set_parent(list_box);
    popover.set_has_arrow(true);
    popover.connect_closed({
        let popover = popover.clone();
        move |_| popover.unparent()
    });
    let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
    popover.set_pointing_to(Some(&rect));

    let vbox = GtkBox::new(gtk::Orientation::Vertical, 0);
    vbox.add_css_class("wifiman-menu");

    let net_clone = net.clone();

    // ── Connect / Disconnect ──
    if net.in_use {
        let btn = make_menu_button("network-offline-symbolic", "Disconnect");
        btn.connect_clicked({
            let popover = popover.clone();
            let spinner = spinner.clone();
            let net = net_clone.clone();
            let lb = list_box_ref.clone();
            let nd = networks_data.clone();
            move |_| {
                popover.popdown();
                spinner.start();
                let net = net.clone();
                let lb2 = lb.clone();
                let nd2 = nd.clone();
                let sp2 = spinner.clone();
                gtk::glib::MainContext::default().spawn_local(async move {
                    nmcli::disconnect_network(&net).await;
                    refresh_list(&lb2, &nd2).await;
                    sp2.stop();
                });
            }
        });
        vbox.append(&btn);
    } else {
        let btn = make_menu_button("network-wireless-connected-symbolic", "Connect");
        btn.connect_clicked({
            let popover = popover.clone();
            let spinner = spinner.clone();
            let net = net_clone.clone();
            let lb = list_box_ref.clone();
            let nd = networks_data.clone();
            let parent_win_clone = parent_window.clone();
            move |_| {
                popover.popdown();
                if net.net_type == NetworkType::Ethernet {
                    if let Some(ref uuid) = net.uuid {
                        let uuid = uuid.clone();
                        spinner.start();
                        let lb2 = lb.clone();
                        let nd2 = nd.clone();
                        let sp2 = spinner.clone();
                        let pw_clone = parent_win_clone.clone();
                        gtk::glib::MainContext::default().spawn_local(async move {
                            if let Err(e) = nmcli::connect_ethernet(&uuid).await {
                                crate::ui::dialogs::show_error_dialog(pw_clone.as_ref(), &format!("Failed to connect:\n{}", e));
                            }
                            refresh_list(&lb2, &nd2).await;
                            sp2.stop();
                        });
                    }
                } else {
                    let ssid = net.ssid.clone();
                    let uuid = net.uuid.clone();
                    let is_open = net.security.is_empty() || net.security == "--";
                    let has_profile = net.uuid.is_some();
                    if is_open || has_profile {
                        spinner.start();
                        let lb2 = lb.clone();
                        let nd2 = nd.clone();
                        let sp2 = spinner.clone();
                        let pw_clone = parent_win_clone.clone();
                        gtk::glib::MainContext::default().spawn_local(async move {
                            if let Err(e) = nmcli::connect_to_network(&ssid, None, uuid.as_deref()).await {
                                crate::ui::dialogs::show_error_dialog(pw_clone.as_ref(), &format!("Failed to connect:\n{}", e));
                            }
                            refresh_list(&lb2, &nd2).await;
                            sp2.stop();
                        });
                    }
                }
            }
        });
        vbox.append(&btn);
    }

    // ── Details (for saved profiles) ────
    if let Some(uuid) = net.uuid.clone() {
        let sep0 = gtk::Separator::new(gtk::Orientation::Horizontal);
        sep0.set_margin_top(4);
        sep0.set_margin_bottom(4);
        vbox.append(&sep0);

        let btn_details = make_menu_button("dialog-information-symbolic", "Details");
        btn_details.connect_clicked({
            let popover = popover.clone();
            let uuid = uuid.clone();
            let pw = parent_window.clone();
            move |_| {
                popover.popdown();
                let uuid = uuid.clone();
                let pw = pw.clone();
                gtk::glib::MainContext::default().spawn_local(async move {
                    crate::ui::details::show_details_window(pw.as_ref(), &uuid).await;
                });
            }
        });
        vbox.append(&btn_details);

        // ── Edit ──
        let btn_edit = make_menu_button("document-edit-symbolic", "Edit Connection");
        btn_edit.connect_clicked({
            let popover = popover.clone();
            let uuid = uuid.clone();
            let pw = parent_window.clone();
            let lb = list_box_ref.clone();
            let nd = networks_data.clone();
            let sp = spinner.clone();
            move |_| {
                popover.popdown();
                let uuid = uuid.clone();
                let pw = pw.clone();
                let lb2 = lb.clone();
                let nd2 = nd.clone();
                let sp2 = sp.clone();
                gtk::glib::MainContext::default().spawn_local(async move {
                    crate::ui::connection_editor::show_editor_window(
                        pw.as_ref(),
                        &uuid,
                        sp2,
                        lb2,
                        nd2,
                    )
                    .await;
                });
            }
        });
        vbox.append(&btn_edit);

        // ── Forget ──
        let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
        sep.set_margin_top(4);
        sep.set_margin_bottom(4);
        vbox.append(&sep);

        let btn_forget = make_menu_button("edit-delete-symbolic", "Forget Network");
        btn_forget.add_css_class("destructive");
        btn_forget.connect_clicked({
            let popover = popover.clone();
            let spinner = spinner.clone();
            let lb = list_box_ref.clone();
            let nd = networks_data.clone();
            let parent_win_clone = parent_window.clone();
            move |_| {
                popover.popdown();
                spinner.start();
                let uuid2 = uuid.clone();
                let lb2 = lb.clone();
                let nd2 = nd.clone();
                let sp2 = spinner.clone();
                let pw_clone = parent_win_clone.clone();
                gtk::glib::MainContext::default().spawn_local(async move {
                    if let Err(e) = nmcli::delete_connection(&uuid2).await {
                        crate::ui::dialogs::show_error_dialog(pw_clone.as_ref(), &format!("Failed to delete connection:\n{}", e));
                    }
                    refresh_list(&lb2, &nd2).await;
                    sp2.stop();
                });
            }
        });
        vbox.append(&btn_forget);
    }

    popover.set_child(Some(&vbox));
    popover.popup();
}
