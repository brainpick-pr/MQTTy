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

use std::cell::{Cell, OnceCell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::client::{MQTTyClient, MQTTyClientMessage, MQTTyClientQos, MQTTyClientVersion};
use crate::display_mode::{MQTTyDisplayMode, MQTTyDisplayModeIface};
use crate::gsettings::MQTTySettingConnection;
use crate::subclass::prelude::*;

use super::{MQTTyMessageRow, MQTTySubscribeGeneralTab};

mod imp {

    use super::*;

    #[derive(gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscribe_view/subscribe_view_notebook.ui")]
    #[properties(wrapper_type = super::MQTTySubscribeViewNotebook)]
    pub struct MQTTySubscribeViewNotebook {
        pub client: RefCell<Option<MQTTyClient>>,

        #[property(get, set, override_interface = MQTTyDisplayModeIface)]
        display_mode: Cell<MQTTyDisplayMode>,

        #[property(get, set, builder(Default::default()))]
        mqtt_version: Cell<MQTTyClientVersion>,

        #[property(get, set)]
        topic: RefCell<String>,

        #[property(get, set)]
        url: RefCell<String>,

        #[property(get, set, builder(Default::default()))]
        qos: Cell<MQTTyClientQos>,

        #[property(get, set)]
        username: RefCell<String>,

        #[property(get, set)]
        password: RefCell<String>,

        #[property(get, set)]
        pub message_count: Cell<u32>,

        #[template_child]
        messages_list: TemplateChild<gtk::ListView>,

        pub messages_model: OnceCell<gio::ListStore>,

        #[template_child]
        pub general_tab: TemplateChild<MQTTySubscribeGeneralTab>,
    }

    impl Default for MQTTySubscribeViewNotebook {
        fn default() -> Self {
            Self {
                display_mode: Cell::new(MQTTyDisplayMode::Desktop),
                mqtt_version: Default::default(),
                topic: Default::default(),
                url: Default::default(),
                qos: Default::default(),
                client: Default::default(),
                username: Default::default(),
                password: Default::default(),
                message_count: Cell::new(0),
                messages_list: Default::default(),
                messages_model: Default::default(),
                general_tab: Default::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscribeViewNotebook {
        const NAME: &'static str = "MQTTySubscribeViewNotebook";

        type Type = super::MQTTySubscribeViewNotebook;

        type ParentType = adw::Bin;

        type Interfaces = (MQTTyDisplayModeIface,);

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.install_action("subscribe-notebook.clear-messages", None, |this, _, _| {
                this.clear_messages();
            });
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTySubscribeViewNotebook {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            // Setup messages list
            let messages_model = gio::ListStore::new::<MQTTyMessageRow>();
            self.messages_model.set(messages_model.clone()).unwrap();

            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_, list_item| {
                let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                // The child will be set in bind
                list_item.set_child(None::<&gtk::Widget>);
            });

            factory.connect_bind(|_, list_item| {
                let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                let item = list_item.item().and_downcast::<MQTTyMessageRow>();
                if let Some(row) = item {
                    list_item.set_child(Some(&row));
                }
            });

            let selection = gtk::NoSelection::new(Some(messages_model));
            self.messages_list.set_model(Some(&selection));
            self.messages_list.set_factory(Some(&factory));

            // Setup action group for MQTT version and QoS
            let group = gio::SimpleActionGroup::new();

            let mqtt_version_state = gio::SimpleAction::new_stateful(
                "mqtt-version",
                Some(glib::VariantTy::STRING),
                &"3".into(),
            );
            mqtt_version_state
                .bind_property("state", &*obj, "mqtt_version")
                .bidirectional()
                .sync_create()
                .transform_to(|_, state: glib::Variant| {
                    let version = match state.str().unwrap_or("3") {
                        "3" => MQTTyClientVersion::V3X,
                        "5" => MQTTyClientVersion::V5,
                        version => {
                            tracing::warn!("Unknown MQTT version '{}', defaulting to V3X", version);
                            MQTTyClientVersion::V3X
                        }
                    };

                    Some(version)
                })
                .transform_from(|_, mqtt_version: MQTTyClientVersion| {
                    let new_state = match mqtt_version {
                        MQTTyClientVersion::V3X => "3",
                        MQTTyClientVersion::V5 => "5",
                    };

                    Some(glib::Variant::from(new_state))
                })
                .build();

            let qos_state =
                gio::SimpleAction::new_stateful("qos", Some(glib::VariantTy::STRING), &"0".into());
            qos_state
                .bind_property("state", &*obj, "qos")
                .bidirectional()
                .sync_create()
                .transform_to(|_, state: glib::Variant| {
                    let qos = match state.str().unwrap_or("0") {
                        "0" => MQTTyClientQos::Qos0,
                        "1" => MQTTyClientQos::Qos1,
                        "2" => MQTTyClientQos::Qos2,
                        qos => {
                            tracing::warn!("Unknown MQTT QoS '{}', defaulting to QoS0", qos);
                            MQTTyClientQos::Qos0
                        }
                    };

                    Some(qos)
                })
                .transform_from(|_, qos: MQTTyClientQos| {
                    let new_state = match qos {
                        MQTTyClientQos::Qos0 => "0",
                        MQTTyClientQos::Qos1 => "1",
                        MQTTyClientQos::Qos2 => "2",
                    };

                    Some(glib::Variant::from(new_state))
                })
                .build();

            group.add_action(&mqtt_version_state);
            group.add_action(&qos_state);

            obj.insert_action_group("subscribe-notebook", Some(&group));

            // Handle profile selection
            self.general_tab.connect_closure(
                "profile-selected",
                false,
                glib::closure_local!(
                    #[weak]
                    obj,
                    #[weak]
                    mqtt_version_state,
                    #[weak]
                    qos_state,
                    move |_tab: MQTTySubscribeGeneralTab, conn: MQTTySettingConnection| {
                        // Update all the notebook properties from the profile
                        obj.set_topic(conn.topic());
                        obj.set_url(conn.url());
                        obj.set_username(conn.username());
                        obj.set_password(conn.password());

                        // Update MQTT version action state
                        let version = conn.mqtt_version();
                        mqtt_version_state.change_state(&version.into());

                        // Update QoS action state
                        let qos = conn.qos();
                        qos_state.change_state(&qos.into());
                    }
                ),
            );
        }
    }
    impl WidgetImpl for MQTTySubscribeViewNotebook {}
    impl BinImpl for MQTTySubscribeViewNotebook {}
    impl MQTTyDisplayModeIfaceImpl for MQTTySubscribeViewNotebook {}

    #[gtk::template_callbacks]
    impl MQTTySubscribeViewNotebook {
        #[template_callback]
        fn on_message_activated(&self, position: u32, _list_view: &gtk::ListView) {
            // TODO: Show message details dialog
            if let Some(model) = self.messages_model.get() {
                if let Some(item) = model.item(position) {
                    let row = item.downcast_ref::<MQTTyMessageRow>().unwrap();
                    if let Some(msg) = row.message() {
                        println!("Message clicked: {:?}", msg.topic());
                    }
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct MQTTySubscribeViewNotebook(ObjectSubclass<imp::MQTTySubscribeViewNotebook>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTySubscribeViewNotebook {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub async fn subscribe(&self) -> Result<(), String> {
        let mqtt_version = self.mqtt_version();

        let client = MQTTyClient::new(
            &self.url(),
            mqtt_version,
            &self.username(),
            &self.password(),
        );

        client.connect_client().await?;

        // Setup message handler
        let messages_model = self.imp().messages_model.get().unwrap().clone();
        let obj_weak = self.downgrade();

        client.connect_message(move |_client, message| {
            let row = MQTTyMessageRow::from_message(message);
            messages_model.insert(0, &row);

            // Update message count
            if let Some(obj) = obj_weak.upgrade() {
                let count = obj.imp().message_count.get();
                obj.imp().message_count.set(count + 1);
                obj.notify("message-count");
            }
        });

        // Subscribe to topic
        client.subscribe(&self.topic(), self.qos()).await?;

        // Store the client to keep connection alive
        self.imp().client.replace(Some(client));

        Ok(())
    }

    pub fn clear_messages(&self) {
        if let Some(model) = self.imp().messages_model.get() {
            model.remove_all();
            self.imp().message_count.set(0);
            self.notify("message-count");
        }
    }
}

impl Default for MQTTySubscribeViewNotebook {
    fn default() -> Self {
        Self::new()
    }
}
