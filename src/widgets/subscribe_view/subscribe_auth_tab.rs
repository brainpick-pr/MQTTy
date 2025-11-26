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
use gettextrs::gettext;
use gtk::{gio, glib};

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscribe_view/subscribe_auth_tab.ui")]
    #[properties(wrapper_type = super::MQTTySubscribeAuthTab)]
    pub struct MQTTySubscribeAuthTab {
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
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscribeAuthTab {
        const NAME: &'static str = "MQTTySubscribeAuthTab";

        type Type = super::MQTTySubscribeAuthTab;

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
    impl ObjectImpl for MQTTySubscribeAuthTab {}
    impl WidgetImpl for MQTTySubscribeAuthTab {}
    impl BinImpl for MQTTySubscribeAuthTab {}

    #[gtk::template_callbacks]
    impl MQTTySubscribeAuthTab {
        #[template_callback]
        fn on_ca_cert_clicked(&self) {
            self.open_file_dialog("ca_cert_path", gettext("Select CA Certificate"));
        }

        #[template_callback]
        fn on_client_cert_clicked(&self) {
            self.open_file_dialog("client_cert_path", gettext("Select Client Certificate"));
        }

        #[template_callback]
        fn on_client_key_clicked(&self) {
            self.open_file_dialog("client_key_path", gettext("Select Client Key"));
        }

        #[template_callback]
        fn on_clear_ca_cert(&self) {
            self.obj().set_ca_cert_path("");
        }

        #[template_callback]
        fn on_clear_client_cert(&self) {
            self.obj().set_client_cert_path("");
        }

        #[template_callback]
        fn on_clear_client_key(&self) {
            self.obj().set_client_key_path("");
        }

        fn open_file_dialog(&self, property: &'static str, title: String) {
            let obj = self.obj().clone();

            let filter = gtk::FileFilter::new();
            filter.add_pattern("*.pem");
            filter.add_pattern("*.crt");
            filter.add_pattern("*.key");
            filter.add_pattern("*.cer");
            filter.set_name(Some(&gettext("Certificate files")));

            let filters = gio::ListStore::new::<gtk::FileFilter>();
            filters.append(&filter);

            let dialog = gtk::FileDialog::builder()
                .title(&title)
                .filters(&filters)
                .modal(true)
                .build();

            if let Some(root) = obj.root() {
                if let Some(window) = root.downcast_ref::<gtk::Window>() {
                    dialog.open(Some(window), gio::Cancellable::NONE, move |result| {
                        if let Ok(file) = result {
                            if let Some(path) = file.path() {
                                let path_str = path.to_string_lossy().to_string();
                                match property {
                                    "ca_cert_path" => obj.set_ca_cert_path(path_str),
                                    "client_cert_path" => obj.set_client_cert_path(path_str),
                                    "client_key_path" => obj.set_client_key_path(path_str),
                                    _ => {}
                                }
                            }
                        }
                    });
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct MQTTySubscribeAuthTab(ObjectSubclass<imp::MQTTySubscribeAuthTab>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
