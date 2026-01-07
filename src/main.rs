//! Template project for Flipper Zero.
//! This app prints "Hello, Rust!" to the console then exits.

#![no_main]
#![no_std]

// Required for panic handler
extern crate flipperzero_rt;

use core::cell::UnsafeCell;
use core::{ffi::c_void, ffi::CStr};

use flipperzero::{debug, error, info, println};
use flipperzero_rt::{entry, manifest};
use flipperzero_sys::{
    dialog_ex_alloc, dialog_ex_free, dialog_ex_get_view, dialog_ex_set_context,
    dialog_ex_set_header, dialog_ex_set_text, furi_hal_cdc_send, furi_hal_light_set,
    furi_hal_usb_reinit, furi_hal_usb_set_config, furi_record_close, furi_record_open,
    furi_timer_alloc, furi_timer_free, furi_timer_is_running, furi_timer_start, furi_timer_stop,
    usb_cdc_dual, usb_cdc_single, view_dispatcher_add_view, view_dispatcher_alloc,
    view_dispatcher_attach_to_gui, view_dispatcher_free, view_dispatcher_remove_view,
    view_dispatcher_run, view_dispatcher_set_event_callback_context,
    view_dispatcher_set_navigation_event_callback, view_dispatcher_stop,
    view_dispatcher_switch_to_view, AlignCenter, DialogEx, FuriTimer, FuriTimerTypeOnce, Gui,
    LightBlue, LightGreen, LightRed, ViewDispatcher, ViewDispatcherTypeFullscreen,
};
use flipperzero_sys::{
    furi_delay_tick, furi_get_tick, furi_hal_power_suppress_charge_enter,
    furi_hal_power_suppress_charge_exit, furi_hal_spi_bus_handle_subghz,
};

use crate::cc1101::{
    CC1101Device, BS_LIMIT, BS_PRE_KI, BS_PRE_KP, CARRIER_SENSE_ABS_THR, CARRIER_SENSE_REL_THR,
    CMD, FOC_LIMIT, FOC_PRE_K, FREQSYNTHCAL, GDO_PIN_CONFIG, MAGN_TARGET, MOD_FORMAT, NUM_PREAMBLE,
    PKTCTRL, PKT_ADDR_CHECK, PKT_FORMAT, PKT_LENGTH_CONFIG, SYNC_MODE,
};
use crate::state::{AtomicState, State};

use heapless::{format, String};

mod cc1101;
mod debug;
mod decode;
mod state;

static RECORD_GUI: &CStr = c"gui";
static APP_NAME: &CStr = c"PowerMon";
static START_TEXT: &CStr = c"Power: x.xxx W";

static BAUD_RATE: f32 = 16150.0;
static FSK_DEV: f32 = 84_000.0;
static TRANSMIT_FREQ: u32 = 433535649;
static TRANSMIT_FREQ_MHZ: f32 = (TRANSMIT_FREQ as f32) / 1_000_000f32;

// Define the FAP Manifest for this application
manifest!(
    name = "PowerMon",
    app_version = 1,
    has_icon = true,
    // See https://github.com/flipperzero-rs/flipperzero/blob/v0.11.0/docs/icons.md for icon format
    icon = "rustacean-10x10.icon",
);

// Define the entry function
entry!(main);

#[derive(Debug, Clone, Copy)]
enum VIEWS {
    MAIN = 0,
}

struct PowerMonApp<'a> {
    gui: &'a mut Gui,
    view_dispatcher: &'a mut ViewDispatcher,
    main_view: &'a mut DialogEx,
    timer: Option<&'a mut FuriTimer>,
    current_view: VIEWS,

    cc1101_device: &'a mut CC1101Device,
    last_packet: u64,
    state: AtomicState,
}

extern "C" fn view_dispatcher_navigation_callback(context: *mut c_void) -> bool {
    let power_mon_app: &mut PowerMonApp = unsafe { &mut *(context as *mut _) };
    debug!("Navigation callback to 0x{:08X}", context.addr());
    debug!(
        "Address of view dispatcher: 0x{:08X}",
        (power_mon_app.view_dispatcher as *const _ as *const c_void).addr()
    );
    debug!(
        "Address of CC1101 device: 0x{:08X}",
        (power_mon_app.cc1101_device as *const _ as *const c_void).addr()
    );
    debug!(
        "Ptr to SUBGHz Device: 0x{:08X}",
        (power_mon_app.cc1101_device.subghz as *const _ as *const c_void).addr()
    );
    power_mon_app.navigation_callback()
}

