/*
 * Copyright (C) 2015 Gaetan Dubreil
 *
 *  All rights reserved.This file is distributed under the terms of the
 *  GNU General Public License version 3.0.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program; if not, write to the Free Software
 * Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA 02111-1307, USA.
 */
use std::cell::RefCell;
use std::fs::File;
use std::io::Write;

use gtk::prelude::*;
use gtk::gio::Cancellable;

use talker::data::Data;
use talker::identifier::Id;

use sourceview5::traits::BufferExt;

use crate::session_actions;
use crate::session_presenter::RSessionPresenter;
use crate::ui::plugin_ui;

pub struct TalkerDataView {
    source_view: sourceview5::View,
    source_view_scrolledwindow: gtk::ScrolledWindow,
    push_talker_data_button: gtk::Button,
    commit_talker_data_button: gtk::Button,
    cancel_talker_data_button: gtk::Button,
    plugin_ui_manager: RefCell<plugin_ui::Manager>,
    session_presenter: RSessionPresenter,
}

impl TalkerDataView {
    pub fn new(
        session_presenter: &RSessionPresenter,
     ) -> TalkerDataView {

        let source_view = sourceview5::View::builder()
            .wrap_mode(gtk::WrapMode::Word)
            .vscroll_policy(gtk::ScrollablePolicy::Natural)
            .highlight_current_line(true)
            .build();

        let source_view_scrolledwindow = gtk::ScrolledWindow::builder()
            .child(&source_view)
            .hadjustment(&source_view.hadjustment().unwrap())
            .vexpand(true)
            .max_content_height(256)
            .build();

        // Push talker data
        let push_talker_data_button = gtk::Button::builder()
            .icon_name("go-up")
            .action_name("session.push_talker_data")
            .tooltip_text(format!("Push talker data ({})", session_actions::PUSH_TALKER_DATA_ACCEL))
            .build();

        // Commit talker data
        let commit_talker_data_button = gtk::Button::builder()
            .icon_name("dialog-ok")
            .action_name("session.commit_talker_data")
            .tooltip_text(format!("Commit talker data ({})", session_actions::COMMIT_TALKER_DATA_ACCEL))
            .build();

        // Cancel talker data
        let cancel_talker_data_button = gtk::Button::builder()
            .icon_name("dialog-cancel")
            .action_name("session.cancel_talker_data")
            .tooltip_text(format!("Cancel talker data ({})", session_actions::CANCEL_TALKER_DATA_ACCEL))
            .build();

        Self {
            source_view,
            source_view_scrolledwindow,
            push_talker_data_button,
            commit_talker_data_button,
            cancel_talker_data_button,
            plugin_ui_manager: RefCell::new(plugin_ui::Manager::new()),
            session_presenter: session_presenter.clone(),
        }
    }

    pub fn add_tools<F>(&self, add: F)
    where F: Fn(&gtk::Button),
    {
        add(&self.push_talker_data_button);
        add(&self.commit_talker_data_button);
        add(&self.cancel_talker_data_button);
    }

    pub fn add_content<F>(&self, add: F)
    where F: Fn(&gtk::ScrolledWindow),
    {
        add(&self.source_view_scrolledwindow)
    }

    // Talker data editor
    pub fn edit_talker_data(&self, window: &gtk::ApplicationWindow, talker_id: Id) -> Result<(), failure::Error> {
        if let Some(talker) = self.session_presenter.borrow().find_talker(talker_id) {
            match &*talker.data().borrow() {
                Data::Int(_) => println!("Todo : Applicationview.edit_talker_data Data::Int"),
                Data::Float(_) => println!("Todo : Applicationview.edit_talker_data Data::Float"),
                Data::String(_) => println!("Todo : Applicationview.edit_talker_data Data::String"),
                Data::Text(data) => {
                    let text_buffer = sourceview5::Buffer::builder()
                        .enable_undo(true)
                        .highlight_matching_brackets(true)
                        .highlight_syntax(true)
                        .build();

                    if let Some(scheme) =
                        sourceview5::StyleSchemeManager::default().scheme("classic-dark")
                    {
                        text_buffer.set_style_scheme(Some(&scheme));
                    }

                    text_buffer.set_text(&data);
                    text_buffer.set_highlight_matching_brackets(true);
                    text_buffer.set_highlight_syntax(true);
                    
                    self.source_view.set_buffer(Some(&text_buffer));
                    self.source_view_scrolledwindow.set_visible(true);
                    self.push_talker_data_button.set_visible(true);
                    self.commit_talker_data_button.set_visible(true);
                    self.cancel_talker_data_button.set_visible(true);
                    self.source_view.grab_focus();
                }
                Data::File(_) => {
                    let open_ctrl = self.session_presenter.clone();

                    let dialog = gtk::FileDialog::builder()
                    .title("Choose a file")
                    .accept_label("Open")
                    .build();

                    dialog.open(Some(window), Cancellable::NONE, move |file| {
                        if let Ok(file) = file {
                            let path_buf = file.path().expect("Couldn't get file path");

                            open_ctrl.borrow_mut().set_talker_data(talker_id,&path_buf.to_string_lossy());
                        }
                    });
                },
                Data::UI => {
                    return self.plugin_ui_manager.borrow_mut().show(talker, &self.session_presenter);
                },
                Data::Nil => (),
            }
        }
        Ok(())
    }

    pub fn get_data(&self) -> String {
        let text_buffer = self.source_view.buffer();
        text_buffer.text(&text_buffer.start_iter(), &text_buffer.end_iter(), false).to_string()
    }

    pub fn hide(&self) {
        self.source_view_scrolledwindow.set_visible(false);
        self.push_talker_data_button.set_visible(false);
        self.commit_talker_data_button.set_visible(false);
        self.cancel_talker_data_button.set_visible(false);
    }

    pub fn is_active(&self) -> bool {
        self.source_view.is_focus()
    }

    // Undo action
    pub fn undo(&self) {
        self.source_view.buffer().undo();
    }

    // Redo action
    pub fn redo(&self) {
        self.source_view.buffer().redo();
    }
}
