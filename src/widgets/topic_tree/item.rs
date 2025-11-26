use std::cell::{OnceCell, RefCell};

use gtk::{gio, glib};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MQTTyTopicItem)]
    pub struct MQTTyTopicItem {
        #[property(get, set)]
        name: RefCell<String>,

        #[property(get, set)]
        full_topic: RefCell<String>,

        #[property(get, set)]
        payload: RefCell<String>,

        pub children: OnceCell<gio::ListStore>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyTopicItem {
        const NAME: &'static str = "MQTTyTopicItem";
        type Type = super::MQTTyTopicItem;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyTopicItem {
        fn constructed(&self) {
            self.parent_constructed();
            let _ = self.children.set(gio::ListStore::new::<super::MQTTyTopicItem>());
        }
    }
}

glib::wrapper! {
    pub struct MQTTyTopicItem(ObjectSubclass<imp::MQTTyTopicItem>);
}

impl MQTTyTopicItem {
    pub fn new(name: &str, full_topic: &str) -> Self {
        glib::Object::builder()
            .property("name", name)
            .property("full_topic", full_topic)
            .build()
    }

    pub fn children(&self) -> gio::ListStore {
        self.imp().children.get().cloned().unwrap_or_else(|| gio::ListStore::new::<MQTTyTopicItem>())
    }

    pub fn add_child(&self, item: &MQTTyTopicItem) {
        self.children().append(item);
    }
    
    pub fn find_child(&self, name: &str) -> Option<MQTTyTopicItem> {
        let children = self.children();
        for i in 0..children.n_items() {
            if let Some(item) = children.item(i).and_downcast::<MQTTyTopicItem>() {
                if item.name() == name {
                    return Some(item);
                }
            }
        }
        None
    }
}
