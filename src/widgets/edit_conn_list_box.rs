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
use std::sync::LazyLock;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::glib::subclass::Signal;

use crate::gsettings::MQTTySettingConnection;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::MQTTyEditConnListBox)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/edit_conn_list_box.ui")]
    pub struct MQTTyEditConnListBox {
        #[property(get, set)]
        conn_model: RefCell<MQTTySettingConnection>,

        #[property(get, set, construct)]
        editing: Cell<bool>,

        #[template_child]
        name_row: TemplateChild<adw::EntryRow>,

        #[template_child]
        url_row: TemplateChild<adw::EntryRow>,

        #[template_child]
        topic_row: TemplateChild<adw::EntryRow>,

        #[template_child]
        username_row: TemplateChild<adw::EntryRow>,

        #[template_child]
        password_row: TemplateChild<adw::PasswordEntryRow>,

        #[template_child]
        mqtt_3_button: TemplateChild<gtk::CheckButton>,

        #[template_child]
        mqtt_5_button: TemplateChild<gtk::CheckButton>,

        #[template_child]
        qos_0_button: TemplateChild<gtk::CheckButton>,

        #[template_child]
        qos_1_button: TemplateChild<gtk::CheckButton>,

        #[template_child]
        qos_2_button: TemplateChild<gtk::CheckButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyEditConnListBox {
        const NAME: &'static str = "MQTTyEditConnListBox";

        type Type = super::MQTTyEditConnListBox;

        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyEditConnListBox {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            obj.connect_conn_model_notify(|obj| {
                let conn_model = obj.conn_model();

                let private = obj.imp();

                // Name
                conn_model
                    .bind_property("name", &*private.name_row, "text")
                    .bidirectional()
                    .sync_create()
                    .build();

                // URL
                conn_model
                    .bind_property("url", &*private.url_row, "text")
                    .bidirectional()
                    .sync_create()
                    .build();

                // Topic
                conn_model
                    .bind_property("topic", &*private.topic_row, "text")
                    .bidirectional()
                    .sync_create()
                    .build();

                // Username
                conn_model
                    .bind_property("username", &*private.username_row, "text")
                    .bidirectional()
                    .sync_create()
                    .build();

                // Password
                conn_model
                    .bind_property("password", &*private.password_row, "text")
                    .bidirectional()
                    .sync_create()
                    .build();

                // MQTT Version
                let mqtt_version = conn_model.mqtt_version();
                if mqtt_version == "5" {
                    private.mqtt_5_button.set_active(true);
                } else {
                    private.mqtt_3_button.set_active(true);
                }

                private.mqtt_3_button.connect_toggled(glib::clone!(
                    #[weak]
                    conn_model,
                    move |btn| {
                        if btn.is_active() {
                            conn_model.set_mqtt_version("3".to_string());
                        }
                    }
                ));

                private.mqtt_5_button.connect_toggled(glib::clone!(
                    #[weak]
                    conn_model,
                    move |btn| {
                        if btn.is_active() {
                            conn_model.set_mqtt_version("5".to_string());
                        }
                    }
                ));

                // QoS
                let qos = conn_model.qos();
                match qos.as_str() {
                    "1" => private.qos_1_button.set_active(true),
                    "2" => private.qos_2_button.set_active(true),
                    _ => private.qos_0_button.set_active(true),
                }

                private.qos_0_button.connect_toggled(glib::clone!(
                    #[weak]
                    conn_model,
                    move |btn| {
                        if btn.is_active() {
                            conn_model.set_qos("0".to_string());
                        }
                    }
                ));

                private.qos_1_button.connect_toggled(glib::clone!(
                    #[weak]
                    conn_model,
                    move |btn| {
                        if btn.is_active() {
                            conn_model.set_qos("1".to_string());
                        }
                    }
                ));

                private.qos_2_button.connect_toggled(glib::clone!(
                    #[weak]
                    conn_model,
                    move |btn| {
                        if btn.is_active() {
                            conn_model.set_qos("2".to_string());
                        }
                    }
                ));
            });
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![
                    Signal::builder("saving-conn").build(),
                    Signal::builder("deleting-conn").build(),
                ]
            });
            &*SIGNALS
        }
    }
    impl WidgetImpl for MQTTyEditConnListBox {}
    impl BinImpl for MQTTyEditConnListBox {}

    #[gtk::template_callbacks]
    impl MQTTyEditConnListBox {
        #[template_callback]
        fn on_save_conn(&self) {
            let obj = self.obj();

            obj.emit_by_name::<()>("saving-conn", &[]);
        }

        #[template_callback]
        fn on_delete_conn(&self) {
            let obj = self.obj();

            obj.emit_by_name::<()>("deleting-conn", &[]);
        }
    }
}

glib::wrapper! {
    pub struct MQTTyEditConnListBox(ObjectSubclass<imp::MQTTyEditConnListBox>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
