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

use std::cell::RefCell;
use std::sync::LazyLock;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use glib::subclass::Signal;

use crate::application::MQTTyApplication;
use crate::gsettings::MQTTySettingConnection;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscribe_view/subscribe_general_tab.ui")]
    #[properties(wrapper_type = super::MQTTySubscribeGeneralTab)]
    pub struct MQTTySubscribeGeneralTab {
        #[property(get, set)]
        url: RefCell<String>,

        #[property(get, set)]
        topic: RefCell<String>,

        #[property(get, set)]
        username: RefCell<String>,

        #[property(get, set)]
        password: RefCell<String>,

        #[template_child]
        pub profile_combo: TemplateChild<adw::ComboRow>,

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
    impl ObjectSubclass for MQTTySubscribeGeneralTab {
        const NAME: &'static str = "MQTTySubscribeGeneralTab";

        type Type = super::MQTTySubscribeGeneralTab;

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
    impl ObjectImpl for MQTTySubscribeGeneralTab {
        fn constructed(&self) {
            self.parent_constructed();

            self.mqtt_3_button.set_action_target(Some("3"));
            self.mqtt_5_button.set_action_target(Some("5"));

            self.qos_0_button.set_action_target(Some("0"));
            self.qos_1_button.set_action_target(Some("1"));
            self.qos_2_button.set_action_target(Some("2"));

            // Setup profile combo
            let obj = self.obj();
            obj.setup_profile_combo();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![Signal::builder("profile-selected")
                    .param_types([MQTTySettingConnection::static_type()])
                    .build()]
            });
            &*SIGNALS
        }
    }
    impl WidgetImpl for MQTTySubscribeGeneralTab {}
    impl BinImpl for MQTTySubscribeGeneralTab {}

    #[gtk::template_callbacks]
    impl MQTTySubscribeGeneralTab {
        #[template_callback]
        fn or(&self, a: bool, b: bool) -> bool {
            a || b
        }
    }
}

glib::wrapper! {
    pub struct MQTTySubscribeGeneralTab(ObjectSubclass<imp::MQTTySubscribeGeneralTab>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTySubscribeGeneralTab {
    pub fn setup_profile_combo(&self) {
        let app = MQTTyApplication::get_singleton();
        let connections = app.settings_connections();
        let combo = &self.imp().profile_combo;

        // Create a string list model with profile names
        let model = gtk::StringList::new(&[]);

        // Add a "None" option first
        model.append("(None)");

        // Add all connection profile names
        for i in 0..connections.n_items() {
            if let Some(conn) = connections.item(i).and_downcast::<MQTTySettingConnection>() {
                let name = conn.name();
                if name.is_empty() {
                    model.append(&format!("Profile {}", i + 1));
                } else {
                    model.append(&name);
                }
            }
        }

        combo.set_model(Some(&model));
        combo.set_selected(0); // Default to "None"

        // Connect to selection changes
        combo.connect_selected_notify(glib::clone!(
            #[weak(rename_to = tab)]
            self,
            move |combo| {
                let selected = combo.selected();
                if selected == 0 {
                    // "None" selected, do nothing
                    return;
                }

                let app = MQTTyApplication::get_singleton();
                if let Some(conn) = app.settings_n_connection(selected - 1) {
                    // Emit the profile-selected signal
                    tab.emit_by_name::<()>("profile-selected", &[&conn]);
                }
            }
        ));

        // Listen for changes to the connections list to update the combo
        connections.connect_items_changed(glib::clone!(
            #[weak]
            combo,
            move |list, _pos, _removed, _added| {
                let model = gtk::StringList::new(&[]);
                model.append("(None)");

                for i in 0..list.n_items() {
                    if let Some(conn) = list.item(i).and_downcast::<MQTTySettingConnection>() {
                        let name = conn.name();
                        if name.is_empty() {
                            model.append(&format!("Profile {}", i + 1));
                        } else {
                            model.append(&name);
                        }
                    }
                }

                combo.set_model(Some(&model));
                combo.set_selected(0);
            }
        ));
    }
}
