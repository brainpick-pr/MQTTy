use std::cell::RefCell;

use gtk::{gio, glib};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::MQTTyTopicItem;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(string = r#"
    <interface>
      <template class="MQTTyTopicTreeView" parent="gtk::Box">
        <property name="orientation">vertical</property>
        <child>
          <object class="gtk::ScrolledWindow">
            <property name="vexpand">true</property>
            <child>
              <object class="gtk::ColumnView" id="column_view">
                <property name="show-row-separators">true</property>
                <property name="show-column-separators">true</property>
                <child>
                    <object class="gtk::ColumnViewColumn">
                        <property name="title">Topic</property>
                        <property name="expand">true</property>
                        <property name="factory">
                            <object class="gtk::SignalListItemFactory" id="topic_factory"/>
                        </property>
                    </object>
                </child>
                <child>
                    <object class="gtk::ColumnViewColumn">
                        <property name="title">Value</property>
                        <property name="factory">
                            <object class="gtk::SignalListItemFactory" id="value_factory"/>
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

        pub root_model: RefCell<gio::ListStore>,
        pub tree_model: RefCell<Option<gtk::TreeListModel>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyTopicTreeView {
        const NAME: &'static str = "MQTTyTopicTreeView";
        type Type = super::MQTTyTopicTreeView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyTopicTreeView {
        fn constructed(&self) {
            self.parent_constructed();
            
            let obj = self.obj();
            let root = gio::ListStore::new::<MQTTyTopicItem>();
            self.root_model.replace(root.clone());

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
        let root_model = self.imp().root_model.borrow();
        
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
