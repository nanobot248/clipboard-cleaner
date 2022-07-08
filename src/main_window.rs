use std::cell::Cell;
use std::io::Write;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use encoding::all::{ASCII, ISO_8859_1, ISO_8859_15, UTF_16BE, UTF_16LE, UTF_8};
use encoding::{DecoderTrap, Encoding};
use glib::ObjectExt;
use glib::signal::Inhibit;
use gtk::{Application, ApplicationWindow, Clipboard, ComboBox, TextView};
use gtk::prelude::{BuilderExt, BuilderExtManual, ButtonExt, GtkWindowExt, LabelExt, TextBufferExt, TextViewExt, TreeModelExt, TreeSelectionExt, WidgetExt};
use parking_lot::{RwLock};
use crate::content_textbox::ContentTextbox;
use crate::{ Config, DEFAULT_SELECTION };
use crate::encoding_dropdown::EncodingDropdown;
use crate::profile_dropdown::ProfileDropdown;
use crate::targets_list::TargetsList;

pub struct MainWindow {
    app: Arc<Application>,
    window: Arc<ApplicationWindow>,
    config: Arc<Config>,

    on_delete_handler: RwLock<Cell<Box<dyn Fn(&MainWindow) -> () + 'static>>>,

    current_target: RwLock<String>,
    current_data: RwLock<Vec<u8>>,
}

impl MainWindow {
    pub fn new(app: Arc<Application>, config: Arc<Config>) -> anyhow::Result<Arc<MainWindow>> {
        let glade_src = include_str!("assets/main_window.glade");
        let builder = gtk::Builder::from_string(glade_src);
        builder.set_application(&*app);

        println!("retrieving application window ...");
        let window: ApplicationWindow = builder.object("main_window")
            .expect("could not create main window");
        // let window = Arc::new(gtk::ApplicationWindow::new(app));
        println!("object type: {}", ObjectExt::type_(&window).name());
        println!("window type: {:?}", window.window_type());
        println!("creating Arc from application window ...");

        // for some reason the main windows closes instantly after being shown if this line
        // is not used:
        let _window1 = gtk::ApplicationWindow::new(&*app);

        let treeview: gtk::TreeView = builder.object("targets_treeview")
            .expect("could not create targets list.");
        let treeview = Arc::new(treeview);
        let targets_list = TargetsList::new(app.clone(), treeview.clone());

        let info_label: gtk::Label = builder.object("info_label")
            .expect("could not create info label.");
        let info_label = Arc::new(info_label);

        let textbox: TextView = builder.object("content_textview")
            .expect("could not create textbox.");
        let textbox = Arc::new(textbox);
        let content_textbox = ContentTextbox::new(app.clone(), textbox.clone());

        let encoding_dropdown: ComboBox = builder.object("encoding_dropdown")
            .expect("could not create encoding dropdown.");
        let encoding_dropdown = Arc::new(encoding_dropdown);
        let encoding_dropdown = EncodingDropdown::new(app.clone(), encoding_dropdown.clone());

        let profiles_dropdown: ComboBox = builder.object("cleanup_profile_dropdown")
            .expect("could not create profiles dropdown.");
        let profiles_dropdown = Arc::new(profiles_dropdown);
        let profiles_dropdown = ProfileDropdown::new(app.clone(), profiles_dropdown.clone(), config.clone())?;

        let result = Arc::new(MainWindow {
            app: app.clone(),
            window: Arc::new(window),
            config: config.clone(),
            on_delete_handler: RwLock::new(Cell::new(Box::new(|_| {()}))),
            current_target: RwLock::new(String::new()),
            current_data: RwLock::new(Vec::new()),
        });

        let result_clone = result.clone();
        result.window.connect_delete_event(move |_window, _event| {
            println!("deleted!");
            let result_clone = result_clone.clone();
            result_clone.fire_delete();
            return Inhibit(false);
        });

        let result_clone = result.clone();
        let encoding_dropdown_clone = encoding_dropdown.clone();
        // let info_label_clone = info_label.clone();
        let content_textbox_clone = content_textbox.clone();
        targets_list.on_selection(move |_targets_list, selection| {
            let result_clone = result_clone.clone();
            let encoding_dropdown_clone = encoding_dropdown_clone.clone();
            // let info_label_clone = info_label_clone.clone();
            // let content_textbox_clone = content_textbox_clone.clone();
            let selection = selection.selected();
            if let Some((model, iterator)) = selection {
                let target_name = model.value(&iterator, 0).get::<String>();
                if let Ok(target_name) = target_name {
                    let target = gdk::Atom::intern(target_name.as_str());
                    let clipboard = gtk::Clipboard::get(&DEFAULT_SELECTION);
                    let content = clipboard.wait_for_contents(&target);
                    if let Some(content) = content {
                        let data = content.data();
                        result_clone.set_data(data);
                        let target_encoding = get_target_encoding(target.name().as_str());
                        println!("target_encoding: {:?}", &target_encoding);
                        encoding_dropdown_clone.set_encoding(target_encoding);
                    }
                }
            } else {
                encoding_dropdown_clone.set_encoding(None);
            }
        });

        let result_clone = result.clone();
        let textbox_clone = textbox.clone();
        let info_label_clone = info_label.clone();
        encoding_dropdown.on_change(move |_dropdown, encoding| {
            let result_clone = result_clone.clone();
            let textbox_clone = textbox_clone.clone();
            let info_label_clone = info_label_clone.clone();
            let text = convert_to_encoding(encoding.clone(), &result_clone.data());
            if let Some(text) = text {
                if contains_control_chars(text.as_str()) {
                    info_label_clone.set_text("Control characters have been replaced with \u{fffd}.");
                    let filtered_text = text.to_string().chars()
                        .map(|ch| if ch < '\u{0020}' && ch != '\t' && ch != '\n' && ch != '\r' {
                            '\u{fffd}'
                        } else {
                            ch
                        }).collect::<String>();
                    textbox_clone.buffer().unwrap().set_text(filtered_text.as_str());
                } else {
                    info_label_clone.set_text("");
                    textbox_clone.buffer().unwrap().set_text(text.as_str());
                }
            } else {
                info_label_clone.set_text(format!("Could not convert data to {:?}", &encoding).as_str());
                textbox_clone.buffer().unwrap().set_text("");
            }
        });

        let wipe_clipboard_button: gtk::Button = builder.object("wipe_clipboard_button")
            .expect("could not create wipe-clipboard button");
        wipe_clipboard_button.connect_clicked(|_button| {
            println!("wiping clipboard ...");
            let clipboard = Clipboard::get(&DEFAULT_SELECTION);
            clipboard.set_text("");
        });

        let cleanup_text_button: gtk::Button = builder.object("cleanup_text_button")
            .expect("could not create cleanup-text button");
        let profiles_dropdown_clone = profiles_dropdown.clone();
        cleanup_text_button.connect_clicked(move |_button| {
            // let profiles_dropdown_clone = profiles_dropdown_clone.clone();
            let profile = profiles_dropdown.profile();
            if let Some(profile) = profile {
                println!("using transformation profile: {:?}", &profile);
                let trafo = profile.transformation;
                let content = content_textbox.content();
                if let Some(content) = content {
                    let text = trafo.execute(content.as_str());
                    println!("transformed text: {}", text);
                    Clipboard::get(&DEFAULT_SELECTION).set_text(text.as_str());
                } else {
                    println!("no content to transform!");
                }
            } else {
                println!("no profile selected in profile dropdown!");
            }
        });

        targets_list.refresh_targets();

        return Ok(result);
    }

