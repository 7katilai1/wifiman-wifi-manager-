use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{
    Box as GtkBox, Button, Image, Label, ListBox, ListBoxRow, ScrolledWindow, SelectionMode,
    Spinner,
};

use crate::models::*;
use crate::nmcli;

/// nmtui "Edit a connection" style — window listing all saved connections
pub fn show_saved_connections_window(
    parent: Option<&gtk::Window>,
    main_spinner: Spinner,
    main_list_box: ListBox,
    main_networks_data: Rc<RefCell<Vec<Network>>>,
) {
    let window = gtk::Window::builder()
        .title("Saved Connections")
        .default_width(460)
        .default_height(500)
        .modal(true)
        .build();
    if let Some(p) = parent {
        window.set_transient_for(Some(p));
    }

    let main_box = GtkBox::new(gtk::Orientation::Vertical, 0);

    // Header with title and add button
    let header_box = GtkBox::new(gtk::Orientation::Horizontal, 8);
    header_box.set_margin_start(12);
    header_box.set_margin_end(12);
    header_box.set_margin_top(12);
    header_box.set_margin_bottom(8);

    let title_label = Label::new(None);
    title_label.set_markup("<b>Saved Connection Profiles</b>");
    title_label.set_halign(gtk::Align::Start);
    title_label.set_hexpand(true);
    header_box.append(&title_label);

    let add_btn = Button::from_icon_name("list-add-symbolic");
    add_btn.set_tooltip_text(Some("New Connection"));
    header_box.append(&add_btn);

    main_box.append(&header_box);

    // Connection list
    let list_box = ListBox::new();
    list_box.set_selection_mode(SelectionMode::None);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .child(&list_box)
        .vexpand(true)
        .build();
    main_box.append(&scrolled);

    // Bottom close button
    let btn_box = GtkBox::new(gtk::Orientation::Horizontal, 0);
    btn_box.set_halign(gtk::Align::End);
    btn_box.set_margin_start(12);
    btn_box.set_margin_end(12);
    btn_box.set_margin_top(8);
    btn_box.set_margin_bottom(12);
    let close_btn = Button::with_label("Close");
    btn_box.append(&close_btn);
    main_box.append(&btn_box);

    window.set_child(Some(&main_box));

    close_btn.connect_clicked({
        let window = window.clone();
        move |_| window.close()
    });

    // Shared state for refreshing
    let saved_list_box = list_box.clone();
    let window_ref = window.clone();
    let main_spinner_ref = main_spinner.clone();
    let main_list_box_ref = main_list_box.clone();
    let main_nd_ref = main_networks_data.clone();

    let reload_saved: Rc<dyn Fn()> = Rc::new({
        let list_box = saved_list_box.clone();
        let window = window_ref.clone();
        let main_spinner = main_spinner_ref.clone();
        let main_list_box = main_list_box_ref.clone();
        let main_nd = main_nd_ref.clone();
        move || {
            let list_box = list_box.clone();
            let window = window.clone();
            let main_spinner = main_spinner.clone();
            let main_list_box = main_list_box.clone();
            let main_nd = main_nd.clone();
            gtk::glib::MainContext::default().spawn_local(async move {
                populate_saved_list(
                    &list_box,
                    &window,
                    main_spinner,
                    main_list_box,
                    main_nd,
                )
                .await;
            });
        }
    });

    // "+" button
    add_btn.connect_clicked({
        let window = window.clone();
        let main_spinner = main_spinner.clone();
        let main_list_box = main_list_box.clone();
        let main_nd = main_networks_data.clone();
        move |_| {
            crate::ui::connection_creator::show_creator_window(
                Some(&window.clone().into()),
                main_spinner.clone(),
                main_list_box.clone(),
                main_nd.clone(),
            );
        }
    });

    // Initial load
    let reload = reload_saved.clone();
    reload();

    window.present();
}

