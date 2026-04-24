pub const APP_CSS: &str = "
    .wifiman-menu {
        padding: 4px;
    }
    .wifiman-menu button {
        background: transparent;
        border: none;
        border-radius: 5px;
        padding: 8px 18px;
        color: inherit;
        font-size: 13px;
        min-width: 160px;
    }
    .wifiman-menu button:hover {
        background: alpha(currentColor, 0.12);
    }
    .wifiman-menu .destructive {
        color: #e74c3c;
    }
    .editor-section {
        margin-top: 12px;
        margin-bottom: 4px;
        font-weight: bold;
        font-size: 14px;
    }
    .detail-section {
        margin-top: 10px;
        margin-bottom: 2px;
        font-weight: bold;
        font-size: 14px;
    }
    .detail-key {
        font-weight: bold;
        color: alpha(currentColor, 0.7);
    }
";

pub fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(APP_CSS);
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
