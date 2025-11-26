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

use adw::subclass::prelude::*;
use gtk::glib;
use gtk::glib::variant::{FromVariant, StaticVariantType};
use gtk::prelude::*;

mod imp {

    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MQTTySettingConnection)]
    pub struct MQTTySettingConnection {
        #[property(get, set)]
        name: RefCell<String>,

        #[property(get, set)]
        url: RefCell<String>,

        #[property(get, set)]
        topic: RefCell<String>,

        #[property(get, set)]
        username: RefCell<String>,

        #[property(get, set)]
        password: RefCell<String>,

        #[property(get, set)]
        mqtt_version: RefCell<String>,

        #[property(get, set)]
        qos: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySettingConnection {
        const NAME: &'static str = "MQTTyOpenConnection";

        type Type = super::MQTTySettingConnection;

        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTySettingConnection {}
}

glib::wrapper! {
    pub struct MQTTySettingConnection(ObjectSubclass<imp::MQTTySettingConnection>);
}

impl MQTTySettingConnection {
    pub fn new(
        name: &str,
        url: &str,
        topic: &str,
        username: &str,
        password: &str,
        mqtt_version: &str,
        qos: &str,
    ) -> Self {
        glib::Object::builder()
            .property("name", name)
            .property("url", url)
            .property("topic", topic)
            .property("username", username)
            .property("password", password)
            .property("mqtt_version", mqtt_version)
            .property("qos", qos)
            .build()
    }

    /// Create a new connection profile with just URL and topic (for backwards compatibility)
    pub fn new_simple(url: &str, topic: &str) -> Self {
        Self::new("", url, topic, "", "", "3", "0")
    }
}

impl Default for MQTTySettingConnection {
    fn default() -> Self {
        Self::new("", "", "", "", "", "3", "0")
    }
}

const VARIANT_TYPE: &str = "(sssssss)";

impl StaticVariantType for MQTTySettingConnection {
    fn static_variant_type() -> std::borrow::Cow<'static, gtk::glib::VariantTy> {
        glib::VariantTy::new(VARIANT_TYPE).unwrap().into()
    }
}

/// Indexes mapping:
/// - 0 <-> name: Connection profile name
/// - 1 <-> url: URL connection
/// - 2 <-> topic: MQTT topic
/// - 3 <-> username: Authentication username
/// - 4 <-> password: Authentication password
/// - 5 <-> mqtt_version: MQTT version ("3" or "5")
/// - 6 <-> qos: Quality of Service ("0", "1", or "2")
type MQTTySettingConnectionTuple = (String, String, String, String, String, String, String);

impl FromVariant for MQTTySettingConnection {
    fn from_variant(variant: &gtk::glib::Variant) -> Option<Self> {
        let tuple = variant.get::<MQTTySettingConnectionTuple>();
        if tuple.is_none() {
            tracing::error!(
                "Could not convert from variant with format '{}', expected '{}'",
                variant.type_(),
                VARIANT_TYPE
            );
        }

        tuple.map(|tuple| tuple.into())
    }
}

impl From<MQTTySettingConnectionTuple> for MQTTySettingConnection {
    fn from(value: MQTTySettingConnectionTuple) -> Self {
        Self::new(&value.0, &value.1, &value.2, &value.3, &value.4, &value.5, &value.6)
    }
}

impl From<MQTTySettingConnection> for MQTTySettingConnectionTuple {
    fn from(value: MQTTySettingConnection) -> Self {
        (
            value.name(),
            value.url(),
            value.topic(),
            value.username(),
            value.password(),
            value.mqtt_version(),
            value.qos(),
        )
    }
}

impl From<MQTTySettingConnection> for glib::Variant {
    fn from(value: MQTTySettingConnection) -> Self {
        // Converts to tuple then to GVariant
        Into::<MQTTySettingConnectionTuple>::into(value).into()
    }
}