    fn fire_delete(&self) {
        self.on_delete_handler.write().get_mut()(&self);
    }

    pub fn on_delete<F: Fn(&MainWindow) -> () + 'static>(&self, handler: F) {
        self.on_delete_handler.write().set(Box::new(handler));
    }

    pub fn data(&self) -> Vec<u8> {
        return self.current_data.read().deref().clone();
    }

    pub fn set_data(&self, data: Vec<u8>) {
        let mut current_data = self.current_data.write();
        current_data.clear();
        current_data.write_all(data.as_slice())
            .expect("could not write new clipboard data to internal buffer!");
    }
}

fn get_target_encoding(target: &str) -> Option<&'static str> {
    let target = target.to_lowercase();
    println!("get_target_encoding: target={}", target.as_str());
    match target.as_str() {
        "utf8_string" => {
            return Some("utf-8");
        },
        "string" | "text" => {
            // the "STRING" target is Latin-1 (aka ISO-8859-1), as defined by ICCCM
            return Some("iso-8859-1");
        },
        _ => {
            let content_type = mime::Mime::from_str(target.as_str());
            println!("content_type: {:?}", &content_type);
            if let Ok(content_type) = content_type {
                let charset = content_type.get_param("charset").map(|name| name.to_string());
                println!("charset: {:?}", &charset);
                if let Some(charset) = charset {
                    println!("unwrapped charset: {:?}", &charset);
                    match charset.as_str() {
                        "utf-8" => {
                            return Some("utf-8");
                        },
                        "utf-16le" => {
                            return Some("utf-16le");
                        },
                        "utf-16be" => {
                            return Some("utf-16be");
                        },
                        "utf-16" | "unicode" => {
                            return Some("utf-16");
                        },
                        "iso-8859-1" => {
                            return Some("iso-8859-1");
                        },
                        "iso-8859-15" => {
                            return Some("iso-8859-15");
                        },
                        "us-ascii" => {
                            return Some("us-ascii");
                        },
                        _ => {}
                    }
                } else {
                    if content_type.type_().as_str() == "text" {
                        return Some("utf-8");
                    }
                }
            } else {
                return Some("utf-8");
            }

            return None;
        }
    }
}