extern "C" fn timer_callback(context: *mut c_void) -> () {
    let power_mon_app: &mut PowerMonApp = unsafe { &mut *(context as *mut _) };
    debug!("Timer callback to 0x{:08X}", context.addr());
    debug!(
        "Address of view dispatcher: 0x{:08X}",
        (power_mon_app.view_dispatcher as *const _ as *const c_void).addr()
    );
    debug!(
        "Address of CC1101 device: 0x{:08X}",
        (power_mon_app.cc1101_device as *const _ as *const c_void).addr()
    );
    debug!(
        "Ptr to SUBGHz Device: 0x{:08X}",
        (power_mon_app.cc1101_device.subghz as *const _ as *const c_void).addr()
    );
    power_mon_app.timer_callback()
}

impl<'a> PowerMonApp<'a> {
    pub fn new(cc1101_device: &'a mut CC1101Device) -> Self {
        let new_self: Self;
        unsafe {
            debug!("Allocating Memory");
            new_self = Self {
                gui: &mut *(furi_record_open(RECORD_GUI.as_ptr()) as *mut Gui),
                view_dispatcher: &mut *(view_dispatcher_alloc()),
                main_view: &mut *(dialog_ex_alloc()),
                current_view: VIEWS::MAIN,
                timer: None,
                cc1101_device: cc1101_device,
                last_packet: 0,
                state: AtomicState::new(State::Initialized),
            };

            debug!("Registering GUI");
            // Register the GUI
            view_dispatcher_attach_to_gui(
                new_self.view_dispatcher,
                new_self.gui,
                ViewDispatcherTypeFullscreen,
            );
            debug!(
                "Address of state is: 0x{:08X}",
                (&new_self as *const _ as *const c_void).addr()
            );
            debug!(
                "Address of view dispatcher: 0x{:08X}",
                (new_self.view_dispatcher as *const _ as *const c_void).addr()
            );
            view_dispatcher_set_navigation_event_callback(
                new_self.view_dispatcher,
                Some(view_dispatcher_navigation_callback),
            );

            // Register the main view
            debug!("Registering View");
            view_dispatcher_add_view(
                new_self.view_dispatcher,
                VIEWS::MAIN as u32,
                dialog_ex_get_view(new_self.main_view),
            );

            // Set up the main view
            debug!("Setting Up View");
            dialog_ex_set_header(
                new_self.main_view,
                APP_NAME.as_ptr(),
                64,
                14,
                AlignCenter,
                AlignCenter,
            );
            dialog_ex_set_text(
                new_self.main_view,
                START_TEXT.as_ptr(),
                64,
                32,
                AlignCenter,
                AlignCenter,
            );

            // Set view
            debug!("Setting View");
            view_dispatcher_switch_to_view(new_self.view_dispatcher, new_self.current_view as u32);
        }

        new_self
    }

    pub fn run(&mut self) {
        let self_ptr = self as *mut _ as *mut c_void;
        if self.state.load() != State::Initialized {
            panic!("Called run from an invalid state");
        }
        unsafe {
            view_dispatcher_set_event_callback_context(self.view_dispatcher, self_ptr);
            dialog_ex_set_context(self.main_view, self_ptr);
            self.timer =
                Some(&mut *(furi_timer_alloc(Some(timer_callback), FuriTimerTypeOnce, self_ptr)));
            let timer = self.timer.as_mut().expect("");
            furi_timer_start(*timer, 100);
            self.state.store(State::Running);
            view_dispatcher_run(self.view_dispatcher);
        }
        debug!("Exited");
    }

    pub fn navigation_callback(&mut self) -> bool {
        debug!("Exiting...");

        if self.state.load() == State::Running {
            let timer = self.timer.as_mut().expect("");
            // We can exit immediately if the radio is not active
            unsafe {
                if furi_timer_is_running(*timer) == 1 {
                    furi_timer_stop(*timer);
                }
                view_dispatcher_stop(self.view_dispatcher);
            }
            self.state.store(State::Stopped);
        } else if self.state.load() == State::Reading {
            debug!("Marking stop");
            // We're in the middle of a radio read, so request stop when completed
            self.state.store(State::Stopping);
        } else {
            panic!("Called stop in an invalid state");
        }
        true
    }

