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
use std::collections::HashMap;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::client::{MQTTyClient, MQTTyClientMessage, MQTTyClientQos, MQTTyClientVersion, TlsOptions};
use crate::display_mode::{MQTTyDisplayMode, MQTTyDisplayModeIface};
use crate::gsettings::MQTTySettingConnection;
use crate::subclass::prelude::*;

use super::{MQTTyMessageRow, MQTTySubscribeGeneralTab};
use crate::widgets::{MQTTyDataChart, MQTTyMessageDetailDialog, MQTTyTopicTreeView};

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
        enable_tls: Cell<bool>,

        #[property(get, set)]
        ca_cert_path: RefCell<String>,

        #[property(get, set)]
        client_cert_path: RefCell<String>,

        #[property(get, set)]
        client_key_path: RefCell<String>,

        #[property(get, set)]
        pub message_count: Cell<u32>,

        #[template_child]
        messages_list: TemplateChild<gtk::ListView>,

        #[template_child]
        pub topic_tree_view: TemplateChild<MQTTyTopicTreeView>,

        pub messages_model: OnceCell<gio::ListStore>,

        #[template_child]
        pub general_tab: TemplateChild<MQTTySubscribeGeneralTab>,

        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,

        /// Store previous messages per topic for diff view
        pub message_history: RefCell<HashMap<String, MQTTyClientMessage>>,

        #[property(get, set)]
        pub search_text: RefCell<String>,

        pub filter_model: OnceCell<gtk::FilterListModel>,
        pub custom_filter: OnceCell<gtk::CustomFilter>,

        #[template_child]
        pub chart_box: TemplateChild<gtk::Box>,

        /// Store charts per topic
        pub topic_charts: RefCell<HashMap<String, MQTTyDataChart>>,
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
                enable_tls: Default::default(),
                ca_cert_path: Default::default(),
                client_cert_path: Default::default(),
                client_key_path: Default::default(),
                message_count: Cell::new(0),
                messages_list: Default::default(),
                topic_tree_view: Default::default(),
                messages_model: Default::default(),
                general_tab: Default::default(),
                search_entry: Default::default(),
                message_history: RefCell::new(HashMap::new()),
                search_text: Default::default(),
                filter_model: Default::default(),
                custom_filter: Default::default(),
                chart_box: Default::default(),
                topic_charts: RefCell::new(HashMap::new()),
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

            // Create custom filter for search
            let search_text_ref = self.search_text.clone();
            let custom_filter = gtk::CustomFilter::new(move |item| {
                let search = search_text_ref.borrow();
                if search.is_empty() {
                    return true;
                }

                if let Some(row) = item.downcast_ref::<MQTTyMessageRow>() {
                    let topic = row.topic().to_lowercase();
                    let body = row.body_preview().to_lowercase();
                    topic.contains(search.as_str()) || body.contains(search.as_str())
                } else {
                    true
                }
            });
            self.custom_filter.set(custom_filter.clone()).unwrap();

            // Create filter model
            let filter_model = gtk::FilterListModel::new(Some(messages_model), Some(custom_filter));
            self.filter_model.set(filter_model.clone()).unwrap();

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

            let selection = gtk::NoSelection::new(Some(filter_model));
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

            // Handle clear retained message request from topic tree
            let obj_weak = obj.downgrade();
            self.topic_tree_view.connect_closure(
                "clear-retained-requested",
                false,
                glib::closure_local!(move |_tree_view: MQTTyTopicTreeView, topic: String| {
                    if let Some(obj) = obj_weak.upgrade() {
                        glib::spawn_future_local(glib::clone!(
                            #[weak]
                            obj,
                            async move {
                                obj.clear_retained_message(&topic).await;
                            }
                        ));
                    }
                }),
            );

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
            if let Some(model) = self.messages_model.get() {
                if let Some(item) = model.item(position) {
                    let row = item.downcast_ref::<MQTTyMessageRow>().unwrap();
                    if let Some(msg) = row.message() {
                        let dialog = MQTTyMessageDetailDialog::new();

                        // Get previous message for diff
                        let history = self.message_history.borrow();
                        let previous = history.get(&msg.topic());

                        dialog.set_message(&msg, previous);

                        if let Some(root) = self.obj().root() {
                            if let Some(window) = root.downcast_ref::<gtk::Window>() {
                                dialog.present(Some(window));
                            }
                        }
                    }
                }
            }
        }

        #[template_callback]
        fn on_search_changed(&self, search_entry: &gtk::SearchEntry) {
            let search_text = search_entry.text().to_string().to_lowercase();
            self.search_text.replace(search_text);

            // Invalidate the filter to re-run filtering
            if let Some(filter) = self.custom_filter.get() {
                filter.changed(gtk::FilterChange::Different);
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

        // Set TLS options
        let tls_options = TlsOptions {
            enabled: self.enable_tls(),
            ca_cert_path: {
                let path = self.ca_cert_path();
                if path.is_empty() { None } else { Some(path) }
            },
            client_cert_path: {
                let path = self.client_cert_path();
                if path.is_empty() { None } else { Some(path) }
            },
            client_key_path: {
                let path = self.client_key_path();
                if path.is_empty() { None } else { Some(path) }
            },
        };
        client.set_tls_options(tls_options);

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

                // Update topic tree
                let topic = message.topic();
                let body = message.body();
                let body_str = String::from_utf8_lossy(&body);
                obj.imp().topic_tree_view.process_message(&topic, &body_str);

                // Update chart with numeric data
                obj.update_chart(&topic, &body_str);

                // Store current message as previous for next diff
                obj.imp().message_history.borrow_mut().insert(topic, message.clone());
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

    /// Update chart with numeric data from a message
    fn update_chart(&self, topic: &str, payload: &str) {
        let mut charts = self.imp().topic_charts.borrow_mut();

        // Get or create chart for this topic
        let chart = charts.entry(topic.to_string()).or_insert_with(|| {
            let chart = MQTTyDataChart::new();
            chart.set_topic(topic);
            chart.set_hexpand(true);
            chart.set_height_request(150);

            // Add to chart box with a frame
            let frame = gtk::Frame::new(Some(topic));
            frame.set_child(Some(&chart));
            frame.set_margin_start(4);
            frame.set_margin_end(4);
            frame.set_margin_top(4);
            frame.set_margin_bottom(4);

            self.imp().chart_box.append(&frame);
            chart
        });

        // Try to add the payload as a data point
        chart.try_add_from_payload(payload);
    }

    /// Clear a retained message by publishing an empty payload with retain flag
    pub async fn clear_retained_message(&self, topic: &str) {
        let client_ref = self.imp().client.borrow();
        if let Some(client) = client_ref.as_ref() {
            let message = MQTTyClientMessage::new();
            message.set_topic(topic);
            message.set_body(&[]);
            message.set_retained(true);
            message.set_qos(self.qos());

            match client.publish(&message).await {
                Ok(_) => {
                    tracing::info!("Cleared retained message for topic: {}", topic);
                }
                Err(e) => {
                    tracing::error!("Failed to clear retained message for {}: {}", topic, e);
                }
            }
        } else {
            tracing::warn!("Cannot clear retained message: not connected");
        }
    }
}

impl Default for MQTTySubscribeViewNotebook {
    fn default() -> Self {
        Self::new()
    }
}