/// Populate saved connections into the ListBox
async fn populate_saved_list(
    list_box: &ListBox,
    window: &gtk::Window,
    main_spinner: Spinner,
    main_list_box: ListBox,
    main_networks_data: Rc<RefCell<Vec<Network>>>,
) {
    // Clear list
    while let Some(row) = list_box.row_at_index(0) {
        list_box.remove(&row);
    }

    let connections = match nmcli::get_saved_connections().await {
        Ok(c) => c,
        Err(e) => {
            crate::ui::dialogs::show_error_dialog(Some(window), &format!("Failed to get saved connections:\n{}", e));
            return;
        }
    };

    for con in &connections {
        let row = ListBoxRow::new();
        row.set_activatable(false);

        let hbox = GtkBox::new(gtk::Orientation::Horizontal, 8);
        hbox.set_margin_start(10);
        hbox.set_margin_end(10);
        hbox.set_margin_top(8);
        hbox.set_margin_bottom(8);

        // Type icon
        let icon_name = match con.con_type.as_str() {
            "802-3-ethernet" => "network-wired-symbolic",
            "802-11-wireless" => "network-wireless-symbolic",
            t if t.contains("bond") => "network-workgroup-symbolic",
            t if t.contains("bridge") => "network-workgroup-symbolic",
            t if t.contains("vlan") => "network-wired-symbolic",
            _ => "network-idle-symbolic",
        };
        hbox.append(&Image::from_icon_name(icon_name));

        // Name
        let name_label = Label::new(Some(&con.name));
        name_label.set_halign(gtk::Align::Start);
        name_label.set_hexpand(true);
        if con.active {
            name_label.set_markup(&format!(
                "<b>{}</b>",
                gtk::glib::markup_escape_text(&con.name)
            ));
        }
        hbox.append(&name_label);

        // Type label
        let type_short = match con.con_type.as_str() {
            "802-3-ethernet" => "Ethernet",
            "802-11-wireless" => "Wi-Fi",
            t if t.contains("bond") => "Bond",
            t if t.contains("bridge") => "Bridge",
            t if t.contains("vlan") => "VLAN",
            other => other,
        };
        let type_label = Label::new(None);
        type_label.set_markup(&format!(
            "<span size=\"small\" alpha=\"60%\">{}</span>",
            gtk::glib::markup_escape_text(type_short)
        ));
        hbox.append(&type_label);

        // Active indicator
        if con.active {
            let active_label = Label::new(None);
            active_label.set_markup("<span foreground=\"#4caf50\" size=\"small\"><b>●</b></span>");
            hbox.append(&active_label);
        }

        // ── Edit button ──
        let edit_btn = Button::from_icon_name("document-edit-symbolic");
        edit_btn.set_tooltip_text(Some("Edit"));
        edit_btn.set_valign(gtk::Align::Center);
        let uuid_for_edit = con.uuid.clone();
        let window_for_edit = window.clone();
        let ms = main_spinner.clone();
        let ml = main_list_box.clone();
        let mnd = main_networks_data.clone();
        let lb_for_refresh = list_box.clone();
        edit_btn.connect_clicked(move |_| {
            let uuid = uuid_for_edit.clone();
            let win = window_for_edit.clone();
            let sp = ms.clone();
            let lb = ml.clone();
            let nd = mnd.clone();
            let slb = lb_for_refresh.clone();
            gtk::glib::MainContext::default().spawn_local(async move {
                crate::ui::connection_editor::show_editor_window(
                    Some(&win.clone().into()),
                    &uuid,
                    sp.clone(),
                    lb.clone(),
                    nd.clone(),
                )
                .await;
                // Editor window presented, saved list should refresh when closed
                // Note: show_editor_window is async and returns after presenting window,
                // main list is already refreshed when the editor's Save button is clicked.
                // Here we also refresh the saved list.
                populate_saved_list(&slb, &win, sp, lb, nd).await;
            });
        });
        hbox.append(&edit_btn);

        // ── Delete button ──
        let del_btn = Button::from_icon_name("edit-delete-symbolic");
        del_btn.set_tooltip_text(Some("Delete"));
        del_btn.set_valign(gtk::Align::Center);
        del_btn.add_css_class("destructive");
        let uuid_for_del = con.uuid.clone();
        let window_for_del = window.clone();
        let ms2 = main_spinner.clone();
        let ml2 = main_list_box.clone();
        let mnd2 = main_networks_data.clone();
        let lb_for_del = list_box.clone();
        del_btn.connect_clicked(move |_| {
            let uuid = uuid_for_del.clone();
            let win = window_for_del.clone();
            let sp = ms2.clone();
            let lb = ml2.clone();
            let nd = mnd2.clone();
            let slb = lb_for_del.clone();
            gtk::glib::MainContext::default().spawn_local(async move {
                if let Err(e) = nmcli::delete_connection(&uuid).await {
                    crate::ui::dialogs::show_error_dialog(Some(&win), &format!("Failed to delete connection:\n{}", e));
                }
                // Refresh both saved list and main list
                crate::ui::dialogs::refresh_list(&lb, &nd).await;
                populate_saved_list(&slb, &win, sp, lb, nd).await;
            });
        });
        hbox.append(&del_btn);

        row.set_child(Some(&hbox));
        list_box.append(&row);
    }

    // Empty state
    if connections.is_empty() {
        let row = ListBoxRow::new();
        row.set_activatable(false);
        let label = Label::new(Some("No saved connections"));
        label.set_margin_top(20);
        label.set_margin_bottom(20);
        label.set_opacity(0.6);
        row.set_child(Some(&label));
        list_box.append(&row);
    }
}