    pub fn timer_callback(&mut self) -> () {
        // Move to READING unless a stop was requested in the meantime.
        let read_state = self.state.update_if(
            |s| !matches!(s, State::Stopping | State::Stopped),
            State::Reading,
        );
        if matches!(read_state, State::Stopping | State::Stopped) {
            debug!("Stop requested before read started");
            unsafe { view_dispatcher_stop(self.view_dispatcher) };
            self.state.store(State::Stopped);
            return;
        }

        debug!("Timer run! Trying to read power");
        let timer = self.timer.as_mut().expect("");
        let current_tick = get_tick();
        let interval = current_tick - self.last_packet;
        let power: Result<decode::DecodeResult, decode::DecodeError>;
        unsafe {
            furi_hal_light_set(LightRed, 255);
            furi_hal_light_set(LightGreen, 0);
            furi_hal_light_set(LightBlue, 0);
        }
        if self.last_packet == 0 || interval > 12000 {
            // Try to find the initial packet
            power = recieve_power(self.cc1101_device, 7000);
        } else {
            power = recieve_power(self.cc1101_device, 500);
        }
        unsafe {
            furi_hal_light_set(LightRed, 0);
        }

        let power_str = match power {
            Ok(decode::DecodeResult {
                power_kw: power,
                packet: _,
                quality_metric: _,
            }) => {
                let power_kw: u32 = power as u32;
                self.last_packet = get_tick();
                let power_w = ((power * 1000f32) as u32) % 1000;
                format!(16; "Power: {}.{:03} W", power_kw, power_w).expect("Format failed!")
            }
            Err(decode::DecodeError::NotEnoughData) => {
                format!(16; "Incomp. Packet").expect("Format failed!")
            }
            Err(decode::DecodeError::PreambleNotFound) => {
                format!(16; "No Preamble").expect("Format failed!")
            }
            Err(decode::DecodeError::SyncNotFound) => {
                format!(16; "No Sync").expect("Format failed!")
            }
            Err(decode::DecodeError::InsufficientSymbols) => {
                format!(16; "Incomp. Packet").expect("Format failed!")
            }
            Err(decode::DecodeError::ChecksumMismatch { expected, actual }) => {
                format!(16; "Chk {} != {}", expected, actual).expect("Format failed!")
            }
            Err(decode::DecodeError::Timeout) => format!(16; "Timeout").expect("Format failed!"),
            Err(decode::DecodeError::Overflow) => format!(16; "Overflow").expect("Format failed!"),
        };

        unsafe {
            dialog_ex_set_text(
                self.main_view,
                power_str.as_ptr(),
                64,
                32,
                AlignCenter,
                AlignCenter,
            );
            let mut mut_power_str: UnsafeCell<_> = power_str.into();
            let mut newline_str: String<2> = String::try_from("\r\n").expect("");
            let test = mut_power_str.get().as_ref().expect("");
            furi_hal_cdc_send(1, mut_power_str.get_mut().as_mut_ptr(), test.len() as u16);
            furi_hal_cdc_send(1, newline_str.as_mut_ptr(), 2);
        }

        // If we asked for a stop, then stop
        if self.state.load() == State::Stopping {
            debug!("Stopping view");
            unsafe { view_dispatcher_stop(self.view_dispatcher) };
            self.state.store(State::Stopped);
        } else {
            // Schedule the next callback
            if power.is_err() {
                self.last_packet = 0;
                unsafe { furi_timer_start(*timer, 5) };
            } else {
                unsafe { furi_timer_start(*timer, 5900) };
            }
            let next_state = self.state.update_if(
                |s| !matches!(s, State::Stopping | State::Stopped),
                State::Running,
            );
            if matches!(next_state, State::Stopping | State::Stopped) {
                debug!("Stop requested while rescheduling");
                unsafe { view_dispatcher_stop(self.view_dispatcher) };
                self.state.store(State::Stopped);
            }
        }
    }
}

impl<'a> Drop for PowerMonApp<'a> {
    fn drop(&mut self) {
        unsafe {
            // Deallocate Timer
            match &mut self.timer {
                Some(timer) => furi_timer_free(*timer),
                None => (),
            };

            // Unregister views
            view_dispatcher_remove_view(self.view_dispatcher, VIEWS::MAIN as u32);

            dialog_ex_free(self.main_view);
            view_dispatcher_free(self.view_dispatcher);
            furi_record_close(RECORD_GUI.as_ptr());

            debug!("Deallocated view");
        }
    }
}

