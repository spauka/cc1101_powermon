use crate::state::AtomicState;
use crate::string_buffer::StringBuffer;

use flipperzero::{debug};

use core::cell::UnsafeCell;
use core::ffi::c_void;
use flipperzero_sys::{
    CdcCallbacks, CdcCtrlLine, CdcState, furi_delay_tick, furi_hal_cdc_send, furi_hal_cdc_set_callbacks, furi_hal_usb_reinit, furi_hal_usb_set_config, furi_hal_usb_unlock, usb_cdc_dual, usb_cdc_line_coding, usb_cdc_single
};
use heapless::String;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum USBState {
    Uninitialized = 0,
    Idle = 1,
    Writing = 2,
    Disconnected = 3,
    Closed = 4,
}

impl From<u8> for USBState {
    #[inline]
    fn from(value: u8) -> Self {
        match value {
            0 => USBState::Uninitialized,
            1 => USBState::Idle,
            2 => USBState::Writing,
            3 => USBState::Disconnected,
            4 => USBState::Closed,
            _ => panic!("Unknown state"),
        }
    }
}

impl From<USBState> for u8 {
    #[inline]
    fn from(s: USBState) -> Self {
        s as u8
    }
}

extern "C" fn tx_ep_callback(context: *mut c_void) -> () {
    let vcp: &mut VCP = unsafe { &mut *(context as *mut _) };
    vcp.state.store(USBState::Idle);
}

extern "C" fn rx_ep_callback(context: *mut c_void) -> () {}

extern "C" fn state_callback(context: *mut c_void, state: CdcState) -> () {
    let vcp: &mut VCP = unsafe { &mut *(context as *mut _) };
    debug!("State callback {}", state.0);
    if state.0 == 0 {
        vcp.state.store(USBState::Disconnected);
        vcp.tx_buf.clear();
    } else {
        vcp.state.store(USBState::Idle)
    }
}

extern "C" fn ctrl_line_callback(context: *mut c_void, ctrl_line: CdcCtrlLine) -> () {}

extern "C" fn config_callback(context: *mut c_void, config: *mut usb_cdc_line_coding) -> () {}

pub struct VCP {
    tx_buf: StringBuffer<64>,
    write_buffer: [u8; 16],
    state: AtomicState<USBState>,
    callbacks: CdcCallbacks,
}

impl VCP {
    pub fn new() -> Self {
        debug!("Opening USB");
        unsafe {
            furi_hal_usb_unlock();
            furi_hal_usb_set_config(&raw mut usb_cdc_dual, core::ptr::null_mut());
        }
        debug!("Done Opening USB");
        Self {
            tx_buf: StringBuffer::new(),
            write_buffer: [0; 16],
            state: AtomicState::new(USBState::Uninitialized),
            callbacks: CdcCallbacks {
                tx_ep_callback: Some(tx_ep_callback),
                rx_ep_callback: None,
                state_callback: Some(state_callback),
                ctrl_line_callback: None,
                config_callback: None,
            },
        }
    }

    pub fn init(&mut self) {
        debug!("Initializing USB");
        let self_ptr = self as *mut _ as *mut c_void;
        let callback_ptr: *mut CdcCallbacks = &self.callbacks as *const _ as *mut _;
        // Set state as initialized but disconnected. Do this before seting callbacks, as we might
        // already be connected, which cdc_set_callbacks will detect.
        self.state.store(USBState::Disconnected);
        unsafe {
            furi_hal_cdc_set_callbacks(1, callback_ptr, self_ptr);
        }
    }

    pub fn write<const N: usize>(&mut self, str: String<N>) {
        let new_state = self
            .state
            .update_if(|state| state == USBState::Idle, USBState::Writing);

        match new_state {
            Ok(USBState::Idle) => {
                let strlen = str.len();
                let mut mut_str: UnsafeCell<_> = str.into();
                unsafe {
                    debug!("Writing {}", mut_str.get_mut().as_str());
                    furi_hal_cdc_send(1, mut_str.get_mut().as_mut_ptr(), strlen.try_into().unwrap());
                }
            }
            Err(USBState::Disconnected) => {
                debug!("USB Disconnected")
            },
            Err(USBState::Writing) => {
                while !matches!(self.state.load(), USBState::Idle | USBState::Disconnected) {
                    unsafe { furi_delay_tick(1); }
                }
                self.write(str);
            }
            Err(_) => {
                debug!("Unexpected state2");
                panic!("USB Port in invalid state for writing");
            },
            _ => {
                debug!("Unexpected state3");
                panic!("Impossible value!");
            },
        };
    }

    pub fn write_str<const N: usize>(&mut self, s: &str) {
        let string: String<N> = String::try_from(s).unwrap();
        self.write(string);
    }
}

impl Drop for VCP {
    fn drop(&mut self) {
        unsafe {
            furi_hal_cdc_set_callbacks(1, core::ptr::null_mut(), core::ptr::null_mut());
            furi_hal_usb_set_config(&raw mut usb_cdc_single, core::ptr::null_mut());
            furi_hal_usb_reinit()
        }
    }
}
