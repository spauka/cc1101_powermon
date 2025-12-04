//! Template project for Flipper Zero.
//! This app prints "Hello, Rust!" to the console then exits.

#![no_main]
#![no_std]

// Required for panic handler
extern crate flipperzero_rt;

use core::ffi::CStr;
use core::ptr::{copy_nonoverlapping, null, null_mut};
use core::result::Result;

use flipperzero::{format, debug, info, error, println};
use flipperzero_rt::{entry, manifest};
use flipperzero_sys::{FuriHalSpiBusHandle, SubGhzDeviceCC1101Int, memcpy};
use flipperzero_sys::{
    furi_delay_ms, furi_delay_tick, furi_hal_gpio_init, furi_hal_gpio_read,
    furi_hal_power_suppress_charge_enter, furi_hal_power_suppress_charge_exit,
    furi_hal_spi_acquire, furi_hal_spi_bus_handle_subghz, furi_hal_spi_bus_trx,
    furi_hal_spi_release, subghz_devices_begin, subghz_devices_deinit, subghz_devices_end,
    subghz_devices_flush_rx, subghz_devices_get_by_name, subghz_devices_get_data_gpio,
    subghz_devices_idle, subghz_devices_init, subghz_devices_is_frequency_valid,
    subghz_devices_load_preset, subghz_devices_reset, subghz_devices_set_frequency,
    subghz_devices_set_tx, subghz_devices_sleep, subghz_devices_write_packet,
    subghz_tx_rx_worker_alloc, subghz_tx_rx_worker_free, subghz_tx_rx_worker_is_running,
    subghz_tx_rx_worker_set_callback_have_read, subghz_tx_rx_worker_start,
    subghz_tx_rx_worker_stop, subghz_tx_rx_worker_write, FuriHalSubGhzPreset2FSKDev476Async,
    FuriHalSubGhzPresetGFSK9_99KbAsync, GpioModeInput, GpioPullNo, GpioSpeedLow,
};

use crate::cc1101::{CC1101Device, Register};

use core::fmt::Write as FmtWrite;
mod cc1101;

const MAX_SPI_BUF: usize = 65; // allow room for command byte + up to 64 payload bytes

static SUBGHZ_DEVICE_CC1101_INT_NAME: &CStr = c"cc1101_int";
static TRANSMIT_FREQ: u32 = 433500000 + 74000 / 2;

// Define the FAP Manifest for this application
manifest!(
    name = "Flipper Zero Rust",
    app_version = 1,
    has_icon = true,
    // See https://github.com/flipperzero-rs/flipperzero/blob/v0.11.0/docs/icons.md for icon format
    icon = "rustacean-10x10.icon",
);

// Define the entry function
entry!(main);

fn init_radio(subghz: *const flipperzero_sys::SubGhzDevice) -> Result<(), ()> {
    Ok(())
}

fn spi_read_burst(handle: *const FuriHalSpiBusHandle, out_buf: &mut [u8], addr: u8, read_len: usize) -> () {
	// read_len is the number of payload bytes requested
	if read_len == 0 {
		return;
	}
	if read_len + 1 > MAX_SPI_BUF {
		error!("Requested read length {} exceeds MAX_SPI_BUF-1", read_len);
		return;
	}
	if out_buf.len() < read_len {
		error!("Output buffer too small for requested read length {}", read_len);
		return;
	}

	let mut tx_buf: [u8; MAX_SPI_BUF] = [0x00; MAX_SPI_BUF];
	let mut rx_buf: [u8; MAX_SPI_BUF] = [0x00; MAX_SPI_BUF];

	// first byte is the command (burst read)
	tx_buf[0] = addr | 0xC0; // Burst read from RX Buffer

	unsafe {
		furi_hal_spi_acquire(handle);
		// transfer command + requested payload bytes
		furi_hal_spi_bus_trx(
			handle,
			tx_buf.as_ptr(),
			rx_buf.as_mut_ptr(),
			read_len + 1,
			250,
		);
		furi_hal_spi_release(handle);
	}

	// rx_buf[0] holds the response to the command byte; payload starts at rx_buf[1]
	out_buf[0..read_len].copy_from_slice(&rx_buf[1..(1 + read_len)]);
}

fn spi_write_burst(handle: *const FuriHalSpiBusHandle, addr: u8, buf: &[u8]) -> () {
    let mut rx_buf: [u8; 64] = [0x00; 64];
    let mut tx_buf: [u8; 64] = [0x00; 64];

    // Create tx buffer
    tx_buf[0] = addr | 0x40; // Burst into TX Buffer
    tx_buf[1..(buf.len()+1)].copy_from_slice(buf);

    unsafe {
        furi_hal_spi_acquire(handle);
        furi_hal_spi_bus_trx(
            handle,
            tx_buf.as_ptr(),
            rx_buf.as_mut_ptr(),
            buf.len()+1,
            250,
        );
        furi_hal_spi_release(handle);
    }
}

fn spi_write_command(handle: *const FuriHalSpiBusHandle, cmd: u8) -> [u8; 1] {
    let spi_tx_buf: [u8; 1] = [cmd];
    let mut spi_rx_buf: [u8; 1] = [0x00];

    unsafe {
        furi_hal_spi_acquire(handle);
        furi_hal_spi_bus_trx(
            handle,
            spi_tx_buf.as_ptr(),
            spi_rx_buf.as_mut_ptr(),
            spi_tx_buf.len(),
            200,
        );
        furi_hal_spi_release(handle);
    }
    return spi_rx_buf;
}

fn spi_read_register(handle: *const FuriHalSpiBusHandle, addr: u8) -> u8 {
    let res = spi_write_register(handle, 0x80 | addr, 0x00);
    return res[1];
}

