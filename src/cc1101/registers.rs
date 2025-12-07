use libm::{fabsf, roundf};
use modular_bitfield::prelude::*;

use super::constants::*;

/// CC1101 Register Definitions
/// Based on CC1101 datasheet section 29

/// Combined Registers

/// 0x00-0x02: GDO Configuration
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct GDOCONFIG {
    pub gdo2_cfg: GDO_PIN_CONFIG,
    pub gdo2_inv: bool,
    #[skip]
    __: B1,

    pub gdo1_cfg: GDO_PIN_CONFIG,
    pub gdo1_inv: bool,
    pub gdo_ds: bool,

    pub gdo0_cfg: GDO_PIN_CONFIG,
    pub gdo0_inv: bool,
    pub temp_sensor_enable: bool,
}


/// 0x03: FIFOTHR – RX FIFO and TX FIFO Thresholds
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct FIFOTHR {
    pub fifo_thr: B4,
    pub close_in_rx: B2,
    pub adc_retention: bool,
    #[skip]
    __: B1,
}


/// 0x04-0x05: SYNC – Sync Word
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct SYNC {
    pub sync_hi: u8,
    pub sync_lo: u8,
}


/// 0x06: PKTLEN – Packet Length
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct PKTLEN {
    pub packet_length: u8,
}


/// 0x07-0x08: Packet Control
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct PKTCTRL {
    pub adr_chk: PKT_ADDR_CHECK,
    pub append_status: bool,
    pub crc_autoflush: bool,
    #[skip]
    __: B1,
    pub pqt: B3,

    pub length_config: PKT_LENGTH_CONFIG,
    pub crc_en: bool,
    #[skip]
    __: B1,
    pub pkt_format: PKT_FORMAT,
    pub white_data: bool,
    #[skip]
    __: B1,
}


/// 0x09: ADDR – Device Address
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct ADDR {
    pub device_addr: u8,
}


/// 0x0A: CHANNR – Channel Number
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct CHANNR {
    pub chan: u8,
}


/// 0x0B-0x0C: Frequency Synthesizer Control
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct FREQSYNTHCTRL {
    pub freq_if: B5,
    #[skip]
    __: B3,

    pub freqoff: u8,
}


/// 0x0D-0x0F: Frequency Control Word
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct FREQCTRL {
    pub freq2: u8,
    pub freq1: u8,
    pub freq0: u8,
}


/// Helper function to set frequency from MHz
impl FREQCTRL {
    pub fn set_freq_mhz(&mut self, freq_mhz: f32) {
        let freq_word = ((freq_mhz * 1_000_000.0 / F_XOSC) * (1u32 << 16) as f32) as u32;
        self.set_freq0((freq_word & 0xFF) as u8);
        self.set_freq1(((freq_word >> 8) & 0xFF) as u8);
        self.set_freq2(((freq_word >> 16) & 0x3F) as u8);
    }

    pub fn get_freq_mhz(&self) -> f32 {
        let freq_word =
            (self.freq2() as u32) << 16 | (self.freq1() as u32) << 8 | self.freq0() as u32;
        (freq_word as f32) * F_XOSC / (1u32 << 16) as f32 / 1_000_000.0
    }
}

/// 0x10-0x14: Modem Configuration
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct MODEMCONFIG {
    pub drate_e: B4,
    pub chanbw_m: B2,
    pub chanbw_e: B2,

    pub drate_m: u8,

    pub sync_mode: SYNC_MODE,
    pub manchester_en: bool,
    pub mod_format: MOD_FORMAT,
    pub dem_dcfilt_off: bool,

    pub chanspc_e: B2,
    #[skip]
    __: B2,
    pub num_preamble: NUM_PREAMBLE,
    pub fec_en: bool,

    pub chanspc_m: u8,
}


/// Helper functions for Modem Configuration
impl MODEMCONFIG {
    pub fn set_data_rate(&mut self, data_rate: f32) {
        // DR = (256 + DRATE_M) * 2^DRATE_E * f_xosc / 2^28
        // Find optimal DRATE_E (0-15) and DRATE_M (0-255)
        let mut best_e = 0u8;
        let mut best_m = 0u8;
        let mut best_error = f32::INFINITY;

        for e in 0..=15 {
            let target = data_rate * (1u64 << 28) as f32 / F_XOSC / (1u32 << e) as f32;
            if target >= 256.0 && target <= 511.0 {
                let m: u8 = roundf(target - 256.0) as u8;
                let actual_rate =
                    ((256 + m as u32) as f32 * (1u32 << e) as f32 * F_XOSC) / (1u64 << 28) as f32;
                let error = fabsf(actual_rate - data_rate);
                if error < best_error {
                    best_error = error;
                    best_e = e;
                    best_m = m;
                }
            }
        }

        self.set_drate_e(best_e);
        self.set_drate_m(best_m);
    }

