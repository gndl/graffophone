
use gtk;
use gtk::glib::clone;
use gtk::prelude::WidgetExt;

use gtk::prelude::*;
use gtk::Widget;

use crate::session_presenter::RSessionPresenter;
use crate::ui::talker_object::TalkerObject;

pub struct TalkersListView {
    talkers_list_toggle: gtk::ToggleButton,
    talkers_box: gtk::Box,
    talkers_box_scrolledwindow: gtk::ScrolledWindow,
    session_presenter: RSessionPresenter,
}

impl TalkersListView {
    pub fn new(session_presenter: &RSessionPresenter) -> TalkersListView {

        let talkers_list_toggle = gtk::ToggleButton::builder()
        .icon_name("view-list-tree")
        .active(true)
        .tooltip_text("Plugins list")
        .build();

        // Talkers box
        let talkers_box = gtk::Box::new(gtk::Orientation::Vertical, 2);

        let talkers_box_scrolledwindow = gtk::ScrolledWindow::builder()
            .min_content_width(256)
            .vexpand(true)
            .child(&talkers_box)
            .visible(true)
            .build();

        // talkers tree toggle
        talkers_list_toggle.connect_toggled(clone!(#[strong] talkers_box_scrolledwindow, move |tb| {
            if tb.is_active() {
                talkers_box_scrolledwindow.set_visible(true);
            } else {
                talkers_box_scrolledwindow.set_visible(false);
            }
        }));

     Self { talkers_list_toggle, talkers_box, talkers_box_scrolledwindow, session_presenter: session_presenter.clone()}
    }

    pub fn add_tools<F>(&self, add: F)
    where F: Fn(&gtk::ToggleButton),
    {
        add(&self.talkers_list_toggle);
    }

    pub fn add_content<F>(&self, add: F)
    where F: Fn(&gtk::ScrolledWindow),
    {
        add(&self.talkers_box_scrolledwindow)
    }

    // Plugins list
    pub fn fill(
        &self,
        categorized_talkers_label_model: &Vec<(String, Vec<(String, String)>)>,
    ) {
        for (category, talkers) in categorized_talkers_label_model {
            let talkers_store = gio::ListStore::new::<TalkerObject>();

            let talkers_item_factory = gtk::SignalListItemFactory::new();
            // talkers list item factory setup
            talkers_item_factory.connect_setup(move |_, list_item| {
                // Create label
                let label = gtk::Label::new(None);
                let list_item = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem");

                list_item.set_child(Some(&label));

                list_item
                    .property_expression("item")
                    .chain_property::<TalkerObject>("label")
                    .bind(&label, "label", Widget::NONE);
            });

            let talkers_selection_model = gtk::SingleSelection::new(Some(talkers_store.clone()));
            let talkers_list = gtk::ListView::builder()
                .model(&talkers_selection_model)
                .factory(&talkers_item_factory)
                .single_click_activate(true)
                .build();

            // talkers list selection
            let session_ctrl = self.session_presenter.clone();
            talkers_list.connect_activate(move |list_view, position| {
                let model = list_view.model().expect("The model has to exist.");
                let talker_object = model
                    .item(position)
                    .and_downcast::<TalkerObject>()
                    .expect("The item has to be an `TalkerObject`.");

                let talker_model = talker_object.model();

                if talker_model.is_empty() {
                } else {
                    session_ctrl.borrow_mut().add_talker(talker_model.as_str());
                }
            });

            for (label, model) in talkers {
                talkers_store.append(&TalkerObject::new(label, model));
            }
            let expander = gtk::Expander::builder()
                .label(category)
                .child(&talkers_list)
                .vexpand(false)
                .build();
            self.talkers_box.append(&expander);
        }
        self.talkers_box.append(&gtk::Label::new(None));
    }

}
