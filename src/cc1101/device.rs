use core::{
    any::Any,
    ffi::CStr,
    fmt::{Debug, Write},
};
use flipperzero::{debug, error, format, info, println};
use flipperzero_sys::{
    furi_hal_gpio_init, furi_hal_gpio_write, furi_hal_spi_acquire, furi_hal_spi_bus_trx,
    furi_hal_spi_release, gpio_rf_sw_0, subghz_devices_begin, subghz_devices_deinit,
    subghz_devices_end, subghz_devices_get_by_name, subghz_devices_get_data_gpio,
    subghz_devices_init, FuriHalSpiBusHandle, GpioModeAnalog, GpioModeInput,
    GpioModeOutputPushPull, GpioPin, GpioPullNo, GpioSpeedLow, SubGhzDeviceCC1101Int,
};
use heapless::String;
use modular_bitfield::prelude::*;

use crate::cc1101::{
    addresses::Register, constants::*, logging::REGISTER_DUMP_BUFFER_SIZE, registers::*,
};
const MAX_SPI_BUF: usize = 65;
static SUBGHZ_DEVICE_CC1101_INT_NAME: &CStr = c"cc1101_int";

/// Represents the full CC1101 register map in RAM.
pub struct CC1101Device {
    pub handle: *const FuriHalSpiBusHandle,
    pub subghz: *const flipperzero_sys::SubGhzDevice,
    pub subghz_gdo0: *const flipperzero_sys::GpioPin,
    pub gdo_config: GDOCONFIG,
    pub fifo_thr: FIFOTHR,
    pub sync: SYNC,
    pub pktlen: PKTLEN,
    pub pktctrl: PKTCTRL,
    pub addr: ADDR,
    pub channr: CHANNR,
    pub freq_synth_ctrl: FREQSYNTHCTRL,
    pub freq_ctrl: FREQCTRL,
    pub modem_config: MODEMCONFIG,
    pub deviatn: DEVIATN,
    pub mcsm: MCSM,
    pub freq_offset_comp: FREQOFFSETCOMP,
    pub bit_sync: BITSYNC,
    pub agc_ctrl: AGCCTRL,
    pub wor_evt: WOREVT,
    pub wor_ctrl: WORCTRL,
    pub front_end: FRONTEND,
    pub freq_synth_cal: FREQSYNTHCAL,
    pub rc_ctrl: RCCRTL,
    pub fs_test: FSTEST,
    pub ptest: PTEST,
    pub agc_test: AGCTEST,
    pub test_settings: TESTSETTINGS,
    pub partnum: PARTNUM,
    pub version: VERSION,
    pub freq_est: FREQEST,
    pub rssi: RSSI,
    pub marc_state: MARCSTATE,
    pub wor_time: WORTIME,
    pub pkt_status: PKTSTATUS,
    pub vco_vc_dac: VCO_VC_DAC,
    pub tx_bytes: TXBYTES,
    pub rx_bytes: RXBYTES,
    pub rc_ctrl_status: RCCTRL_STATUS,
}

impl CC1101Device {
    /// Creates a device instance where every register is zeroed in RAM.
    pub fn new(handle: *const FuriHalSpiBusHandle) -> Self {
        let subghz: *const flipperzero_sys::SubGhzDevice;
        let subghz_gdo0: *const GpioPin;

        unsafe {
            // Register device
            subghz_devices_init();
            subghz = subghz_devices_get_by_name(SUBGHZ_DEVICE_CC1101_INT_NAME.as_ptr());

            // Initialize the radio device
            subghz_devices_begin(subghz);

            // Init GPIO Pin
            subghz_gdo0 = subghz_devices_get_data_gpio(subghz);
            furi_hal_gpio_init(subghz_gdo0, GpioModeInput, GpioPullNo, GpioSpeedLow);
        }

        let mut new_self = Self {
            handle,
            subghz: subghz,
            subghz_gdo0: subghz_gdo0,
            gdo_config: GDOCONFIG::new(),
            fifo_thr: FIFOTHR::new(),
            sync: SYNC::new(),
            pktlen: PKTLEN::new(),
            pktctrl: PKTCTRL::new(),
            addr: ADDR::new(),
            channr: CHANNR::new(),
            freq_synth_ctrl: FREQSYNTHCTRL::new(),
            freq_ctrl: FREQCTRL::new(),
            modem_config: MODEMCONFIG::new(),
            deviatn: DEVIATN::new(),
            mcsm: MCSM::new(),
            freq_offset_comp: FREQOFFSETCOMP::new(),
            bit_sync: BITSYNC::new(),
            agc_ctrl: AGCCTRL::new(),
            wor_evt: WOREVT::new(),
            wor_ctrl: WORCTRL::new(),
            front_end: FRONTEND::new(),
            freq_synth_cal: FREQSYNTHCAL::new(),
            rc_ctrl: RCCRTL::new(),
            fs_test: FSTEST::new(),
            ptest: PTEST::new(),
            agc_test: AGCTEST::new(),
            test_settings: TESTSETTINGS::new(),
            partnum: PARTNUM::new(),
            version: VERSION::new(),
            freq_est: FREQEST::new(),
            rssi: RSSI::new(),
            marc_state: MARCSTATE::new(),
            wor_time: WORTIME::new(),
            pkt_status: PKTSTATUS::new(),
            vco_vc_dac: VCO_VC_DAC::new(),
            tx_bytes: TXBYTES::new(),
            rx_bytes: RXBYTES::new(),
            rc_ctrl_status: RCCTRL_STATUS::new(),
        };

        // Reset the radio
        new_self.spi_send_command(CMD::SRES);

        // Sync state
        new_self.sync_state();

        // Set GDO0 and GDO1 mode
        new_self
            .gdo_config
            .set_gdo0_cfg(GDO_PIN_CONFIG::HighImpedance);
        new_self
            .gdo_config
            .set_gdo1_cfg(GDO_PIN_CONFIG::HighImpedance);

        // Set up RF Switch to 300 - 348MHz path permanently
        // See https://github.com/flipperdevices/flipperzero-firmware/blob/c9ab2b6827fc4d646e98ad0fc15a264240b58986/targets/f7/furi_hal/furi_hal_subghz.c#L348
        // for settings
        unsafe {
            furi_hal_gpio_init(
                &gpio_rf_sw_0,
                GpioModeOutputPushPull,
                GpioPullNo,
                GpioSpeedLow,
            );
            furi_hal_gpio_write(&gpio_rf_sw_0, false);
        }
        // Write to device
        new_self
            .gdo_config
            .set_gdo2_cfg(GDO_PIN_CONFIG::HardwareZero);
        new_self.gdo_config.set_gdo2_inv(true);
        new_self.write_register(new_self.gdo_config);

        return new_self;
    }