    pub fn get_data_rate(&self) -> f32 {
        let drate_e = self.drate_e();
        ((256 + self.drate_m() as u32) as f32 * (1u32 << drate_e) as f32 * F_XOSC)
            / (1u64 << 28) as f32
    }

    pub fn set_channel_spacing(&mut self, spacing_khz: f32) {
        // Δf = (256 + CHANSPC_M) * 2^CHANSPC_E * f_xosc / 2^18
        // Find optimal CHANSPC_E (0-3) and CHANSPC_M (0-255)
        let mut best_e = 0u8;
        let mut best_m = 0u8;
        let mut best_error = f32::INFINITY;

        for e in 0..=3 {
            let target = (spacing_khz * 1000.0) * (1u32 << 18) as f32 / F_XOSC / (1u32 << e) as f32;
            if target >= 256.0 && target <= 511.0 {
                let m: u8 = roundf(target - 256.0) as u8;
                let actual_spacing = ((256 + m as u32) as f32 * (1u32 << e) as f32 * F_XOSC)
                    / (1u32 << 18) as f32
                    / 1000.0;
                let error = fabsf(actual_spacing - spacing_khz);
                if error < best_error {
                    best_error = error;
                    best_e = e;
                    best_m = m;
                }
            }
        }

        self.set_chanspc_e(best_e);
        self.set_chanspc_m(best_m);
    }

    pub fn get_channel_spacing(&self) -> f32 {
        let chanspc_e = self.chanspc_e();
        ((256 + self.chanspc_m() as u32) as f32 * (1u32 << chanspc_e) as f32 * F_XOSC)
            / (1u32 << 18) as f32
            / 1000.0
    }
}

/// 0x15: Deviation
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct DEVIATN {
    pub deviation_m: B3,
    #[skip]
    __: B1,
    pub deviation_e: B3,
    #[skip]
    __: B1
}
/// Helper functions for Deviation and MCS
impl DEVIATN {
    pub fn set_deviation(&mut self, deviation_hz: f32) {
        // f_dev = (8 + DEVIATION_M) * 2^DEVIATION_E * f_xosc / 2^17
        // Find optimal DEVIATION_E (0-7) and DEVIATION_M (0-7)
        let mut best_e = 0u8;
        let mut best_m = 0u8;
        let mut best_error = f32::INFINITY;

        for e in 0..=7 {
            for m in 0..=7 {
                let actual_deviation =
                    ((8 + m) as f32 * (1u32 << e) as f32 * F_XOSC) / (1u32 << 17) as f32;
                let error = fabsf(actual_deviation - deviation_hz);
                if error < best_error {
                    best_error = error;
                    best_e = e;
                    best_m = m;
                }
            }
        }

        self.set_deviation_e(best_e);
        self.set_deviation_m(best_m);
    }

    pub fn get_deviation(&self) -> f32 {
        let deviation_e = self.deviation_e();
        let deviation_m = self.deviation_m();
        ((8 + deviation_m) as f32 * (1u32 << deviation_e) as f32 * F_XOSC) / (1u32 << 17) as f32
    }
}

/// 0x16-0x18: Main Radio Control State Machine
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct MCSM {
    pub rx_time: B3,
    pub rx_time_qual: bool,
    pub rx_time_rssi: bool,
    #[skip]
    __: B3,

    pub txoff_mode: TXOFF_MODE,
    pub rxoff_mode: RXOFF_MODE,
    pub cca_mode: CCA_MODE,
    #[skip]
    __: B2,

    pub xosc_force_on: bool,
    pub pin_ctrl_en: bool,
    pub po_timeout: B2,
    pub fs_autocal: FS_AUTOCAL,
    #[skip]
    __: B2,
}

/// 0x19: Frequency Offset Compensation
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct FREQOFFSETCOMP {
    pub foc_limit: FOC_LIMIT,
    pub foc_post_k: FOC_POST_K,
    pub foc_pre_k: FOC_PRE_K,
    pub foc_bs_cs_gate: bool,
    #[skip]
    __: B2,
}

/// 0x1A: Bit Synchronization
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct BITSYNC {
    pub bs_limit: BS_LIMIT,
    pub bs_post_kp: BS_POST_KP,
    pub bs_post_ki: BS_POST_KI,
    pub bs_pre_kp: BS_PRE_KP,
    pub bs_pre_ki: BS_PRE_KI,
}


