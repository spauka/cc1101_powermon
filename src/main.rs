//! Template project for Flipper Zero.
//! This app prints "Hello, Rust!" to the console then exits.

#![no_main]
#![no_std]

// Required for panic handler
extern crate flipperzero_rt;

use core::{ffi::CStr};

use flipperzero::{debug, error, info, println};
use flipperzero_rt::{entry, manifest};
use flipperzero_sys::{
    furi_delay_tick, furi_get_tick, furi_hal_gpio_read, furi_hal_spi_bus_handle_subghz,
};

use crate::cc1101::{
    CC1101Device, BS_LIMIT, BS_PRE_KI, BS_PRE_KP, CARRIER_SENSE_ABS_THR, CARRIER_SENSE_REL_THR,
    CMD, FOC_LIMIT, FOC_PRE_K, FREQSYNTHCAL, GDO_PIN_CONFIG, MAGN_TARGET, MOD_FORMAT, NUM_PREAMBLE,
    PKTCTRL, PKT_ADDR_CHECK, PKT_FORMAT, PKT_LENGTH_CONFIG, SYNC_MODE,
};

mod cc1101;
mod decode;
mod debug;

static BAUD_RATE: f32 = 16150.0;
static FSK_DEV: f32 = 84_000.0;
static TRANSMIT_FREQ: u32 = 433535649;
static TRANSMIT_FREQ_MHZ: f32 = (TRANSMIT_FREQ as f32) / 1_000_000f32;

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

// Entry point
fn main(_args: Option<&CStr>) -> i32 {
    info!("Starting Radio!");
    let mut cc1101_device: CC1101Device;
    let mut rx_buf = [0u8; 128];
    unsafe {
        cc1101_device = CC1101Device::new(&furi_hal_spi_bus_handle_subghz);
    }
    info!("Initialized Radio!");

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

    cc1101_device.agc_ctrl.set_magn_target(MAGN_TARGET::D33);
    cc1101_device
        .agc_ctrl
        .set_carrier_sense_rel_thr(CARRIER_SENSE_REL_THR::D14);
    cc1101_device
        .agc_ctrl
        .set_carrier_sense_abs_thr(CARRIER_SENSE_ABS_THR::P6DB);
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

    cc1101_device.print_state(true);

    unsafe {
        for _i in 0..10 {
            cc1101_device.spi_send_command(CMD::SCAL);
            furi_delay_tick(10);
            cc1101_device.spi_send_command(CMD::SFRX);
            cc1101_device.spi_send_command(CMD::SRX);

            let mut timeout = 6000; // Wait 6 seconds
            while !furi_hal_gpio_read(cc1101_device.subghz_gdo0) {
                // Wait for GDO0 to be set -> carrier sense
                furi_delay_tick(1);
                if timeout == 0 {
                    info!("Timeout1");
                    break;
                }
                timeout -= 1;
            }
            furi_delay_tick(10);

            let start_time = furi_get_tick();
            if timeout != 0 {
                let mut read_bytes: usize = 0;
                let elapsed_ms = furi_get_tick() - start_time;
                while elapsed_ms < 500
                    && read_bytes < 127
                    && furi_hal_gpio_read(cc1101_device.subghz_gdo0)
                {
                    cc1101_device.sync_field(|dev| &mut dev.rx_bytes);
                    let mut rx_bytes1: usize = cc1101_device.rx_bytes.num_rxbytes() as usize;
                    cc1101_device.sync_field(|dev| &mut dev.rx_bytes);
                    let rx_bytes2: usize = cc1101_device.rx_bytes.num_rxbytes() as usize;

                    if rx_bytes1 == rx_bytes2 && rx_bytes1 > 0 && rx_bytes1 < 64 {
                        if read_bytes + rx_bytes1 >= 127 {
                            rx_bytes1 = 127 - read_bytes;
                        }
                        cc1101_device.spi_read_burst(
                            0xC0 | 0x3F,
                            &mut rx_buf[read_bytes..read_bytes + rx_bytes1],
                        );
                        read_bytes += rx_bytes1;
                    } else if rx_bytes1 > 64 {
                        error!("RX Buffer Overflow")
                    } else {
                        continue;
                    }
                }
                cc1101_device.spi_send_command(CMD::SIDLE);
                if read_bytes > 0 {
                    let res = decode::decode_power(&rx_buf, read_bytes);
                    match res {
                        Ok(decode::DecodeResult {
                            power_kw: power,
                            packet: _,
                            quality_metric: _,
                        }) => println!("Power: {} W", (power * 1000.0) as u32),
                        Err(decode::DecodeError::NotEnoughData) => println!("Incomplete packet"),
                        Err(decode::DecodeError::PreambleNotFound) => {
                            println!("Preamble not found")
                        }
                        Err(decode::DecodeError::SyncNotFound) => println!("Sync not found"),
                        Err(decode::DecodeError::InsufficientSymbols) => {
                            println!("Incomplete packet decoded")
                        }
                        Err(decode::DecodeError::ChecksumMismatch { expected, actual }) => {
                            println!("Checksum mismatch ({} != {})", expected, actual)
                        }
                    }
                }
            }
        }
    }
    println!("Done, Exiting!");

    0
}
