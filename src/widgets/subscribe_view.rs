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

mod message_row;
mod subscribe_auth_tab;
mod subscribe_general_tab;
mod subscribe_view_notebook;

pub use message_row::MQTTyMessageRow;
pub use subscribe_auth_tab::MQTTySubscribeAuthTab;
pub use subscribe_general_tab::MQTTySubscribeGeneralTab;
pub use subscribe_view_notebook::MQTTySubscribeViewNotebook;

use std::cell::Cell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use formatx::formatx;
use gettextrs::gettext;
use gtk::glib;

use crate::application::MQTTyApplication;
use crate::display_mode::{MQTTyDisplayMode, MQTTyDisplayModeIface};
use crate::main_window::MQTTyWindow;
use crate::subclass::prelude::*;

mod imp {

    use crate::toast::MQTTyToastBuilder;

    use super::*;

    #[derive(gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscribe_view/subscribe_view.ui")]
    #[properties(wrapper_type = super::MQTTySubscribeView)]
    pub struct MQTTySubscribeView {
        #[property(get, set, override_interface = MQTTyDisplayModeIface)]
        display_mode: Cell<MQTTyDisplayMode>,

        #[template_child]
        tab_view: TemplateChild<adw::TabView>,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,

        #[template_child]
        subscribe_button: TemplateChild<gtk::Button>,
    }

    impl Default for MQTTySubscribeView {
        fn default() -> Self {
            Self {
                display_mode: Cell::new(MQTTyDisplayMode::Desktop),
                tab_view: Default::default(),
                stack: Default::default(),
                subscribe_button: Default::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscribeView {
        const NAME: &'static str = "MQTTySubscribeView";

        type Type = super::MQTTySubscribeView;

        type ParentType = adw::Bin;

        type Interfaces = (MQTTyDisplayModeIface,);

        fn class_init(klass: &mut Self::Class) {
            klass.install_action("subscribe-view.new-tab", None, |this, _, _| {
                let notebook = MQTTySubscribeViewNotebook::new();
                this.bind_property("display_mode", &notebook, "display_mode")
                    .sync_create()
                    .build();

                let topic_expr = notebook
                    .property_expression_weak("topic")
                    .chain_closure::<String>(glib::closure!(
                        move |_: Option<glib::Object>, topic: String| {
                            if topic.is_empty() {
                                gettext("(untitled)")
                            } else {
                                topic
                            }
                        }
                    ));

                let page = this.imp().tab_view.append(&notebook);

                topic_expr.bind(&page, "title", glib::Object::NONE);

                // We create a tooltip based on topic and url values
                gtk::ClosureExpression::new::<String>(
                    [
                        topic_expr.upcast(),
                        notebook.property_expression_weak("url").upcast(),
                    ],
                    glib::closure!(move |_: Option<glib::Object>, topic: String, url: String| {
                        if url.is_empty() {
                            topic
                        } else {
                            [topic, url].join("\r\n")
                        }
                    }),
                )
                .bind(&page, "tooltip", glib::Object::NONE);
            });

            klass.install_action("subscribe-view.subscribe", None, |this, _, _| {
                let notebook = this
                    .imp()
                    .tab_view
                    .selected_page()
                    .unwrap()
                    .child()
                    .downcast::<MQTTySubscribeViewNotebook>()
                    .unwrap();

                let app = MQTTyApplication::get_singleton();

                let window = app
                    .active_window()
                    .unwrap()
                    .downcast::<MQTTyWindow>()
                    .unwrap();

                let subscribing_toast = MQTTyToastBuilder::new()
                    .timeout(2)
                    .title(gettext("Subscribing to topic..."))
                    .build();

                window.toast(&subscribing_toast);

                glib::spawn_future_local(async move {
                    let ret = notebook.subscribe().await;

                    subscribing_toast.dismiss();

                    let toast = match ret {
                        Ok(_) => MQTTyToastBuilder::new()
                            .title(
                                formatx!(
                                    gettext("Subscribed to topic {}"),
                                    notebook.topic()
                                )
                                .unwrap(),
                            )
                            .icon(
                                gtk::Image::builder()
                                    .icon_name("object-select-symbolic")
                                    .css_classes(["success"])
                                    .build()
                                    .as_ref(),
                            )
                            .timeout(2)
                            .build(),

                        Err(e) => MQTTyToastBuilder::new()
                            .title(formatx!(gettext("Error while subscribing: {}"), e).unwrap())
                            .icon(
                                gtk::Image::builder()
                                    .icon_name("network-error-symbolic")
                                    .build()
                                    .as_ref(),
                            )
                            .timeout(2)
                            .build(),
                    };

                    window.toast(&toast);
                });
            });

            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTySubscribeView {
        fn constructed(&self) {
            self.parent_constructed();

            let stack = &self.stack;
            let subscribe_button = &self.subscribe_button;

            self.tab_view.connect_n_pages_notify(glib::clone!(
                #[weak]
                stack,
                #[weak]
                subscribe_button,
                move |tab_view| {
                    let n_pages = tab_view.n_pages();
                    stack.set_visible_child_name(if n_pages == 0 { "no-tabs" } else { "tabs" });

                    subscribe_button.set_visible(n_pages != 0);
                }
            ));
        }
    }
    impl WidgetImpl for MQTTySubscribeView {}
    impl BinImpl for MQTTySubscribeView {}
    impl MQTTyDisplayModeIfaceImpl for MQTTySubscribeView {}
}

glib::wrapper! {
    pub struct MQTTySubscribeView(ObjectSubclass<imp::MQTTySubscribeView>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