    pub fn spi_send_command(&self, command: CMD) -> u8 {
        let spi_tx: u8 = command as u8;
        let mut spi_rx: u8 = 0x00;

        unsafe {
            furi_hal_spi_acquire(self.handle);
            furi_hal_spi_bus_trx(self.handle, &spi_tx, &mut spi_rx, 1, 200);
            furi_hal_spi_release(self.handle);
        }
        return spi_rx;
    }

    pub fn spi_read_burst(&self, addr: u8, buf: &mut [u8]) {
        if buf.is_empty() || buf.len() + 1 > MAX_SPI_BUF {
            return;
        }

        if self.handle.is_null() {
            return;
        }

        let mut tx_buf = [0u8; MAX_SPI_BUF];
        let mut rx_buf = [0u8; MAX_SPI_BUF];

        tx_buf[0] = addr | 0xC0;

        unsafe {
            furi_hal_spi_acquire(self.handle);
            furi_hal_spi_bus_trx(
                self.handle,
                tx_buf.as_ptr(),
                rx_buf.as_mut_ptr(),
                buf.len() + 1,
                250,
            );
            furi_hal_spi_release(self.handle);
        }

        buf.copy_from_slice(&rx_buf[1..(buf.len() + 1)]);
    }

    pub fn spi_write_burst(&self, addr: u8, buf: &[u8]) -> () {
        if buf.is_empty() || buf.len() + 1 > MAX_SPI_BUF {
            return;
        }

        if self.handle.is_null() {
            return;
        }

        debug!("Write register 0x{:02X}: {:?}", addr, buf);

        let mut tx_buf = [0u8; MAX_SPI_BUF];
        let mut rx_buf = [0u8; MAX_SPI_BUF];

        // Create tx buffer
        tx_buf[0] = addr | 0x40; // Burst into TX Buffer
        tx_buf[1..(buf.len() + 1)].copy_from_slice(buf);

        unsafe {
            furi_hal_spi_acquire(self.handle);
            furi_hal_spi_bus_trx(
                self.handle,
                tx_buf.as_ptr(),
                rx_buf.as_mut_ptr(),
                buf.len() + 1,
                250,
            );
            furi_hal_spi_release(self.handle);
        }
    }

    pub fn read_register<const S: usize, T: Register + From<[u8; S]>>(&self) -> T {
        let mut raw = [0u8; S];
        self.spi_read_burst(T::ADDRESS, &mut raw);
        T::from(raw)
    }

    pub fn write_register<const S: usize, T: Register + Into<[u8; S]>>(&self, register: T) {
        self.spi_write_burst(T::ADDRESS, &register.into());
    }

    pub fn sync_field<const S: usize, T, F>(&mut self, selector: F)
    where
        T: Register + From<[u8; S]>,
        F: FnOnce(&mut Self) -> &mut T,
    {
        let value = self.read_register::<S, T>();
        let slot = selector(self);
        *slot = value;
    }

    pub fn dump_register(&self, reg: &impl Debug) {
        let mut buffer = String::<REGISTER_DUMP_BUFFER_SIZE>::new();
        write!(&mut buffer, "{:?}", reg).ok();
        debug!("{}", buffer.as_str());
    }

