use super::component::ComponentUpdates;
use super::updater_app::Updater;
use crate::launcher::wstring;

use nwg::NativeUi;

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

pub fn run_gui(updates: Arc<Vec<ComponentUpdates>>) -> anyhow::Result<()> {
    nwg::init()?;
    //nwg::Font::set_global_family("Segoe UI")?;
    let mut font = nwg::Font::default();
    nwg::Font::builder()
        .family("Segoe UI")
        .size(20)
        .build(&mut font)?;
    nwg::Font::set_global_default(Some(font));
    enable_visual_styles();
    let _updater = Updater::build_ui(Updater::new(updates))?;
    nwg::dispatch_thread_events();

    Ok(())
}

fn enable_visual_styles() {
    use winapi::shared::basetsd::ULONG_PTR;
    use winapi::shared::minwindef::{DWORD, ULONG};
    use winapi::um::libloaderapi::GetModuleHandleW;
    use winapi::um::winbase::{ActivateActCtx, CreateActCtxW, ACTCTXW};
    use winapi::um::winuser::MAKEINTRESOURCEW;

    const ACTCTX_FLAG_HMODULE_VALID: DWORD = 0x080;
    const ACTCTX_FLAG_RESOURCE_NAME_VALID: DWORD = 0x008;

    unsafe {
        let mut activation_cookie: ULONG_PTR = 0;
        let act_ctx = ACTCTXW {
            cbSize: std::mem::size_of::<ACTCTXW>() as ULONG,
            dwFlags: ACTCTX_FLAG_HMODULE_VALID | ACTCTX_FLAG_RESOURCE_NAME_VALID,
            lpSource: std::ptr::null(),
            wProcessorArchitecture: 0,
            wLangId: 0,
            lpAssemblyDirectory: std::ptr::null(),
            lpResourceName: MAKEINTRESOURCEW(1),
            lpApplicationName: std::ptr::null(),
            hModule: GetModuleHandleW(wstring::Wstring::new("launcher").into()),
        };

        let handle = CreateActCtxW(&act_ctx);
        ActivateActCtx(handle, &mut activation_cookie);
    }
}

pub struct UpdaterGui {
    inner: Rc<Updater>,
    default_handler: RefCell<Option<nwg::EventHandler>>,
}

impl nwg::NativeUi<UpdaterGui> for Updater {
    fn build_ui(mut data: Updater) -> Result<UpdaterGui, nwg::NwgError> {
        use nwg::Event as E;

        const WINDOW_WIDTH: i32 = 450;
        const WINDOW_HEIGHT: i32 = 300;

        let screen_width = nwg::Monitor::width();
        let screen_height = nwg::Monitor::height();

        let center_x = (screen_width - WINDOW_WIDTH) / 2;
        let center_y = (screen_height - WINDOW_HEIGHT) / 2;

        nwg::Window::builder()
            .flags(
                nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE/* | nwg::WindowFlags::RESIZABLE*/,
            )
            .size((WINDOW_WIDTH, WINDOW_HEIGHT))
            .position((center_x, center_y))
            .title("CoD4x Updater")
            .build(&mut data.window)?;

        // Controls
        nwg::Button::builder()
            .size((280, 70))
            .text("Cancel")
            .parent(&data.window)
            .build(&mut data.button)?;

        nwg::Notice::builder()
            .parent(&data.window)
            .build(&mut data.scrollbuffer_notice)?;

        nwg::RichTextBox::builder()
            .flags(
                nwg::RichTextBoxFlags::VISIBLE
                    | nwg::RichTextBoxFlags::VSCROLL
                    | nwg::RichTextBoxFlags::AUTOVSCROLL
                    | nwg::RichTextBoxFlags::SAVE_SELECTION,
            )
            .readonly(true)
            .size((280, 25))
            .parent(&data.window)
            .build(&mut data.scrollbuffer)?;

        nwg::ProgressBar::builder()
            .size((280, 25))
            .parent(&data.window)
            .build(&mut data.progress_bar)?;

        nwg::Notice::builder()
            .parent(&data.window)
            .build(&mut data.progress_notice)?;

        nwg::Notice::builder()
            .parent(&data.window)
            .build(&mut data.thread_finished)?;

        // Wrap-up
        let ui = UpdaterGui {
            inner: Rc::new(data),
            default_handler: Default::default(),
        };

        // Events
        let evt_ui = Rc::downgrade(&ui.inner);

        let thread_handle = Rc::clone(&ui.inner.thread_handle);

        let handle_events = move |evt, _evt_data, handle: nwg::ControlHandle| {
            if let Some(ui) = evt_ui.upgrade() {
                match evt {
                    E::OnInit => {
                        if handle == ui.window {
                            Updater::start_download(&ui);
                        }
                    }
                    E::OnButtonClick => {
                        if handle == ui.button {
                            if ui.button.text() == "Cancel" {
                                Updater::cancel_update(&ui);
                                ui.button.set_text("OK");
                            } else if ui.button.text() == "OK" {
                                ui.window.close();
                            }
                        }
                    }
                    E::OnWindowClose => {
                        if handle == ui.window {
                            Updater::cancel_update(&ui);
                            nwg::stop_thread_dispatch();
                        }
                    }
                    E::OnNotice => {
                        if handle == ui.progress_notice {
                            Updater::on_progress_notice(&ui);
                        } else if handle == ui.scrollbuffer_notice {
                            Updater::on_scrollbuffer_notice(&ui);
                        } else if handle == ui.thread_finished {
                            if let Some(join_handle) = thread_handle.borrow_mut().take() {
                                join_handle.join().ok(); // We don't care about any errors here
                                ui.button.set_text("OK");
                            }
                        }
                    }
                    _ => {}
                }
            }
        };

        *ui.default_handler.borrow_mut() = Some(nwg::full_bind_event_handler(
            &ui.window.handle,
            handle_events,
        ));

        use nwg::stretch::{
            geometry::Size,
            style::{AlignSelf, Dimension as D, FlexDirection},
        };

        nwg::FlexboxLayout::builder()
            .parent(&ui.window)
            .flex_direction(FlexDirection::Column)
            // ScrollBuffer
            .child(&ui.scrollbuffer)
            .child_flex_grow(2.0)
            .child_size(Size {
                width: D::Auto,
                height: D::Percent(0.8),
            })
            // ProgressBar
            .child(&ui.progress_bar)
            .child_size(Size {
                width: D::Auto,
                height: D::Points(25.0),
            })
            // Button
            .child(&ui.button)
            .child_align_self(AlignSelf::FlexEnd)
            .child_size(Size {
                width: D::Points(80.0),
                height: D::Points(25.0),
            })
            .build(&ui.layout)?;

        Updater::set_scrollbuffer_handler(&ui.scrollbuffer);

        Ok(ui)
    }
}

impl Drop for UpdaterGui {
    /// To make sure that everything is freed without issues, the default handler must be unbound.
    fn drop(&mut self) {
        let handler = self.default_handler.borrow();
        if let Some(h) = handler.as_ref() {
            nwg::unbind_event_handler(h);
        }
    }
}

impl Deref for UpdaterGui {
    type Target = Updater;

    fn deref(&self) -> &Updater {
        &self.inner
    }
}