fn configure_radio(cc1101_device: &mut CC1101Device, debug: bool) -> () {
    cc1101_device
        .gdo_config
        .set_gdo0_cfg(GDO_PIN_CONFIG::CarrierSense);
    cc1101_device.write_register(cc1101_device.gdo_config);

    cc1101_device.fifo_thr.set_fifo_thr(0xF);
    cc1101_device.write_register(cc1101_device.fifo_thr);

    cc1101_device.pktlen.set_packet_length(255);
    cc1101_device.write_register(cc1101_device.pktlen);

    cc1101_device.pktctrl = PKTCTRL::new()
        .with_pqt(0x00)
        .with_append_status(false)
        .with_adr_chk(PKT_ADDR_CHECK::NONE)
        .with_white_data(false)
        .with_pkt_format(PKT_FORMAT::NORMAL)
        .with_crc_en(false)
        .with_length_config(PKT_LENGTH_CONFIG::FIXED);
    cc1101_device.write_register(cc1101_device.pktctrl);

    cc1101_device.freq_ctrl.set_freq_mhz(TRANSMIT_FREQ_MHZ);
    cc1101_device.write_register(cc1101_device.freq_ctrl);

    cc1101_device.modem_config.set_chanbw_e(3);
    cc1101_device.modem_config.set_chanbw_m(0);
    cc1101_device.modem_config.set_mod_format(MOD_FORMAT::FSK2);
    cc1101_device.modem_config.set_manchester_en(false);
    cc1101_device
        .modem_config
        .set_sync_mode(SYNC_MODE::NO_PREAMBLE_SYNC_CS);
    cc1101_device
        .modem_config
        .set_num_preamble(NUM_PREAMBLE::P4);
    cc1101_device.modem_config.set_data_rate(BAUD_RATE * 2.0);
    cc1101_device.write_register(cc1101_device.modem_config);

    cc1101_device.deviatn.set_deviation(FSK_DEV / 2.0);
    cc1101_device.write_register(cc1101_device.deviatn);

    cc1101_device.agc_ctrl.set_magn_target(MAGN_TARGET::D40);
    cc1101_device
        .agc_ctrl
        .set_carrier_sense_rel_thr(CARRIER_SENSE_REL_THR::D14);
    cc1101_device
        .agc_ctrl
        .set_carrier_sense_abs_thr(CARRIER_SENSE_ABS_THR::P7DB);
    cc1101_device.write_register(cc1101_device.agc_ctrl);

    // Limit feedback pre sync word, since it's mostly zeros
    cc1101_device.freq_offset_comp.set_foc_bs_cs_gate(true);
    cc1101_device.freq_offset_comp.set_foc_pre_k(FOC_PRE_K::K);
    cc1101_device
        .freq_offset_comp
        .set_foc_limit(FOC_LIMIT::PM_BW_8);
    cc1101_device.write_register(cc1101_device.freq_offset_comp);

    cc1101_device.bit_sync.set_bs_pre_ki(BS_PRE_KI::KI);
    cc1101_device.bit_sync.set_bs_pre_kp(BS_PRE_KP::KP);
    cc1101_device.bit_sync.set_bs_limit(BS_LIMIT::PM0);
    cc1101_device.write_register(cc1101_device.bit_sync);

    // Set FSCAL from RF Studio
    cc1101_device.freq_synth_cal = FREQSYNTHCAL::from_bytes([0x1F, 0x00, 0x2A, 0xE9]);
    cc1101_device.write_register(cc1101_device.freq_synth_cal);

    // Set TEST register from RF Studio
    cc1101_device.test_settings.set_test0(0x09);
    cc1101_device.test_settings.set_test1(0x35);
    cc1101_device.test_settings.set_test2(0x81);
    cc1101_device.write_register(cc1101_device.test_settings);

    if debug {
        cc1101_device.print_state(true);
    }

    delay(100);
    cc1101_device.spi_send_command(CMD::SCAL);
    delay(100);
}

fn delay(delay_ms: u32) -> () {
    unsafe {
        furi_delay_tick(delay_ms);
    }
}

fn get_tick() -> u64 {
    static mut LAST_COUNTER: u32 = 0;
    static mut LAST_BASE: u64 = 0;
    let new_value: u64;
    unsafe {
        let new_counter: u32 = furi_get_tick();
        if new_counter < LAST_COUNTER {
            LAST_BASE += 1u64 << 32;
        }
        LAST_COUNTER = new_counter;
        new_value = LAST_BASE | (LAST_COUNTER as u64);
    }
    return new_value;
}

