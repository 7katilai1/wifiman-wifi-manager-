mod app;
mod models;
mod nmcli;
mod style;
mod ui;
mod utils;

fn main() -> gtk::glib::ExitCode {
    app::build_app()
}