    pub fn sync_state(&mut self) {
        self.sync_field(|dev| &mut dev.gdo_config);
        self.sync_field(|dev| &mut dev.fifo_thr);
        self.sync_field(|dev| &mut dev.sync);
        self.sync_field(|dev| &mut dev.pktlen);
        self.sync_field(|dev| &mut dev.pktctrl);
        self.sync_field(|dev| &mut dev.addr);
        self.sync_field(|dev| &mut dev.channr);
        self.sync_field(|dev| &mut dev.freq_synth_ctrl);
        self.sync_field(|dev| &mut dev.freq_ctrl);
        self.sync_field(|dev| &mut dev.modem_config);
        self.sync_field(|dev| &mut dev.deviatn);
        self.sync_field(|dev| &mut dev.mcsm);
        self.sync_field(|dev| &mut dev.freq_offset_comp);
        self.sync_field(|dev| &mut dev.bit_sync);
        self.sync_field(|dev| &mut dev.agc_ctrl);
        self.sync_field(|dev| &mut dev.wor_evt);
        self.sync_field(|dev| &mut dev.wor_ctrl);
        self.sync_field(|dev| &mut dev.front_end);
        self.sync_field(|dev| &mut dev.freq_synth_cal);
        self.sync_field(|dev| &mut dev.rc_ctrl);
        self.sync_field(|dev| &mut dev.fs_test);
        self.sync_field(|dev| &mut dev.ptest);
        self.sync_field(|dev| &mut dev.agc_test);
        self.sync_field(|dev| &mut dev.test_settings);
        self.sync_field(|dev| &mut dev.partnum);
        self.sync_field(|dev| &mut dev.version);
        self.sync_field(|dev| &mut dev.freq_est);
        self.sync_field(|dev| &mut dev.rssi);
        self.sync_field(|dev| &mut dev.marc_state);
        self.sync_field(|dev| &mut dev.wor_time);
        self.sync_field(|dev| &mut dev.pkt_status);
        self.sync_field(|dev| &mut dev.vco_vc_dac);
        self.sync_field(|dev| &mut dev.tx_bytes);
        self.sync_field(|dev| &mut dev.rx_bytes);
        self.sync_field(|dev| &mut dev.rc_ctrl_status);
    }

    /// Prints the current state of every register using the shared debug logger.
    pub fn print_state(&mut self, sync: bool) {
        if sync {
            self.sync_state();
        }

        self.dump_register(&self.gdo_config);
        self.dump_register(&self.fifo_thr);
        self.dump_register(&self.sync);
        self.dump_register(&self.pktlen);
        self.dump_register(&self.pktctrl);
        self.dump_register(&self.addr);
        self.dump_register(&self.channr);
        self.dump_register(&self.freq_synth_ctrl);
        self.dump_register(&self.freq_ctrl);
        self.dump_register(&self.modem_config);
        self.dump_register(&self.deviatn);
        self.dump_register(&self.mcsm);
        self.dump_register(&self.freq_offset_comp);
        self.dump_register(&self.bit_sync);
        self.dump_register(&self.agc_ctrl);
        self.dump_register(&self.wor_evt);
        self.dump_register(&self.wor_ctrl);
        self.dump_register(&self.front_end);
        self.dump_register(&self.freq_synth_cal);
        self.dump_register(&self.rc_ctrl);
    }

    pub fn print_test_state(&mut self, sync: bool) {
        if sync {
            self.sync_state();
        }

        self.dump_register(&self.fs_test);
        self.dump_register(&self.ptest);
        self.dump_register(&self.agc_test);
        self.dump_register(&self.test_settings);
    }

    pub fn print_status(&mut self, sync: bool) {
        if sync {
            self.sync_state();
        }

        self.dump_register(&self.partnum);
        self.dump_register(&self.version);
        self.dump_register(&self.freq_est);
        self.dump_register(&self.rssi);
        self.dump_register(&self.marc_state);
        self.dump_register(&self.wor_time);
        self.dump_register(&self.pkt_status);
        self.dump_register(&self.vco_vc_dac);
        self.dump_register(&self.tx_bytes);
        self.dump_register(&self.rx_bytes);
        self.dump_register(&self.rc_ctrl_status);
    }
}

impl Drop for CC1101Device {
    fn drop(&mut self) {
        // Idle the radio
        self.spi_send_command(CMD::SIDLE);
        unsafe {
            // Reset the GPIO pin
            self.gdo_config.set_gdo0_cfg(GDO_PIN_CONFIG::HighImpedance);
            //self.write_register(self.gdo_config);
            furi_hal_gpio_init(self.subghz_gdo0, GpioModeAnalog, GpioPullNo, GpioSpeedLow);

            // Close the subghz device
            subghz_devices_end(self.subghz);
            subghz_devices_deinit();
        }
        // Power down radio
        self.spi_send_command(CMD::SPWD);
    }
}
