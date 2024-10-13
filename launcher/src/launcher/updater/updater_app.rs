use super::component::ComponentUpdates;
use crate::launcher::http;
use crate::launcher::wstring;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;

const NOT_BOUND: &str = "RichTextBox is not yet bound to a winapi object";
const BAD_HANDLE: &str = "INTERNAL ERROR: RichTextBox handle is not HWND!";

const RED: &str = "$[255,50,50]";
const GREEN: &str = "$[35,180,75]";
const BLUE: &str = "$[50,50,255]";
const ORANGE: &str = "$[255,130,40]";
const RESET: &str = "$[reset]";

pub struct Updater {
    pub updates: Arc<Vec<ComponentUpdates>>,

    pub window: nwg::Window,
    pub layout: nwg::FlexboxLayout,
    pub scrollbuffer: nwg::RichTextBox,
    pub button: nwg::Button,

    pub scrollbuffer_tx: mpsc::Sender<String>,
    pub scrollbuffer_rx: mpsc::Receiver<String>,
    pub scrollbuffer_notice: nwg::Notice,

    pub progress_bar: nwg::ProgressBar,
    pub progress_notice: nwg::Notice,
    pub progress_tx: mpsc::Sender<f64>,
    pub progress_rx: mpsc::Receiver<f64>,
    pub cancel_update: Arc<Mutex<bool>>,
    pub thread_handle: Rc<RefCell<Option<JoinHandle<()>>>>,
    pub thread_finished: nwg::Notice,
}

impl Updater {
    pub fn new(updates: Arc<Vec<ComponentUpdates>>) -> Self {
        let (progress_tx, progress_rx) = mpsc::channel::<f64>();
        let (scrollbuffer_tx, scrollbuffer_rx) = mpsc::channel::<String>();
        Self {
            updates,
            window: Default::default(),
            layout: Default::default(),
            scrollbuffer: Default::default(),
            button: Default::default(),
            scrollbuffer_tx,
            scrollbuffer_rx,
            scrollbuffer_notice: Default::default(),
            progress_bar: Default::default(),
            progress_notice: Default::default(),
            progress_tx,
            progress_rx,
            cancel_update: Arc::new(Mutex::new(false)),
            thread_handle: Rc::new(RefCell::new(None)),
            thread_finished: Default::default(),
        }
    }

    pub fn set_scrollbuffer_handler(richedit: &nwg::RichTextBox) -> nwg::RawEventHandler {
        use winapi::um::winuser::WM_SETFOCUS;

        let handler_id = 0x10000; // handler ids equal or smaller than 0xFFFF are reserved by NWG

        nwg::bind_raw_event_handler(
            &richedit.handle,
            handler_id,
            move |_hwnd, msg, _wparam, _lparam| {
                if msg == WM_SETFOCUS {
                    unsafe {
                        winapi::um::winuser::DestroyCaret();
                        // winapi::um::winuser::HideCaret(hwnd);
                    };
                    return Some(0);
                }
                None
            },
        )
        .unwrap()
    }

    pub fn on_progress_notice(&self) {
        for progress in self.progress_rx.try_iter() {
            self.progress_bar.set_pos(progress as u32);
        }
    }

    pub fn parse_color_str(text: &str) -> Vec<(&str, Option<[u8; 3]>)> {
        let mut result = Vec::new();
        let mut current_color = None;
        let mut start = 0;

        let mut i = 0;
        while let Some(c_start) = text[i..].find("$[") {
            let c_start = i + c_start;
            if let Some(c_end) = text[c_start + 2..].find(']') {
                let c_end = c_start + 2 + c_end;
                let color_str = &text[c_start + 2..c_end];

                // Handle the text before the "$[" sequence.
                if start != c_start {
                    result.push((&text[start..c_start], current_color));
                }

                // Handle reset sequence.
                if color_str == "reset" {
                    current_color = None;
                } else {
                    // Try to parse the RGB color values.
                    let parts: Vec<&str> = color_str.split(',').collect();

                    if parts.len() == 3 {
                        if let (Ok(r), Ok(g), Ok(b)) = (
                            parts[0].trim().parse::<u8>(),
                            parts[1].trim().parse::<u8>(),
                            parts[2].trim().parse::<u8>(),
                        ) {
                            current_color = Some([r, g, b]);
                        }
                    }
                }

                // Update the start position after the processed sequence
                start = c_end + 1;
                i = start;
            } else {
                // No closing brace, treat the rest of the string as literal.
                break;
            }
        }

        // Push any remaining text with the current color.
        if start < text.len() {
            result.push((&text[start..], current_color));
        }

        result
    }

