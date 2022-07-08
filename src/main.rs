extern crate core;

use std::sync::Arc;
use gdk::Atom;

use gio::ApplicationFlags;
use gio::prelude::*;
use gtk::Application;
use gtk::gdk::SELECTION_CLIPBOARD;
use crate::config::{Config, Transformation, TransformationAction};
use crate::config_loader::ConfigurationLoaderBuilder;
use crate::main_window::MainWindow;

mod config;
mod config_loader;
mod main_window;
mod profile_dropdown;
mod encoding_dropdown;
mod targets_list;
mod content_textbox;
mod text_transformation;
mod char_filter;

const DEFAULT_SELECTION: Atom = SELECTION_CLIPBOARD;

fn new_ui(app: Arc<Application>, config: Arc<Config>) -> anyhow::Result<Arc<MainWindow>> {
    let main_window = MainWindow::new(app.clone(), config)?;
    return Ok(main_window.clone());
}

fn main() {
    glib::set_program_name(Some("clipboard-cleaner"));
    glib::set_application_name("clipboard-cleaner");
    let app = Arc::new(gtk::Application::new(Some("net.laerrus.ClipboardCleaner"),
                                    ApplicationFlags::default()));

    let mut config_loader = ConfigurationLoaderBuilder::new( "net.laerrus", "Laerrus Ultd.", "clipboard-cleaner");
    config_loader.base_name("clipboard-cleaner");
    let config_loader = config_loader.build();
    let mut config: anyhow::Result<Config> = config_loader.load_configuration();
    if config.is_err() {
        println!("No configuration file found. Using integrated default config.");
        let config_yaml = include_str!("assets/default-config.yaml");
        config = serde_yaml::from_str::<Config>(config_yaml).or_else(|err| Err(anyhow::Error::from(err)));
    }
    println!("config: {:?}", &config);

    let config = Arc::new(config.expect("Could not read configuration."));
    let app_clone = app.clone();
    app.connect_activate(move |_app_ref| {
        let app_clone = app_clone.clone();
        new_ui(app_clone.clone(), config.clone())
            .expect("could not build UI")
            .on_delete(move |_window| {
                let app_clone = app_clone.clone();
                app_clone.quit();
            });
    });

    // let trafo = SimpleTransformationAction::from_str("l{ident}o{char}r{hex-esc}e{uni-simple}m{uni-esc} {uni-codepoint}i{rust}p{entity}")
    //     .expect("Could not parse transformation pattern.");
    // let transformed1 = "\u{fffd}sum dolor sit amed!".chars().map(|ch| {
    //     let new_ch = (&trafo).execute(ch);
    //     println!("mapping {} to: {:?}", ch, &new_ch);
    //     return new_ch.or(Some(String::new())).unwrap();
    // }).collect::<String>();
    // println!("transformed1: {}",  transformed1.as_str());

    app.run();
}