fn convert_to_encoding(encoding: Option<String>, data: &Vec<u8>) -> Option<String> {
    if let Some(encoding) = encoding {
        match encoding.to_lowercase().as_str() {
            "utf-8" => {
                return String::from_utf8(data.clone()).ok();
            },
            "utf-16le" => {
                return UTF_16LE.decode(data.as_slice(), DecoderTrap::Replace).ok();
            },
            "utf-16be" => {
                return UTF_16BE.decode(data.as_slice(), DecoderTrap::Replace).ok();
            },
            "utf-16" | "unicode" => {
                let mut mode = 0;
                if data.len() >= 2 {
                    if *data.get(0).unwrap() == 0xFEu8 && *data.get(1).unwrap() == 0xFFu8 {
                        mode = 1;
                    } else if *data.get(0).unwrap() == 0xFFu8 && *data.get(1).unwrap() == 0xFEu8 {
                        mode = 2
                    } else {
                        #[cfg(target_endian = "big")] { mode = 1; }
                        #[cfg(target_endian = "little")] { mode = 2; }
                    }
                }
                if mode == 1 {
                    return UTF_16BE.decode(data.as_slice(), DecoderTrap::Replace).ok();
                } else if mode == 2 {
                    return UTF_16LE.decode(data.as_slice(), DecoderTrap::Replace).ok();
                } else {
                    println!("No valid BOM and no default mode.");
                    return None;
                }
            },
            "iso-8859-1" => {
                return ISO_8859_1.decode(data.as_slice(), DecoderTrap::Replace).ok();
            },
            "iso-8859-15" => {
                return ISO_8859_15.decode(data.as_slice(), DecoderTrap::Replace).ok();
            },
            "us-ascii" | "ascii" => {
                return ASCII.decode(data.as_slice(), DecoderTrap::Replace).ok();
            },
            _ => {
                return None;
            }
        }
    } else {
        return None;
    }
}

fn convert_to_string(target: &str, data: &Vec<u8>) -> Option<String> {
    let target = target.to_lowercase();
    println!("convert_to_string: target={}", target.as_str());
    match target.as_str() {
        "utf8_string" => {
            return String::from_utf8(data.clone()).ok();
        },
        "string" => {
            // the "STRING" target is Latin-1 (aka ISO-8859-1), as defined by ICCCM
            return ISO_8859_1.decode(data.as_slice(), DecoderTrap::Replace).ok();
        },
        "text" => {
            // the "TEXT" target's encoding is chose by the owner application, so we can only guess:
            return ISO_8859_1.decode(data.as_slice(), DecoderTrap::Replace).ok();
        },
        _ => {
            let content_type = mime::Mime::from_str(target.as_str());
            println!("content_type: {:?}", &content_type);
            if let Ok(content_type) = content_type {
                let charset = content_type.get_param("charset").map(|name| name.to_string());
                println!("charset: {:?}", &charset);
                let mut text: Option<String> = None;
                if let Some(charset) = charset {
                    println!("unwrapped charset: {:?}", &charset);
                    match charset.as_str() {
                        "utf-8" => {
                            text = String::from_utf8(data.clone()).ok();
                        },
                        "utf-16le" => {
                            text = UTF_16LE.decode(data.as_slice(), DecoderTrap::Replace).ok();
                        },
                        "utf-16be" => {
                            text = UTF_16BE.decode(data.as_slice(), DecoderTrap::Replace).ok();
                        },
                        "utf-16" | "unicode" => { // äöüß
                            let mut mode = 0;
                            if data.len() >= 2 {
                                if *data.get(0).unwrap() == 0xFEu8 && *data.get(1).unwrap() == 0xFFu8 {
                                    mode = 1;
                                } else if *data.get(0).unwrap() == 0xFFu8 && *data.get(1).unwrap() == 0xFEu8 {
                                    mode = 2
                                } else {
                                    #[cfg(target_endian = "big")] { mode = 1; }
                                    #[cfg(target_endian = "little")] { mode = 2; }
                                }
                            }
                            if mode == 1 {
                                text = UTF_16BE.decode(data.as_slice(), DecoderTrap::Replace).ok();
                            } else if mode == 2 {
                                text = UTF_16LE.decode(data.as_slice(), DecoderTrap::Replace).ok();
                            }
                        },
                        "iso-8859-1" => {
                            text = ISO_8859_1.decode(data.as_slice(), DecoderTrap::Replace).ok();
                        },
                        "iso-8859-15" => {
                            text = ISO_8859_15.decode(data.as_slice(), DecoderTrap::Replace).ok();
                        },
                        "us-ascii" => {
                            text = ASCII.decode(data.as_slice(), DecoderTrap::Replace).ok();
                        },
                        _ => {}
                    }
                } else {
                    if content_type.type_().as_str() == "text" {
                        text = UTF_8.decode(data.as_slice(), DecoderTrap::Replace).ok();
                    }
                }

                return text;
            } else {
                return UTF_8.decode(data.as_slice(), DecoderTrap::Replace).ok();
            }
        }
    }
}

fn contains_control_chars(text: &str) -> bool {
    for ch in text.chars() {
        if ch < (32 as char) && ch != '\t' && ch != '\n' && ch != '\r' {
            return true;
        }
    }

    return false;
}