    pub fn scrollbuffer_push(&self, line: &str) {
        for (line_segment, color) in Self::parse_color_str(line) {
            let start = self.scrollbuffer.len();
            Self::append(&self.scrollbuffer, line_segment);

            if color.is_some() {
                let end = start + line_segment.len() as u32;
                self.scrollbuffer
                    .set_selection(std::ops::Range { start, end });
                self.scrollbuffer.set_char_format(&nwg::CharFormat {
                    text_color: Some(color.unwrap_or([0, 0, 0])),
                    ..Default::default()
                });
            }
        }

        Self::append(&self.scrollbuffer, "\n");
    }

    fn append(control: &nwg::RichTextBox, text: &str) {
        use winapi::um::winuser::WM_VSCROLL;
        let handle = Self::check_hwnd(&control.handle, NOT_BOUND, BAD_HANDLE);
        let wstr = wstring::Wstring::new(text);
        const EM_REPLACESEL: u32 = 0x00C2;

        control.set_selection(std::ops::Range {
            start: -2_i32 as u32,
            end: -1_i32 as u32,
        });

        Self::send_message(handle, EM_REPLACESEL, 0, wstr.as_ptr() as LPARAM);

        Self::send_message(
            handle,
            WM_VSCROLL,
            winapi::um::winuser::SB_BOTTOM as WPARAM,
            0,
        );
    }

    fn check_hwnd(handle: &nwg::ControlHandle, not_bound: &str, bad_handle: &str) -> HWND {
        use winapi::um::winuser::IsWindow;

        if handle.blank() {
            panic!("{}", not_bound);
        }
        match handle.hwnd() {
            Some(hwnd) => match unsafe { IsWindow(hwnd) } {
                0 => {
                    panic!("The window handle is no longer valid. This usually means the control was freed by the OS");
                }
                _ => hwnd,
            },
            None => {
                panic!("{}", bad_handle);
            }
        }
    }

    fn send_message(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT {
        unsafe { winapi::um::winuser::SendMessageW(hwnd, msg, w, l) }
    }

    pub fn on_scrollbuffer_notice(&self) {
        for line in self.scrollbuffer_rx.try_iter() {
            self.scrollbuffer_push(line.as_str());
        }
    }

    pub fn start_download(&self) {
        let scrollbuffer_sender = self.scrollbuffer_tx.clone();
        let scrollbuffer_noticer = self.scrollbuffer_notice.sender();

        let progress_sender = self.progress_tx.clone();
        let progress_noticer = self.progress_notice.sender();

        let cancel_update = Arc::clone(&self.cancel_update);
        let updates = Arc::clone(&self.updates);

        let thread_noticer = self.thread_finished.sender();

        let thread_handle = thread::spawn(move || {
            let progress_callback = Self::create_progress_callback(
                progress_sender.clone(),
                progress_noticer,
                cancel_update,
            );

            let status_report = |status: String| {
                Self::send_to_ui(&scrollbuffer_sender, &scrollbuffer_noticer, status);
            };

            let mut all_ok = true;
            for (updates, component) in updates.as_ref() {
                status_report(format!("{BLUE}Updating {}...", component.name()));

                Self::send_to_ui(&progress_sender, &progress_noticer, 0.0);

                match component.update(updates, &status_report, &progress_callback) {
                    Err(err) => match err.downcast_ref::<curl::Error>() {
                        // User abort
                        Some(curl_err) if curl_err.is_aborted_by_callback() => {
                            status_report(format!("{ORANGE}Update aborted"));
                            return;
                        }
                        // Update error
                        _ => {
                            all_ok = false;
                            status_report(format!(
                                "{RED}Failed to update {}: {}",
                                component.name(),
                                err
                            ));
                        }
                    },
                    // Update successful
                    Ok(_) => {
                        status_report(format!(
                            "{GREEN}Successfully updated{RESET} {}\n",
                            component.name()
                        ));
                        Self::send_to_ui(&progress_sender, &progress_noticer, 100.0);
                    }
                };
            }

            match all_ok {
                true => status_report(format!("{GREEN}Update successful")),
                false => status_report(format!("{RED}There were some errors during updating")),
            }
            thread_noticer.notice();
        });

        *self.thread_handle.borrow_mut() = Some(thread_handle);
    }

    pub fn cancel_update(&self) {
        if let Ok(mut cancel_update) = self.cancel_update.lock() {
            *cancel_update = true;
        }
    }

    fn create_progress_callback(
        sender: mpsc::Sender<f64>,
        noticer: nwg::NoticeSender,
        cancel_update: Arc<Mutex<bool>>,
    ) -> http::ProgressCallback {
        http::ProgressCallback::new(move |p: f64| {
            if sender.send(p).is_ok() {
                noticer.notice();
            }
            match cancel_update.lock() {
                Ok(cancel) => !*cancel,
                _ => true, // continue
            }
        })
    }

    fn send_to_ui<T>(sender: &mpsc::Sender<T>, noticer: &nwg::NoticeSender, data: T) {
        if sender.send(data).is_ok() {
            noticer.notice();
        }
    }
}
