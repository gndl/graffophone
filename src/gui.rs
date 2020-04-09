//#[macro_use]
// use std::cell::RefCell;
// use std::rc::Rc;

//use gdk::EventMask;
use gio::prelude::*;
//use gtk::gtk_sys::GtkScrolledWindow;
use gtk::prelude::*;
use gtk::{
    BoxExt, ButtonExt, CellRendererText, ContainerExt, GtkWindowExt, HeaderBarExt, IconSize,
    ScrolledWindowExt, TreeStore, TreeView, TreeViewColumn, WidgetExt,
};

//use cairo::enums::{FontSlant, FontWeight};
//use cairo::Context;

use gramotor::factory::Factory;

use crate::graph_controler::RGraphControler;
use crate::graph_view::GraphView;

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

pub fn build(application: &gtk::Application, graph_controler: RGraphControler) {
    let selected_talker_label = gtk::Label::new(Some("Selected talkers :"));

    match Factory::visit(|factory| build_talkers_tree(factory)) {
        Err(_) => (),
        Ok(talkers_tree) => {
            // header bar
            let headerbar = gtk::HeaderBar::new();
            let talkers_tree_toggle =
                gtk::Button::new_from_icon_name(Some("gtk-index"), IconSize::SmallToolbar);

            headerbar.set_title(Some("Graffophone"));
            headerbar.set_show_close_button(true);

            // header bar left controls
            headerbar.pack_start(&talkers_tree_toggle);

            // header bar right controls
            let stop_button =
                gtk::Button::new_from_icon_name(Some("gtk-media-stop"), IconSize::SmallToolbar);

            headerbar.pack_end(&stop_button);

            let play_button =
                gtk::Button::new_from_icon_name(Some("gtk-media-play"), IconSize::SmallToolbar);

            headerbar.pack_end(&play_button);

            // Split pane
            let split_pane = gtk::Box::new(gtk::Orientation::Horizontal, 10);

            //            split_pane.set_size_request(-1, -1);

            // Talkers tree
            let talkers_tree_scrolledwindow =
                gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
            talkers_tree_scrolledwindow.set_min_content_width(256);
            //            talkers_tree_scrolledwindow.set_vexpand(true);
            talkers_tree_scrolledwindow.add(&talkers_tree);
            split_pane.pack_start(&talkers_tree_scrolledwindow, false, true, 0);

            // Graph view
            let graph_view = GraphView::new_ref(graph_controler.clone());

            //            let graph_area = DrawingArea::new();

            let graph_view_scrolledwindow =
                gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
            graph_view_scrolledwindow.add(graph_view.borrow().area());
            split_pane.pack_start(&graph_view_scrolledwindow, true, true, 0);

            // Vertical box
            let v_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
            v_box.pack_start(&split_pane, true, true, 0);
            v_box.pack_start(&selected_talker_label, false, false, 0);

            // Actions
            // talkers tree selection
            talkers_tree.connect_cursor_changed(move |tree_view| {
                let selection = tree_view.get_selection();

                if let Some((model, iter)) = selection.get_selected() {
                    match model.get_value(&iter, 1).get::<String>() {
                        Ok(otalker_model) => match otalker_model {
                            Some(talker_model) => {
                                graph_controler.borrow_mut().add_talker(&talker_model);
                                graph_view.borrow().draw();
                            }
                            None => {
                                if let Some(path) = model.get_path(&iter) {
                                    if tree_view.row_expanded(&path) {
                                        tree_view.collapse_row(&path);
                                    } else {
                                        tree_view.expand_row(&path, true);
                                    }
                                }
                            }
                        },
                        Err(e) => eprintln!("{}", e),
                    };
                }
            });

            // talkers tree toggle
            talkers_tree_toggle.connect_clicked(move |_| {
                if talkers_tree_scrolledwindow.is_visible() {
                    talkers_tree_scrolledwindow.hide();
                } else {
                    talkers_tree_scrolledwindow.show();
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
