use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, Image, Label};

/// Create button with icon + label for context menu
pub fn make_menu_button(icon_name: &str, label_text: &str) -> Button {
    let btn = Button::new();
    let hbox = GtkBox::new(gtk::Orientation::Horizontal, 8);
    hbox.append(&Image::from_icon_name(icon_name));
    hbox.append(&Label::new(Some(label_text)));
    btn.set_child(Some(&hbox));
    btn.set_halign(gtk::Align::Fill);
    btn
}

/// Form row: label + widget side by side
pub fn form_row(label_text: &str, widget: &impl IsA<gtk::Widget>) -> GtkBox {
    let row = GtkBox::new(gtk::Orientation::Horizontal, 12);
    row.set_margin_start(8);
    row.set_margin_end(8);
    row.set_margin_top(4);
    row.set_margin_bottom(4);

    let label = Label::new(Some(label_text));
    label.set_halign(gtk::Align::Start);
    label.set_width_chars(16);
    label.set_xalign(0.0);
    row.append(&label);

    widget.set_hexpand(true);
    row.append(widget);

    row
}

/// Section title label
pub fn section_label(text: &str) -> Label {
    let label = Label::new(None);
    label.set_markup(&format!("<b>{}</b>", gtk::glib::markup_escape_text(text)));
    label.set_halign(gtk::Align::Start);
    label.set_margin_start(8);
    label.set_margin_top(12);
    label.set_margin_bottom(4);
    label.add_css_class("editor-section");
    label
}
