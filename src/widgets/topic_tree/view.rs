use std::cell::{OnceCell, RefCell};
use std::sync::LazyLock;

use gettextrs::gettext;
use gtk::{gio, glib};
use gtk::glib::subclass::Signal;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::MQTTyTopicItem;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(string = r#"
    <interface>
      <template class="MQTTyTopicTreeView" parent="GtkBox">
        <property name="orientation">vertical</property>
        <child>
          <object class="GtkScrolledWindow">
            <property name="vexpand">true</property>
            <child>
              <object class="GtkColumnView" id="column_view">
                <property name="show-row-separators">true</property>
                <property name="show-column-separators">true</property>
                <child>
                    <object class="GtkColumnViewColumn">
                        <property name="title">Topic</property>
                        <property name="expand">true</property>
                        <property name="factory">
                            <object class="GtkSignalListItemFactory" id="topic_factory"/>
                        </property>
                    </object>
                </child>
                <child>
                    <object class="GtkColumnViewColumn">
                        <property name="title">Value</property>
                        <property name="factory">
                            <object class="GtkSignalListItemFactory" id="value_factory"/>
                        </property>
                    </object>
                </child>
              </object>
            </child>
          </object>
        </child>
      </template>
    </interface>
    "#)]
    #[properties(wrapper_type = super::MQTTyTopicTreeView)]
    pub struct MQTTyTopicTreeView {
        #[template_child]
        pub column_view: TemplateChild<gtk::ColumnView>,

        pub root_model: OnceCell<gio::ListStore>,
        pub tree_model: RefCell<Option<gtk::TreeListModel>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyTopicTreeView {
        const NAME: &'static str = "MQTTyTopicTreeView";
        type Type = super::MQTTyTopicTreeView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyTopicTreeView {
        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![
                    Signal::builder("clear-retained-requested")
                        .param_types([String::static_type()])
                        .build(),
                ]
            });
            &SIGNALS
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            let root = gio::ListStore::new::<MQTTyTopicItem>();
            let _ = self.root_model.set(root.clone());

            let tree_model = gtk::TreeListModel::new(
                root,
                false,
                false,
                |item| {
                    let item = item.downcast_ref::<MQTTyTopicItem>().unwrap();
                    Some(item.children().upcast())
                }
            );
            
            let selection = gtk::SingleSelection::new(Some(tree_model.clone()));
            self.column_view.set_model(Some(&selection));
            self.tree_model.replace(Some(tree_model));

            // Setup Factories
            // Topic Column (with TreeExpander)
            let topic_factory = obj.imp().column_view.columns().item(0).unwrap().downcast::<gtk::ColumnViewColumn>().unwrap().factory().unwrap().downcast::<gtk::SignalListItemFactory>().unwrap();
            
            topic_factory.connect_setup(|_, list_item| {
                let expander = gtk::TreeExpander::new();
                let label = gtk::Label::new(None);
                label.set_halign(gtk::Align::Start);
                expander.set_child(Some(&label));
                list_item.downcast_ref::<gtk::ListItem>().unwrap().set_child(Some(&expander));
            });

            topic_factory.connect_bind(|_, list_item| {
                let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                let expander = list_item.child().unwrap().downcast::<gtk::TreeExpander>().unwrap();
                let row = list_item.item().and_downcast::<gtk::TreeListRow>().unwrap();
                let item = row.item().and_downcast::<MQTTyTopicItem>().unwrap();

                expander.set_list_row(Some(&row));
                let label = expander.child().unwrap().downcast::<gtk::Label>().unwrap();
                item.bind_property("name", &label, "label").sync_create().build();
            });

            // Value Column
            let value_factory = obj.imp().column_view.columns().item(1).unwrap().downcast::<gtk::ColumnViewColumn>().unwrap().factory().unwrap().downcast::<gtk::SignalListItemFactory>().unwrap();
            
            value_factory.connect_setup(|_, list_item| {
                let label = gtk::Label::new(None);
                label.set_halign(gtk::Align::Start);
                label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                list_item.downcast_ref::<gtk::ListItem>().unwrap().set_child(Some(&label));
            });

            value_factory.connect_bind(|_, list_item| {
                let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                let label = list_item.child().unwrap().downcast::<gtk::Label>().unwrap();
                let row = list_item.item().and_downcast::<gtk::TreeListRow>().unwrap();
                let item = row.item().and_downcast::<MQTTyTopicItem>().unwrap();

                item.bind_property("payload", &label, "label").sync_create().build();
            });

            // Setup context menu for right-click
            self.setup_context_menu();
        }
    }

    impl MQTTyTopicTreeView {
        fn setup_context_menu(&self) {
            let obj = self.obj();

            // Create popover menu
            let menu = gio::Menu::new();
            menu.append(Some(&gettext("Clear Retained Message")), Some("topic-tree.clear-retained"));

            let popover = gtk::PopoverMenu::from_model(Some(&menu));
            popover.set_parent(&*self.column_view);
            popover.set_has_arrow(false);

            // Install action
            let action_group = gio::SimpleActionGroup::new();

            let clear_action = gio::SimpleAction::new("clear-retained", None);
            let obj_weak = obj.downgrade();
            let selection_clone = self.column_view.model().unwrap().downcast::<gtk::SingleSelection>().ok();
            clear_action.connect_activate(move |_, _| {
                if let Some(obj) = obj_weak.upgrade() {
                    if let Some(selection) = &selection_clone {
                        if let Some(row) = selection.selected_item().and_downcast::<gtk::TreeListRow>() {
                            if let Some(item) = row.item().and_downcast::<MQTTyTopicItem>() {
                                let topic = item.full_topic();
                                obj.emit_by_name::<()>("clear-retained-requested", &[&topic]);
                            }
                        }
                    }
                }
            });
            action_group.add_action(&clear_action);
            obj.insert_action_group("topic-tree", Some(&action_group));

            // Setup gesture for right-click
            let gesture = gtk::GestureClick::new();
            gesture.set_button(3); // Right-click
            let popover_clone = popover.clone();
            gesture.connect_released(move |gesture, _, x, y| {
                let widget = gesture.widget();
                let point = gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
                popover_clone.set_pointing_to(Some(&point));
                popover_clone.popup();
            });
            self.column_view.add_controller(gesture);
        }
    }

    impl WidgetImpl for MQTTyTopicTreeView {}
    impl BoxImpl for MQTTyTopicTreeView {}
}

glib::wrapper! {
    pub struct MQTTyTopicTreeView(ObjectSubclass<imp::MQTTyTopicTreeView>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl MQTTyTopicTreeView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn process_message(&self, topic: &str, payload: &str) {
        let parts: Vec<&str> = topic.split('/').collect();
        let Some(root_model) = self.imp().root_model.get() else {
            return;
        };

        let mut current_model = root_model.clone();
        let mut full_path = String::new();

        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                full_path.push('/');
            }
            full_path.push_str(part);

            let mut found = None;
            for j in 0..current_model.n_items() {
                if let Some(item) = current_model.item(j).and_downcast::<MQTTyTopicItem>() {
                    if item.name() == *part {
                        found = Some(item);
                        break;
                    }
                }
            }

            let item = if let Some(item) = found {
                item
            } else {
                let new_item = MQTTyTopicItem::new(part, &full_path);
                current_model.append(&new_item);
                new_item
            };

            if i == parts.len() - 1 {
                item.set_payload(payload);
            }

            current_model = item.children();
        }
    }
}
