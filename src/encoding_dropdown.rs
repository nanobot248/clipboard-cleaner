use std::cell::Cell;
use std::sync::Arc;
use gtk::{Application, ComboBox};
use gtk::prelude::ComboBoxExt;
use parking_lot::RwLock;

pub struct EncodingDropdown {
    app: Arc<Application>,
    dropdown: Arc<ComboBox>,

    on_change_handler: RwLock<Cell<Box<dyn Fn(&EncodingDropdown, Option<String>) -> () + 'static>>>,
}

impl EncodingDropdown {
    pub fn new(app: Arc<Application>, dropdown: Arc<ComboBox>) -> Arc<EncodingDropdown> {
        let result = Arc::new(EncodingDropdown {
            app: app.clone(),
            dropdown: dropdown.clone(),
            on_change_handler: RwLock::new(Cell::new(Box::new(|_, _| {}))),
        });
        let result_clone = result.clone();
        dropdown.clone().connect_changed(move |_dropdown| {
            let result_clone = result_clone.clone();
            result_clone.on_change_handler.write().get_mut()(&*result_clone, result_clone.encoding());
        });
        return result.clone();
    }

    pub fn encoding(&self) -> Option<String> {
        let id = self.dropdown.active_id();
        if let Some(id) = id {
            if id.as_str() == "-" {
                return None;
            } else {
                return Some(id.to_string());
            }
        }
        return None;
    }

    pub fn set_encoding(&self, encoding: Option<&str>) {
        if let Some(encoding) = encoding {
            self.dropdown.set_active_id(Some(encoding));
        } else {
            self.dropdown.set_active_id(Some("-"));
        }
        self.fire_change();
    }

    pub fn fire_change(&self) {
        self.on_change_handler.write().get_mut()(self, self.encoding());
    }

    pub fn on_change<F: Fn(&EncodingDropdown, Option<String>) -> () + 'static>(&self, handler: F) {
        self.on_change_handler.write().set(Box::new(handler));
    }
}