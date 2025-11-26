// Copyright (c) 2025 Oscar Pernia
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::cell::{Cell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use crate::client::{MQTTyClientMessage, MQTTyClientQos};

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscribe_view/message_row.ui")]
    #[properties(wrapper_type = super::MQTTyMessageRow)]
    pub struct MQTTyMessageRow {
        #[property(get, set)]
        topic: RefCell<String>,

        #[property(get, set)]
        body_preview: RefCell<String>,

        #[property(get, set)]
        timestamp: RefCell<String>,

        #[property(get, set)]
        qos_label: RefCell<String>,

        #[property(get, set)]
        retained: Cell<bool>,

        /// Store the full message for later access
        pub message: RefCell<Option<MQTTyClientMessage>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyMessageRow {
        const NAME: &'static str = "MQTTyMessageRow";

        type Type = super::MQTTyMessageRow;

        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyMessageRow {}
    impl WidgetImpl for MQTTyMessageRow {}
    impl BoxImpl for MQTTyMessageRow {}
}

glib::wrapper! {
    pub struct MQTTyMessageRow(ObjectSubclass<imp::MQTTyMessageRow>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl MQTTyMessageRow {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn from_message(message: &MQTTyClientMessage) -> Self {
        let row = Self::new();

        row.set_topic(message.topic());

        // Create a preview of the body (first 100 chars)
        let body = message.body();
        let body_str = String::from_utf8_lossy(&body);
        let preview = if body_str.len() > 100 {
            format!("{}...", &body_str[..100])
        } else {
            body_str.to_string()
        };
        row.set_body_preview(preview);

        // Format timestamp
        let now = glib::DateTime::now_local().unwrap();
        row.set_timestamp(now.format("%H:%M:%S").unwrap().to_string());

        // Format QoS label
        let qos_str = match message.qos() {
            MQTTyClientQos::Qos0 => "QoS 0",
            MQTTyClientQos::Qos1 => "QoS 1",
            MQTTyClientQos::Qos2 => "QoS 2",
        };
        row.set_qos_label(qos_str.to_string());

        row.set_retained(message.retained());

        // Store the full message
        row.imp().message.replace(Some(message.clone()));

        row
    }

    pub fn message(&self) -> Option<MQTTyClientMessage> {
        self.imp().message.borrow().clone()
    }
}

impl Default for MQTTyMessageRow {
    fn default() -> Self {
        Self::new()
    }
}
