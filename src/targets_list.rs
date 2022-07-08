use std::cell::Cell;
use std::sync::Arc;
use gdk::{Atom, SELECTION_CLIPBOARD};
use glib::{ObjectExt, ToValue};
use gtk::prelude::{GtkListStoreExt, GtkListStoreExtManual, TreeModelExt, TreeSelectionExt, TreeViewColumnExt, TreeViewExt};
use gtk::{Application, TreeSelection, TreeView};
use parking_lot::RwLock;

pub const DEFAULT_SELECTION: Atom = SELECTION_CLIPBOARD;

pub struct TargetsList {
    app: Arc<Application>,
    treeview: Arc<TreeView>,
    on_model_change_handler: RwLock<Cell<Box<dyn Fn(&TargetsList) -> () + 'static>>>,
    on_selection_handler: RwLock<Cell<Box<dyn Fn(&TargetsList, &TreeSelection) -> () + 'static>>>,
}

impl TargetsList {
    pub fn new(app: Arc<Application>, treeview: Arc<TreeView>) -> Arc<TargetsList> {
        println!("creating list store ...");
        let list_store = create_list_store();
        treeview.set_model(Some(&*list_store));

        {
            let renderer = gtk::CellRendererText::new();
            let column = gtk::TreeViewColumn::new();
            column.pack_start(&renderer, true);
            column.set_title("Content Type");
            column.add_attribute(&renderer, "text", 0);
            column.set_sort_column_id(0);
            treeview.append_column(&column);
        }

        let result = Arc::new(TargetsList {
            app: app.clone(),
            treeview: treeview.clone(),
            on_model_change_handler: RwLock::new(Cell::new(Box::new(|_| {}))),
            on_selection_handler: RwLock::new(Cell::new(Box::new(|_, _| {}))),
        });

        let result_clone = result.clone();
        treeview.selection().connect_changed(move |selection| {
            let result_clone = result_clone.clone();
            result_clone.fire_selection(selection);
        });

        let result_clone = result.clone();
        let treeview_clone = result.treeview.clone();
        gtk::Clipboard::get(&DEFAULT_SELECTION).connect_local("owner-change", true, move |_value| {
            let result_clone = result_clone.clone();
            // let treeview_clone = treeview_clone.clone();
            println!("clipboard changed!");
            result_clone.refresh_targets();
            return None;
        });

        result.select_default_target();

        return result;
    }

    pub fn refresh_targets(&self) {
        let model = create_list_store();
        self.treeview.clone().set_model(Some(&*model));

        self.select_default_target();
        self.fire_model_change();
    }

    fn fire_selection(&self, selection: &TreeSelection) {
        self.on_selection_handler.write().get_mut()(self, selection);
    }

    fn fire_model_change(&self) {
        self.on_model_change_handler.write().get_mut()(self);
    }

    pub fn on_model_change<F: Fn(&TargetsList) -> () + 'static>(&self, handler: F) {
        self.on_model_change_handler.write().set(Box::new(handler));
    }

    pub fn on_selection<F: Fn(&TargetsList, &TreeSelection) -> () + 'static>(&self, handler: F) {
        self.on_selection_handler.write().set(Box::new(handler));
    }

    pub fn select_default_target(&self) {
        let tree_model = self.treeview.model().unwrap();
        let tree_iter = tree_model.iter_first();
        let mut string_set = false;
        if let Some(tree_iter) = tree_iter {
            loop {
                let value = tree_model.value(&tree_iter, 0);
                let target_name = value.get::<String>();
                if let Ok(target_name) = target_name {
                    if target_name.as_str() == "UTF8_STRING" {
                        self.treeview.selection().select_iter(&tree_iter);
                        break;
                    } else if !string_set && target_name.as_str() == "TEXT" {
                        self.treeview.selection().select_iter(&tree_iter);
                    } else if target_name.as_str() == "STRING" {
                        self.treeview.selection().select_iter(&tree_iter);
                        string_set = true;
                    }
                }

                if !tree_model.iter_next(&tree_iter) {
                    break;
                }
            }
        }
    }
}

fn get_clipboard_targets() -> Vec<Atom> {
    let clipboard = gtk::Clipboard::get(&DEFAULT_SELECTION);
    let targets = clipboard.wait_for_targets();

    return targets.or(Some(Vec::new())).unwrap();
}

fn create_list_store() -> Arc<gtk::ListStore> {
    let col_types: [glib::Type; 1] = [
        glib::Type::STRING,
    ];

    let store = Arc::new(gtk::ListStore::new(&col_types));
    let mut data = get_clipboard_targets();
    data.sort_by(|a, b| a.name().as_str().cmp(b.name().as_str()));
    for target in data {
        let name = target.name().to_string();
        let values: [(u32, &dyn ToValue); 1] = [
            (0, &name),
        ];
        store.set(&store.append(), &values);
    }

    return store;
}