/// 0x1B-0x1D: AGC Control
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct AGCCTRL {
    pub magn_target: MAGN_TARGET,
    pub max_lna_gain: MAX_LNA_GAIN,
    pub max_dvga_gain: MAX_DVGA_GAIN,

    pub carrier_sense_abs_thr: CARRIER_SENSE_ABS_THR,
    pub carrier_sense_rel_thr: CARRIER_SENSE_REL_THR,
    pub agc_lna_priority: bool,
    #[skip]
    __: B1,

    pub filter_length: FILTER_LENGTH,
    pub agc_freeze: AGC_FREEZE,
    pub wait_time: WAIT_TIME,
    pub hyst_level: HYST_LEVEL,
}


/// 0x1E-0x1F: Event0 Timeout
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct WOREVT {
    pub event0: u16,
}


/// 0x20: WORCTRL – Wake On Radio Control
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct WORCTRL {
    pub wor_res: B2,
    #[skip]
    __: B1,
    pub rc_cal: bool,
    pub event1: EVENT1,
    pub rc_pd: bool,
}


/// 0x21-0x22: Front End Configuration
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct FRONTEND {
    pub mix_current: B2,
    pub lodiv_buf_current_rx: B2,
    pub lna2mix_current: B2,
    pub lna_current: B2,

    pub pa_power: B3,
    #[skip]
    __: B1,
    pub lodiv_buf_current_tx: B2,
    #[skip]
    __: B2,
}


/// 0x23-0x26: Frequency Synthesizer Calibration
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct FREQSYNTHCAL {
    pub fscal3: B4,
    pub chp_curr_cal_en: B2,
    pub fscal3_high: B2,

    pub fscal2: B5,
    pub vco_core_h_en: bool,
    #[skip]
    __: B2,

    pub fscal1: B6,
    #[skip]
    __: B2,

    pub fscal0: B7,
    #[skip]
    __: B1,
}


/// 0x27-0x28: RC Oscillator Configuration
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct RCCRTL {
    pub rcctrl1: B7,
    #[skip]
    __: B1,

    pub rcctrl0: B7,
    #[skip]
    __: B1,
}


/// 0x29: FSTEST – Frequency Synthesizer Calibration Control
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct FSTEST {
    pub fstest: u8,
}


/// 0x2A: PTEST – Production Test
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct PTEST {
    pub ptest: u8,
}


/// 0x2B: AGCTEST – AGC Test
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct AGCTEST {
    pub agctest: u8,
}


/// 0x2C-0x2E: Test Settings
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct TESTSETTINGS {
    pub test2: u8,
    pub test1: u8,
    pub test0: u8,
}


/// Status Registers

/// 0x30: PARTNUM – Chip ID
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct PARTNUM {
    pub partnum: u8,
}


/// 0x31: VERSION – Chip ID
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct VERSION {
    pub version: u8,
}


/// 0x32: FREQEST – Frequency Offset Estimate from Demodulator
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct FREQEST {
    pub freqoff_est: u8,
}


/// 0x33: LQI – Demodulator Estimate for Link Quality
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct LQI {
    pub lqi_est: B7,
    pub crc_ok: bool,
}

/// 0x34: RSSI – Received Signal Strength Indication
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct RSSI {
    pub rssi: u8,
}


/// 0x35: MARCSTATE – Main Radio Control State Machine State
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct MARCSTATE {
    pub marcstate: u8,
}


/// 0x36-0x37: WORTIME – High/Low Byte of WOR Time
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct WORTIME {
    pub wortime: u16,
}


/// 0x38: PKTSTATUS – Current GDOx Status and Packet Status
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct PKTSTATUS {
    pub pktstatus: u8,
}


/// 0x39: VCO_VC_DAC – Current Setting from PLL Calibration Module
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct VCO_VC_DAC {
    pub vco_vc_dac: u8,
}


/// 0x3A: TXBYTES – Underflow and Number of Bytes
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct TXBYTES {
    pub num_txbytes: B7,
    pub txfifo_underflow: bool,
}


/// 0x3B: RXBYTES – Overflow and Number of Bytes
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct RXBYTES {
    pub num_rxbytes: B7,
    pub rxfifo_overflow: bool,
}


/// 0x3C-0x3D: RCCTRL_STATUS – Last RC Oscillator Calibration Result
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct RCCTRL_STATUS {
    pub rcctrl0_status: u8,
    pub rcctrl1_status: u8,
}

