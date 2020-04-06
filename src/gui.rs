//#[macro_use]
use gio::prelude::*;
use gtk::prelude::*;
//use gtk::gtk_sys::{GtkScrolledWindow, GtkViewport};
use gtk::{
    ApplicationWindow, BoxExt, ButtonExt, ButtonsType, CellRendererText, ContainerExt, DialogFlags,
    GtkWindowExt, HeaderBarExt, MessageDialog, MessageType, Orientation, TreeStore, TreeView,
    TreeViewColumn, WidgetExt, WindowPosition,
};

use gramotor::factory::Factory;

fn build_talkers_tree(factory: &Factory) -> Result<TreeView, failure::Error> {
    // talkers pane
    let talkers_tree = TreeView::new();
    let talkers_store = TreeStore::new(&[String::static_type(), String::static_type()]);

    talkers_tree.set_model(Some(&talkers_store));
    talkers_tree.set_headers_visible(false);

    let column = TreeViewColumn::new();
    let cell = CellRendererText::new();

    column.pack_start(&cell, true);
    column.add_attribute(&cell, "text", 0);
    talkers_tree.append_column(&column);

    for (category, talkers) in factory.get_categorized_talkers_label_model() {
        let category_iter = talkers_store.insert_with_values(None, None, &[0], &[&category]);

        for (label, model) in talkers {
            talkers_store.insert_with_values(
                Some(&category_iter),
                None,
                &[0, 1],
                &[&label, &model],
            );
        }
    }
    Ok(talkers_tree)
}

pub fn build(application: &gtk::Application) {
    let msgbox = gtk::Label::new(Some("Select a talker!"));

    match Factory::visit(|factory| build_talkers_tree(factory)) {
        Err(_) => (),
        Ok(talkers_tree) => {
            let headerbar = gtk::HeaderBar::new();
            let talkers_tree_toggle = gtk::Button::new_with_label("Talkers");

            headerbar.set_title(Some("Graffophone"));
            headerbar.set_show_close_button(true);
            headerbar.pack_start(&talkers_tree_toggle);
            /*
                        let gtkviewport = GtkViewport::new();
                        gtkviewport.add(&talkers_tree);
                        let gtkscrolledwindow = GtkScrolledWindow::new();
                        gtkscrolledwindow.add(&gtkviewport);
            */
            let split_pane = gtk::Box::new(Orientation::Horizontal, 10);

            split_pane.set_size_request(-1, -1);
            split_pane.add(&talkers_tree);
            split_pane.add(&msgbox);

            let v_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
            v_box.pack_start(&split_pane, false, false, 0);

            // Actions
            // talkers tree selection
            talkers_tree.connect_cursor_changed(move |tree_view| {
                let selection = tree_view.get_selection();
                if let Some((model, iter)) = selection.get_selected() {
                    match model.get_value(&iter, 1).get::<String>() {
                        Ok(otalker_model) => {
                            if let Some(talker_model) = otalker_model {
                                msgbox.set_text(&format!(
                                    "{}\n{}",
                                    msgbox.get_text().unwrap(),
                                    talker_model
                                ));
                            }
                        }
                        Err(_) => (),
                    };
                }
            });

            // talkers tree toggle
            talkers_tree_toggle.connect_clicked(move |_| {
                if talkers_tree.is_visible() {
                    talkers_tree.hide();
                } else {
                    talkers_tree.show();
                }
            });

            // ApplicationWindow
            let window = gtk::ApplicationWindow::new(application);

            window.set_titlebar(Some(&headerbar));
            window.set_border_width(10);
            window.set_position(gtk::WindowPosition::Center);
            window.set_default_size(1024, 768);

            window.add(&v_box);
            window.show_all();
        }
    }
}
/*
fn build_talkers_menu(application: &gtk::Application, factory: &Factory, msgbox: &gtk::Label) {
let categories_menu_bar = gio::Menu::new();

for (category, talkers) in factory.get_categorized_talkers_label_model() {
let category_menu = gio::Menu::new();

for (label, model) in talkers {
let action_name = label.replace(" ", "_");

category_menu.append(Some(&label), Some(&action_name));

let model_action = gio::SimpleAction::new(&action_name, None);
model_action.connect_activate(clone!(@weak msgbox => move |_, _| {
msgbox.set_text(&model);
            }));

            application.add_action(&model_action);
        }
        categories_menu_bar.append_submenu(Some(&category), &category_menu);
    }

    application.set_app_menu(Some(&categories_menu_bar));
}
*/
