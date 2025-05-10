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

use sourceview5::{prelude::{BufferExt, SearchSettingsExt, TextBufferExt}, SearchContext};

use talker::data::Data;
use talker::identifier::Id;
use talker::talker::RTalker;

use crate::session_actions;
use crate::session_presenter::RSessionPresenter;
use crate::settings;
use crate::ui::plugin_ui;

pub struct TalkerDataView {
    buffer: sourceview5::Buffer,
    source_view: sourceview5::View,
    source_view_scrolledwindow: gtk::ScrolledWindow,
    push_talker_data_button: gtk::Button,
    commit_talker_data_button: gtk::Button,
    cancel_talker_data_button: gtk::Button,
    plugin_ui_manager: RefCell<plugin_ui::Manager>,
    session_presenter: RSessionPresenter,
    language_definition_directory: std::path::PathBuf,
    search_context: SearchContext,
    search_start: gtk::TextIter,
}

impl TalkerDataView {
    pub fn new(
        session_presenter: &RSessionPresenter,
     ) -> TalkerDataView {

        let buffer = sourceview5::Buffer::builder()
            .enable_undo(true)
            .highlight_matching_brackets(true)
            .highlight_syntax(true)
            .build();

        if let Some(scheme) =
            sourceview5::StyleSchemeManager::default().scheme("classic-dark")
        {
            buffer.set_style_scheme(Some(&scheme));
        }

        let source_view = sourceview5::View::builder()
            .buffer(&buffer)
            .wrap_mode(gtk::WrapMode::Word)
            .vscroll_policy(gtk::ScrollablePolicy::Natural)
            .highlight_current_line(true)
            .monospace(true)
            .show_line_numbers(true)
            .tab_width(4)
            .hexpand(true)
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

        
        // Language management
        let mut language_definition_directory = settings::get_directory();

        match dirs::home_dir() {
            Some(home_dir) => {
                match home_dir.to_str() {
                    Some(home) => {
                        for p in sourceview5::LanguageManager::default().search_path() {
                            if p.contains(home) {
                                language_definition_directory = std::path::PathBuf::from(p);
                                break;
                            }
                        }
                    }
                    None => (),
                }
            }
            None => (),
        }

        match std::fs::create_dir_all(language_definition_directory.as_path()) {
            Ok(()) => (),
            Err(e) => println!("{}", e),
        }

        // Search context
        let search_context = SearchContext::builder().buffer(&buffer).build();
        search_context.settings().set_at_word_boundaries(false);
        search_context.settings().set_case_sensitive(true);
        search_context.settings().set_regex_enabled(false);
        search_context.settings().set_wrap_around(true);
        let search_start = buffer.start_iter();

        Self {
            buffer,
            source_view,
            source_view_scrolledwindow,
            push_talker_data_button,
            commit_talker_data_button,
            cancel_talker_data_button,
            plugin_ui_manager: RefCell::new(plugin_ui::Manager::new()),
            session_presenter: session_presenter.clone(),
            language_definition_directory,
            search_context,
            search_start,
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

    fn edit_text(&self, talker: &RTalker, text: &String) {

        self.buffer.set_text(text);
            
        if let Some(language) = talker.data_language() {
            match sourceview5::LanguageManager::new().language(&language.id) {
                Some(lang) => self.buffer.set_language(Some(&lang)),
                None => {
                    match language.definition {
                        Some(definition) => {
                            let language_definition_file = format!("{}.lang", language.id);
                            let language_definition_path = self.language_definition_directory.join(language_definition_file);

                            match File::create(&language_definition_path) {
                                Ok(mut file) => {
                                    match file.write_all(definition.as_bytes()) {
                                        Ok(()) => {
                                            if let Some(ref language) = sourceview5::LanguageManager::new().language(&language.id) {
                                                self.buffer.set_language(Some(language));
                                            }
                                        }
                                        Err(_) => (),
                                    }
                                }
                                Err(_) => (),
                            }
                        }
                        None => (),
                    }
                }
            }
        }

        self.source_view.grab_focus();

        self.source_view_scrolledwindow.set_visible(true);
        self.push_talker_data_button.set_visible(true);
        self.commit_talker_data_button.set_visible(true);
        self.cancel_talker_data_button.set_visible(true);
    }

    fn edit_file_path(&self, window: &gtk::ApplicationWindow, talker_id: Id) {
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
    }

    // Talker data editor
    pub fn edit_talker_data(&self, window: &gtk::ApplicationWindow, talker_id: Id) -> Result<(), failure::Error> {
        if let Some(talker) = self.session_presenter.borrow().find_talker(talker_id) {
            match &*talker.data().borrow() {
                Data::Int(_) => println!("Todo : Applicationview.edit_talker_data Data::Int"),
                Data::Float(_) => println!("Todo : Applicationview.edit_talker_data Data::Float"),
                Data::String(_) => println!("Todo : Applicationview.edit_talker_data Data::String"),
                Data::Text(text) => self.edit_text(talker, text),
                Data::File(_) => self.edit_file_path(window, talker_id),
                Data::UI => {
                    return self.plugin_ui_manager.borrow_mut().show(talker, &self.session_presenter);
                },
                Data::Nil => (),
            }
        }
        Ok(())
    }

    pub fn get_data(&self) -> String {
        self.buffer.text(&self.buffer.start_iter(), &self.buffer.end_iter(), false).to_string()
    }

    pub fn hide(&self) {
        self.source_view_scrolledwindow.set_visible(false);
        self.push_talker_data_button.set_visible(false);
        self.commit_talker_data_button.set_visible(false);
        self.cancel_talker_data_button.set_visible(false);
    }

    pub fn is_active(&self) -> bool {
        self.source_view_scrolledwindow.is_visible()
    }

    pub fn set_sensitive(&self, sensitive: bool) {
        self.source_view.set_sensitive(sensitive);
    }

    // Undo action
    pub fn undo(&self) {
        self.buffer.undo();
    }

    // Redo action
    pub fn redo(&self) {
        self.buffer.redo();
    }


    // Find action
    pub fn start_research(&mut self) {
        self.search_start = self.buffer.start_iter();

        self.search_context.set_highlight(true);
        self.set_sensitive(false);
    }

    pub fn find(&mut self, pattern: &str) {

        self.search_context.settings().set_search_text(Some(pattern));

        let search_result = self.search_context.forward(&self.search_start);

        if let Some((start, end, _)) = &search_result {
            self.buffer.place_cursor(start);
            self.buffer.select_range(start, end);
            self.search_start = *start;
        }
    }
    
    pub fn find_next(&mut self, backward: bool) {

        if !self.search_start.forward_char() {
            self.search_start = self.buffer.start_iter();
        }

        let search_result = if backward {
            self.search_context.backward(&self.search_start)
        }
        else {
            self.search_context.forward(&self.search_start)
        };

        if let Some((start, end, _)) = &search_result {
            self.buffer.place_cursor(start);
            self.buffer.select_range(start, end);
            self.search_start = *start;
        }
    }
    
    pub fn finish_research(&self) {
        self.search_context.set_highlight(false);
        self.set_sensitive(true);
    }
}
