use xcb::{x, Xid};

const X11_KEY_ESCAPE: x::Keycode = 9;

pub struct XWindow {
    conn: xcb::Connection,
    window: x::Window,
    wm_del_window: x::Atom,
    resized: bool,
}

impl XWindow {
    pub fn new(title: &str) -> Result<XWindow, failure::Error> {
        // Connect to the X server.
        let (conn, screen_num) = xcb::Connection::connect(None)?;

        // Fetch the `x::Setup` and get the main `x::Screen` object.
        let setup = conn.get_setup();
        let screen = setup.roots().nth(screen_num as usize).unwrap();

        // Generate an `Xid` for the client window.
        let window: x::Window = conn.generate_id();

        let ev_msk = x::EventMask::FOCUS_CHANGE | x::EventMask::KEY_PRESS | x::EventMask::KEY_RELEASE;

        // Window creation request
        let create_window_ck = conn.send_request_checked(&x::CreateWindow {
            depth: x::COPY_FROM_PARENT as u8,
            wid: window,
            parent: screen.root(),
            x: 0,
            y: 0,
            width: 300,
            height: 300,
            border_width: 0,
            class: x::WindowClass::InputOutput,
            visual: screen.root_visual(),
            // this list must be in same order than `Cw` enum order
            value_list: &[
                x::Cw::BackPixel(screen.black_pixel()),
                x::Cw::EventMask(ev_msk)
            ],
        });

        // WM protocols request
        let wm_protocols_ck = conn.send_request(&x::InternAtom {
            only_if_exists: true, name: b"WM_PROTOCOLS",
        });

        // Window delete protocol request
        let wm_del_window_ck = conn.send_request(&x::InternAtom {
            only_if_exists: true, name: b"WM_DELETE_WINDOW",
        });

        // Window type request
        let wt_ck = conn.send_request(&x::InternAtom {
            only_if_exists: false, name: b"_NET_WM_WINDOW_TYPE",
        });

        let wtd_ck = conn.send_request(&x::InternAtom {
            only_if_exists: false, name: b"_NET_WM_WINDOW_TYPE_DIALOG",
        });
        let wtn_ck = conn.send_request(&x::InternAtom {
            only_if_exists: false, name: b"_NET_WM_WINDOW_TYPE_NORMAL",
        });

        // PID request
        let pid_prop_ck = conn.send_request(&x::InternAtom {
            only_if_exists: false, name: b"_NET_WM_PID",
        });

        // Check if the window creation worked.
        conn.check_request(create_window_ck)?;

        // WM protocols reply
        let wm_protocols = conn.wait_for_reply(wm_protocols_ck)?.atom();

        // Window delete protocol reply
        let wm_del_window = conn.wait_for_reply(wm_del_window_ck)?.atom();

        // Window type reply
        let wt = conn.wait_for_reply(wt_ck)?.atom();

        let wtd = conn.wait_for_reply(wtd_ck)?.atom();
        let wtn = conn.wait_for_reply(wtn_ck)?.atom();

        // PID reply
        let pid_prop = conn.wait_for_reply(pid_prop_ck)?.atom();


        // Change wm protocols request
        let change_wm_protocols_ck = conn.send_request_checked(&x::ChangeProperty {
            mode: x::PropMode::Replace, window,
            property: wm_protocols, r#type: x::ATOM_ATOM, data: &[wm_del_window],
        });

        // Change window type request
        let change_window_type_ck = conn.send_request_checked(&x::ChangeProperty {
            mode: x::PropMode::Replace, window,
            property: wt, r#type: x::ATOM_CARDINAL, data: &[wtd, wtn],
        });

        // Change PID request
        let change_pid_ck = conn.send_request_checked(&x::ChangeProperty {
            mode: x::PropMode::Replace, window,
            property: pid_prop, r#type: x::ATOM_CARDINAL, data: &[std::process::id()],
        });

        // Grab key request
        let grab_key_ck = conn.send_request_checked(&x::GrabKey {
            owner_events: true,
            grab_window: window,
            modifiers: x::ModMask::ANY,
            key: X11_KEY_ESCAPE,
            pointer_mode: x::GrabMode::Async,
            keyboard_mode: x::GrabMode::Async,
        });

        // Change window name request
        let change_window_name_ck = conn.send_request_checked(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window,
            property: x::ATOM_WM_NAME,
            r#type: x::ATOM_STRING,
            data: title.as_bytes(),
        });

        conn.check_request(change_wm_protocols_ck)?;
        conn.check_request(change_window_type_ck)?;
        conn.check_request(change_pid_ck)?;
        conn.check_request(grab_key_ck)?;
        conn.check_request(change_window_name_ck)?;

        println!("New XWindow {}", window.resource_id());

        return Ok(Self { conn, window, wm_del_window, resized: false});
    }

    pub fn child_window(&self) -> Result<x::Window, failure::Error>{
        let res = self.conn.wait_for_reply_unchecked(
            self.conn.send_request_unchecked(&x::QueryTree {window: self.window}))?;
        
        if let Some(qtr) = res {
            if !qtr.children().is_empty() {
                return Ok(qtr.children()[0]);
            }
        }
        Ok(self.window)
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), failure::Error> {
        println!("XWindow {} resize : width {}, height {}", self.id(), width, height);

        if width == 0 || height == 0 {
            return Ok(());
        }

        let resize_ck = self.conn.send_request_checked(&x::ConfigureWindow {
            window: self.window,
            value_list: &[
                x::ConfigWindow::Width(width),
                x::ConfigWindow::Height(height),
            ],
        });
        self.conn.check_request(resize_ck)?;

        self.resized = true;
        Ok(())
    }

    pub fn show(&mut self) -> Result<(), failure::Error>{
        let child_window = self.child_window()?;
        println!("Child window : {:?}", child_window);

        if !self.resized {
            let child_geo = self.conn.wait_for_reply(self.conn.send_request(&x::GetGeometry {
                drawable: x::Drawable::Window(child_window),
            }))?;
            
            let width = child_geo.width() as u32;
            let height = child_geo.height() as u32;
            
            self.resize(width, height)?;
        }

        self.conn.send_request(&x::MapWindow {window: child_window});

        println!("Host window : {:?}", self.window);
        self.conn.send_request(&x::MapWindow {window: self.window});

        self.conn.flush()?;

        Ok(())
    }

    pub fn idle(&self) -> Result<bool, failure::Error> {
        let conn = &self.conn;

        match conn.poll_for_event()? {
            Some(event) => {
                match event {
                    xcb::Event::X(x::Event::KeyPress(ev)) => {
                        if ev.detail() == X11_KEY_ESCAPE {
                            return Ok(false);
                        }
                    }
                    xcb::Event::X(x::Event::ClientMessage(ev)) => {
                        // We have received a message from the server
                        if let x::ClientMessageData::Data32([atom, ..]) = ev.data() {
                            if atom == self.wm_del_window.resource_id() {
                                // The received atom is "WM_DELETE_WINDOW".
                                return Ok(false);
                            }
                        }
                    }
                    _ => {}
                }
            }
            None => ()
        }
        Ok(true)
    }

    pub fn id(&self) -> u32 {
        self.window.resource_id()
    }
}