fn recieve_power(
    cc1101_device: &mut CC1101Device,
    read_timeout: u32,
) -> Result<decode::DecodeResult, decode::DecodeError> {
    let mut rx_buf = [0u8; 128];

    debug!("Starting Read");
    cc1101_device.spi_send_command(CMD::SFRX);
    unsafe {
        furi_hal_power_suppress_charge_enter();
    }
    delay(10);
    debug!("Set radio to READ mode");
    cc1101_device.spi_send_command(CMD::SRX);

    let mut timeout = read_timeout;
    while !cc1101_device.read_gdo0() {
        // Wait for GDO0 to be set -> carrier sense
        delay(1);
        if timeout == 0 {
            info!("Timeout1");
            break;
        }
        timeout -= 1;
    }
    delay(10);

    let start_time = get_tick();
    if timeout != 0 {
        let mut read_bytes: usize = 0;
        let mut elapsed_ms = get_tick() - start_time;
        let mut first_byte = 0;
        while elapsed_ms < 2000 && read_bytes < 127 {
            elapsed_ms = get_tick() - start_time;

            cc1101_device.sync_field(|dev| &mut dev.rx_bytes);
            let mut rx_bytes1: usize = cc1101_device.rx_bytes.num_rxbytes() as usize;
            // cc1101_device.sync_field(|dev| &mut dev.rx_bytes);
            //let rx_bytes2: usize = cc1101_device.rx_bytes.num_rxbytes() as usize;

            if rx_bytes1 > 0 && rx_bytes1 < 64 {
                if read_bytes + rx_bytes1 >= 127 {
                    rx_bytes1 = 127 - read_bytes;
                }
                cc1101_device
                    .spi_read_burst(0xC0 | 0x3F, &mut rx_buf[read_bytes..read_bytes + rx_bytes1]);
                if read_bytes == 0 {
                    first_byte = elapsed_ms;
                }
                read_bytes += rx_bytes1;
                delay(1);
            } else if rx_bytes1 > 64 {
                error!("RX Buffer Overflow");
                break;
            } else {
                delay(1);
            }

            if elapsed_ms - first_byte > 10 {
                break;
            }
        }
        cc1101_device.spi_send_command(CMD::SIDLE);
        cc1101_device.spi_send_command(CMD::SCAL);
        unsafe {
            furi_hal_power_suppress_charge_exit();
        }
        debug!("Read bytes: {}", read_bytes);
        if read_bytes > 0 {
            debug!("Decoding....");
            let res = decode::decode_power(&rx_buf, read_bytes);
            match res {
                Ok(decode::DecodeResult {
                    power_kw: power,
                    packet: _,
                    quality_metric: _,
                }) => info!("Power: {} W", (power * 1000.0) as u32),
                Err(decode::DecodeError::NotEnoughData) => info!("Incomplete packet"),
                Err(decode::DecodeError::PreambleNotFound) => {
                    info!("Preamble not found")
                }
                Err(decode::DecodeError::SyncNotFound) => info!("Sync not found"),
                Err(decode::DecodeError::InsufficientSymbols) => {
                    info!("Incomplete packet decoded")
                }
                Err(decode::DecodeError::ChecksumMismatch { expected, actual }) => {
                    info!("Checksum mismatch ({} != {})", expected, actual)
                }
                _ => {
                    info!("Unknown decode error")
                }
            }
            return res;
        } else if elapsed_ms >= 2000 {
            debug!("Timeout 2 {}", elapsed_ms);
            return Err(decode::DecodeError::Timeout);
        } else if read_bytes == 0 {
            debug!("No data read");
            return Err(decode::DecodeError::PreambleNotFound);
        } else {
            debug!("Overflow: {}", read_bytes);
            return Err(decode::DecodeError::Overflow);
        }
    } else {
        debug!("Timeout 3 {}", read_timeout - timeout);

        cc1101_device.spi_send_command(CMD::SIDLE);
        cc1101_device.spi_send_command(CMD::SCAL);
        unsafe {
            furi_hal_power_suppress_charge_exit();
        }

        return Err(decode::DecodeError::Timeout);
    }
}

// Entry point
fn main(_args: Option<&CStr>) -> i32 {
    // Initialize GUI
    info!("Starting Radio!");
    let mut cc1101_device: CC1101Device;
    unsafe {
        cc1101_device = CC1101Device::new(&furi_hal_spi_bus_handle_subghz);
    }
    configure_radio(&mut cc1101_device, true);
    info!("Initialized Radio!");

    debug!(
        "Address of CC1101 device: 0x{:08X}",
        (&cc1101_device as *const _ as *const c_void).addr()
    );
    debug!(
        "Ptr to SUBGHz Device: 0x{:08X}",
        (cc1101_device.subghz as *const _ as *const c_void).addr()
    );

    // Set dual USB CDC
    unsafe { furi_hal_usb_set_config(&raw mut usb_cdc_dual, core::ptr::null_mut()) };

    let mut power_mon_app = PowerMonApp::new(&mut cc1101_device);
    power_mon_app.run();

    unsafe {
        furi_hal_usb_set_config(&raw mut usb_cdc_single, core::ptr::null_mut());
        furi_hal_usb_reinit()
    };

    println!("Done, Exiting!");

    0
}
