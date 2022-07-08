use std::cell::Cell;
use std::sync::Arc;
use glib::ToValue;
use gtk::{Application, ComboBox, ListStore};
use gtk::prelude::*;
use parking_lot::RwLock;
use crate::config::{Config, TransformationProfile};
use crate::text_transformation::TextTransformation;
use serde::{ Serialize, Deserialize };

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformationProfileInfo {
    pub profile: TransformationProfile,
    pub transformation: TextTransformation,
}

impl TransformationProfileInfo {
    pub fn new(config: &Config, profile: &TransformationProfile) -> anyhow::Result<TransformationProfileInfo> {
        let transformation = TextTransformation::new(config, profile)?;
        return Ok(TransformationProfileInfo {
            profile: profile.clone(),
            transformation
        });
    }

    pub fn name(&self) -> &str {
        return self.profile.name();
    }
}

pub struct ProfileDropdown {
    app: Arc<Application>,
    dropdown: Arc<ComboBox>,
    config: Arc<Config>,
    profiles: Vec<Arc<TransformationProfileInfo>>,

    on_change_handler: RwLock<Cell<Box<dyn Fn(&ProfileDropdown, Option<String>) -> () + 'static>>>,
}

impl ProfileDropdown {
    pub fn new(app: Arc<Application>, dropdown: Arc<ComboBox>, config: Arc<Config>) -> anyhow::Result<Arc<ProfileDropdown>> {
        let mut profiles: Vec<Arc<TransformationProfileInfo>> = Vec::new();
        for profile_config in config.profiles().iter() {
            let profile_info = Arc::new(TransformationProfileInfo::new(&*config, profile_config)?);
            profiles.push(profile_info);
        }

        let result = Arc::new(ProfileDropdown {
            app: app.clone(),
            dropdown: dropdown.clone(),
            config: config.clone(),
            profiles,
            on_change_handler: RwLock::new(Cell::new(Box::new(|_, _| {}))),
        });

        {
            let renderer = gtk::CellRendererText::new();
            dropdown.pack_start(&renderer, true);
            ComboBox::add_attribute(&*dropdown, &renderer, "text", 0);
            ComboBoxExt::set_id_column(&*dropdown, 1);
        }

        result.refresh_profiles();

        return Ok(result);
    }

    pub fn refresh_profiles(&self) {
        let col_types: [glib::Type; 2] = [
            glib::Type::STRING,
            glib::Type::STRING,
        ];
        let model = ListStore::new(&col_types);
        for profile in self.config.profiles().iter() {
            println!("adding profile: {:?}", &profile);
            let values : [(u32, &dyn ToValue); 2] = [
                (0, &profile.display_name()),
                (1, &profile.name()),
            ];
            let id = model.append();
            println!("id: {:?}", &id);
            model.set(&id, &values);
        }
        self.dropdown.set_model(Some(&model));

        if let Some(name) = self.config.default_profile_name() {
            ComboBox::set_active_id(&*self.dropdown, Some(name.as_str()));
        }
    }

    pub fn profile_name(&self) -> Option<String> {
        let id = self.dropdown.active_id();
        return id.map(|s| s.to_string());
    }

    pub fn profile(&self) -> Option<TransformationProfileInfo> {
        let name = self.profile_name();
        if let Some(name) = name {
            for profile in self.profiles.iter() {
                if profile.name() == name.as_str() {
                    return Some((**profile).clone());
                }
            }
        }

        return None;
    }

    pub fn fire_change(&self) {
        self.on_change_handler.write().get_mut()(self, self.profile_name());
    }

    pub fn on_change<F: Fn(&ProfileDropdown, Option<String>) -> () + 'static>(&self, handler: F) {
        self.on_change_handler.write().set(Box::new(handler));
    }
}