fn spi_write_register(handle: *const FuriHalSpiBusHandle, addr: u8, val: u8) -> [u8; 2] {
    let spi_tx_buf: [u8; 2] = [addr, val];
    let mut spi_rx_buf: [u8; 2] = [0x00, 0x00];

    unsafe {
        furi_hal_spi_acquire(handle);
        furi_hal_spi_bus_trx(
            handle,
            spi_tx_buf.as_ptr(),
            spi_rx_buf.as_mut_ptr(),
            spi_tx_buf.len(),
            200,
        );
        furi_hal_spi_release(handle);
    }
    return spi_rx_buf;
}

// Entry point
fn main(_args: Option<&CStr>) -> i32 {
    info!("Starting Radio!");
    let mut cc1101_device: CC1101Device;
    unsafe {
        cc1101_device = CC1101Device::new(&furi_hal_spi_bus_handle_subghz);
    }
    info!("Initialized Radio!");

    unsafe {
    //     let msg: [u8; 31] = *b"\x20\x00\x80\x00\x00\x00\xff\x24\x96\x4b\x64\xb6\x5b\x25\x96\xd9\x2c\x92\x49\x2d\x96\xd9\x64\x96\x5b\x24\x92\xcb\x24\xb2\x49";

    //     // Set the transmitter properties
    //     //subghz_devices_load_preset(subghz, FuriHalSubGhzPresetGFSK9_99KbAsync, null_mut());
    //     if !subghz_devices_is_frequency_valid(subghz, TRANSMIT_FREQ) {
    //         println!("Invalid frequency requested {}.", TRANSMIT_FREQ);
    //     } else {
    //         subghz_devices_set_frequency(subghz, TRANSMIT_FREQ);
    //     }
    //     subghz_devices_idle(subghz);

        // Prepare a message to send and send it.
        furi_hal_power_suppress_charge_enter();
        // spi_write_command(&furi_hal_spi_bus_handle_subghz, 0x33);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x02, 0x06);
        // spi_read_register(&furi_hal_spi_bus_handle_subghz, 0x03);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x04, 0x00);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x05, 0x00);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x07, 0x00);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x08, 0x00);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x0B, 0x06);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x10, 0xF9);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x11, 0x44);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x12, 0x05);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x13, 0x22);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x14, 0x00);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x15, 0x45);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x16, 0x07);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x17, 0x30);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x18, 0x04);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x19, 0x00);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x1A, 0xA3);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x1B, 0x03);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x1C, 0x60);
        // spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x1D, 0x91);

        cc1101_device.print_state(true);

    //     for n in 1..3 {
    //         debug!("Trying to write {} bytes", msg.len());
    //         spi_write_command(&furi_hal_spi_bus_handle_subghz, 0x3B);
    //         spi_write_burst(&furi_hal_spi_bus_handle_subghz, 0x3F, &msg);
    //         spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x06, msg.len() as u8);
    //         subghz_devices_set_tx(subghz);
    //         let mut timeout: u32 = 20;
    //         while !furi_hal_gpio_read(subghz_gdo0) {
    //             // Wait for GDO0 to be set -> sync transmitted
    //             furi_delay_tick(1);
    //             if timeout == 0 {
    //                 println!("Timeout1");
    //                 break;
    //             }
    //             timeout -= 1;
    //         }
    //         timeout = 20;
    //         while furi_hal_gpio_read(subghz_gdo0) {
    //             // Wait for GDO0 to be cleared -> end of packet
    //             furi_delay_tick(1);
    //             if timeout == 0 {
    //                 println!("Timeout2");
    //                 break;
    //             }
    //             timeout -= 1;
    //         }
    //         subghz_devices_idle(subghz);
    //         debug!("Written {} of 1.", n);
    //     }
    //     info!("Written!");


    //     // Try to perform a read
    //     // Set sync
    //     spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x02, 0x01);
    //     spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x03, 0x0F);
    //     spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x04, 0x00);
    //     spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x05, 0x00);
    //     spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x12, 0x04);
    //     // Clear RX buffer
    //     spi_write_command(&furi_hal_spi_bus_handle_subghz, 0x3A);
    //     // Set Max Packet Length
    //     spi_write_register(&furi_hal_spi_bus_handle_subghz, 0x06, 64);
    //     // Set RX mode
    //     spi_write_command(&furi_hal_spi_bus_handle_subghz, 0x34);
    //     // Check state
    //     spi_write_command(&furi_hal_spi_bus_handle_subghz, 0x3D);
    //     let mut timeout = 12000; // Wait 6 seconds
    //     while !furi_hal_gpio_read(subghz_gdo0) {
    //         // Wait for GDO0 to be set -> sync received
    //         furi_delay_tick(1);
    //         if timeout == 0 {
    //             println!("Timeout1");
    //             break;
    //         }
    //         timeout -= 1;
    //     }
    //     furi_delay_tick(10);
    //     // Set IDLE
    //     spi_write_command(&furi_hal_spi_bus_handle_subghz, 0x36);
    //     // Read number of bytes
    //     let read_size = spi_read_register(&furi_hal_spi_bus_handle_subghz, 0x3B | 0x40);
    //     if read_size > 0 {
    //         // Read into RX Buffer
    //         spi_read_burst(&furi_hal_spi_bus_handle_subghz, &mut rx_buf, 0xBF, read_size as usize);
    //         for i in 0..read_size {
    //             debug!("rx_buf[{}] = 0x{:02x}", i, rx_buf[i as usize])
    //         }
    //     }

        furi_hal_power_suppress_charge_exit();
    }

    println!("Done, Exiting!");

    0
}
