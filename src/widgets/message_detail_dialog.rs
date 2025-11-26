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

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use sourceview::prelude::*;

use crate::client::MQTTyClientMessage;

mod imp {
    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MQTTyMessageDetailDialog)]
    pub struct MQTTyMessageDetailDialog {
        #[property(get, set)]
        pub topic: RefCell<String>,

        #[property(get, set)]
        pub formatted_body: RefCell<String>,

        #[property(get, set)]
        pub diff_text: RefCell<String>,

        #[property(get, set)]
        pub show_diff: std::cell::Cell<bool>,

        pub message: RefCell<Option<MQTTyClientMessage>>,
        pub previous_message: RefCell<Option<MQTTyClientMessage>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyMessageDetailDialog {
        const NAME: &'static str = "MQTTyMessageDetailDialog";
        type Type = super::MQTTyMessageDetailDialog;
        type ParentType = adw::Dialog;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyMessageDetailDialog {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }
    impl WidgetImpl for MQTTyMessageDetailDialog {}
    impl AdwDialogImpl for MQTTyMessageDetailDialog {}
}

glib::wrapper! {
    pub struct MQTTyMessageDetailDialog(ObjectSubclass<imp::MQTTyMessageDetailDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTyMessageDetailDialog {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    fn setup_ui(&self) {
        self.set_title("Message Details");
        self.set_content_width(600);
        self.set_content_height(500);

        let toolbar_view = adw::ToolbarView::new();

        // Header bar
        let header = adw::HeaderBar::new();
        toolbar_view.add_top_bar(&header);

        // Main content
        let notebook = gtk::Notebook::new();
        notebook.set_vexpand(true);

        // Body tab with formatted JSON
        let body_page = self.create_body_page();
        notebook.append_page(&body_page, Some(&gtk::Label::new(Some("Body"))));

        // Diff tab
        let diff_page = self.create_diff_page();
        notebook.append_page(&diff_page, Some(&gtk::Label::new(Some("Diff"))));

        toolbar_view.set_content(Some(&notebook));
        self.set_child(Some(&toolbar_view));
    }

    fn create_body_page(&self) -> gtk::Widget {
        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_vexpand(true);

        let source_view = sourceview::View::new();
        source_view.set_editable(false);
        source_view.set_monospace(true);
        source_view.set_show_line_numbers(true);
        source_view.set_wrap_mode(gtk::WrapMode::Word);

        // Set up syntax highlighting for JSON
        let buffer = source_view
            .buffer()
            .downcast::<sourceview::Buffer>()
            .unwrap();

        let lang_manager = sourceview::LanguageManager::default();
        if let Some(json_lang) = lang_manager.language("json") {
            buffer.set_language(Some(&json_lang));
        }

        let scheme_manager = sourceview::StyleSchemeManager::default();
        if let Some(scheme) = scheme_manager.scheme("Adwaita-dark") {
            buffer.set_style_scheme(Some(&scheme));
        }

        self.bind_property("formatted_body", &buffer, "text")
            .sync_create()
            .build();

        scrolled.set_child(Some(&source_view));
        scrolled.upcast()
    }

    fn create_diff_page(&self) -> gtk::Widget {
        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_vexpand(true);

        let source_view = sourceview::View::new();
        source_view.set_editable(false);
        source_view.set_monospace(true);
        source_view.set_show_line_numbers(true);
        source_view.set_wrap_mode(gtk::WrapMode::Word);

        let buffer = source_view
            .buffer()
            .downcast::<sourceview::Buffer>()
            .unwrap();

        let lang_manager = sourceview::LanguageManager::default();
        if let Some(diff_lang) = lang_manager.language("diff") {
            buffer.set_language(Some(&diff_lang));
        }

        self.bind_property("diff_text", &buffer, "text")
            .sync_create()
            .build();

        scrolled.set_child(Some(&source_view));
        scrolled.upcast()
    }

    pub fn set_message(&self, message: &MQTTyClientMessage, previous: Option<&MQTTyClientMessage>) {
        self.set_topic(message.topic());

        let body = message.body();
        let body_str = String::from_utf8_lossy(&body);

        // Try to format as JSON
        let formatted = if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&body_str) {
            serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| body_str.to_string())
        } else {
            body_str.to_string()
        };
        // Generate diff if previous message exists
        if let Some(prev) = previous {
            let prev_body = prev.body();
            let prev_str = String::from_utf8_lossy(&prev_body);

            // Try to format previous as JSON too
            let prev_formatted = if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&prev_str) {
                serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| prev_str.to_string())
            } else {
                prev_str.to_string()
            };

            let diff = self.generate_diff(&prev_formatted, &formatted);
            self.set_diff_text(diff);
            self.set_show_diff(true);
        } else {
            self.set_diff_text("No previous message to compare");
            self.set_show_diff(false);
        }

        self.set_formatted_body(formatted);

        self.imp().message.replace(Some(message.clone()));
        self.imp().previous_message.replace(previous.cloned());
    }

    fn generate_diff(&self, old: &str, new: &str) -> String {
        use similar::{ChangeTag, TextDiff};

        let diff = TextDiff::from_lines(old, new);
        let mut result = String::new();

        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            result.push_str(&format!("{}{}", sign, change));
        }

        if result.is_empty() {
            "No changes detected".to_string()
        } else {
            result
        }
    }
}

impl Default for MQTTyMessageDetailDialog {
    fn default() -> Self {
        Self::new()
    }
}
