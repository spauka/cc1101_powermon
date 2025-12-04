use modular_bitfield::prelude::*;

pub const F_XOSC: f32 = 26_000_000.0;

// Enums for option fields
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum CMD {
    // Reset chip
    SRES = 0x30,
    // Enable and calibrate frequency synthesizer
    SFSTXON = 0x31,
    // Turn off crystal oscillator
    SXOFF = 0x32,
    // Calibrate frequency synthesizer and turn it off
    SCAL = 0x33,
    // Enable RX
    SRX = 0x34,
    // If in IDLE state: Enable TX. If in RX state and CCA is enabled:
    // Only go to TX if channel is clear.
    STX = 0x35,
    // Exit RX / TX, turn off frequency synthesizer and exit Wake-On-Radio
    // mode if applicable
    SIDLE = 0x36,
    // Start automatic RX polling sequence
    SWOR = 0x38,
    // Enter power down mode when CSn goes high
    SPWD = 0x39,
    // Flush the RX FIFO buffer
    SFRX = 0x3A,
    // Flush the TX FIFO buffer
    SFTX = 0x3B,
    // Reset real time clock to Event1 value
    SWORRST = 0x3C,
    // No operation
    SNOP = 0x3D,
}


#[derive(Debug, Clone, Copy, Specifier)]
pub enum PKT_FORMAT {
    NORMAL,
    SYNC_SERIAL,
    RANDOM_TX,
    ASYNC_SERIAL,
}


#[derive(Debug, Clone, Copy, Specifier)]
pub enum MAX_DVGA_GAIN {
    ALL,
    HIGHEST_NOT,
    TWO_HIGHEST_NOT,
    THREE_HIGHEST_NOT,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum MAX_LNA_GAIN {
    MAX,
    M2_6,
    M6_1,
    M7_4,
    M9_2,
    M11_5,
    M14_6,
    M17_1,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum MAGN_TARGET {
    D24,
    D27,
    D30,
    D33,
    D36,
    D38,
    D40,
    D42,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum CARRIER_SENSE_REL_THR {
    DISABLED,
    D6,
    D10,
    D14,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum CARRIER_SENSE_ABS_THR {
    P0DB,
    P1DB,
    P2DB,
    P3DB,
    P4DB,
    P5DB,
    P6DB,
    P7DB,
    DISABLED,
    N7DB,
    N6DB,
    N5DB,
    N4DB,
    N3DB,
    N2DB,
    N1DB
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum HYST_LEVEL {
    NO,
    LOW,
    MEDIUM,
    LARGE,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum WAIT_TIME {
    S8,
    S16,
    S24,
    S32,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum AGC_FREEZE {
    NORMAL,
    SYNC_WORD,
    ANALOG,
    BOTH,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum FILTER_LENGTH {
    S8,
    S16,
    S32,
    S64,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum MOD_FORMAT {
    FSK2,
    GFSK,
    RESERVED2,
    ASK_OOK,
    FSK4,
    RESERVED5,
    RESERVED6,
    MSK,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum SYNC_MODE {
    NO_PREAMBLE_SYNC,
    S15_16,
    S16_16,
    S30_32,
    NO_PREAMBLE_SYNC_CS,
    S15_16_CS,
    S16_16_CS,
    S30_32_CS,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum NUM_PREAMBLE {
    P2,
    P3,
    P4,
    P6,
    P8,
    P12,
    P16,
    P24,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum TXOFF_MODE {
    IDLE,
    FSTXON,
    TX,
    RX,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum RXOFF_MODE {
    IDLE,
    FSTXON,
    TX,
    STAY_RX,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum CCA_MODE {
    ALWAYS,
    RSSI_BELOW,
    NOT_RX,
    BOTH,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum FS_AUTOCAL {
    NEVER,
    IDLE_TO_RX_TX,
    RX_TX_TO_IDLE,
    EVERY_4,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum FOC_LIMIT {
    PM0,
    PM_BW_8,
    PM_BW_4,
    PM_BW_2,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum FOC_PRE_K {
    K,
    K2,
    K3,
    K4,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum FOC_POST_K {
    SAME,
    K_2,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum BS_PRE_KI {
    KI,
    KI2,
    KI3,
    KI4,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum BS_PRE_KP {
    KP,
    KP2,
    KP3,
    KP4,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum BS_POST_KI {
    SAME,
    KI_2,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum BS_POST_KP {
    SAME,
    KP,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum BS_LIMIT {
    PM0,
    PM3_125,
    PM6_25,
    PM12_5,
}

#[derive(Debug, Clone, Copy, Specifier)]
pub enum EVENT1 {
    P4,
    P6,
    P8,
    P12,
    P16,
    P24,
    P32,
    P48,
}

#[derive(Debug, Clone, Copy, Specifier)]
#[bits = 6]
pub enum GDO_PIN_CONFIG {
    /// Asserts when RX FIFO is filled at/above RX threshold or when the end of packet occurs.
    RxFifoAboveThreshold = 0x00,
    /// Asserts when RX FIFO is filled at/above threshold and de-asserts when empty.
    RxFifoThresholdOrEndOfPacket = 0x01,
    /// Asserts when TX FIFO is filled at/above TX threshold.
    TxFifoAboveThreshold = 0x02,
    /// Asserts when TX FIFO is full.
    TxFifoFull = 0x03,
    /// Asserts on RX FIFO overflow.
    RxFifoOverflow = 0x04,
    /// Asserts on TX FIFO underflow.
    TxFifoUnderflow = 0x05,
    /// Asserts when a sync word has been sent/received.
    SyncWordSymbol = 0x06,
    /// Asserts when a packet with CRC OK is received.
    PacketWithCrc = 0x07,
    /// Preamble quality indicator reached the programmed PQT value.
    PqiReached = 0x08,
    /// Clear channel assessment comparator.
    ClearChannelAssessment = 0x09,
    /// Lock detector output (PLL lock) when viewed as an interrupt source.
    LockDetectorOutput = 0x0A,
    /// Serial clock (synchronous serial mode).
    SerialClock = 0x0B,
    /// Serial synchronous data output.
    SerialSyncData = 0x0C,
    /// Serial data output (asynchronous serial mode).
    SerialDataOutput = 0x0D,
    /// Carrier sense active when RSSI is above threshold.
    CarrierSense = 0x0E,
    /// CRC OK indicator from the last received packet.
    CrcOk = 0x0F,
    /// Reserved – used for test.
    ReservedTest0x10 = 0x10,
    /// Reserved – used for test.
    ReservedTest0x11 = 0x11,
    /// Reserved – used for test.
    ReservedTest0x12 = 0x12,
    /// Reserved – used for test.
    ReservedTest0x13 = 0x13,
    /// Reserved – used for test.
    ReservedTest0x14 = 0x14,
    /// Reserved – used for test.
    ReservedTest0x15 = 0x15,
    /// Alternate serial RX output, RX_HARD_DATA[1].
    RxHardData1 = 0x16,
    /// Alternate serial RX output, RX_HARD_DATA[0].
    RxHardData0 = 0x17,
    /// Reserved – used for test.
    ReservedTest0x18 = 0x18,
    /// Reserved – used for test.
    ReservedTest0x19 = 0x19,
    /// Reserved – used for test.
    ReservedTest0x1A = 0x1A,
    /// PA power-down control (same level in SLEEP/TX states).
    PaPowerDown = 0x1B,
    /// LNA power-down control (same level in SLEEP/RX states).
    LnaPowerDown = 0x1C,
    /// RX_SYMBOL_TICK alternative serial RX output.
    RxSymbolTick = 0x1D,
    /// Reserved – used for test.
    ReservedTest0x1E = 0x1E,
    /// Reserved – used for test.
    ReservedTest0x1F = 0x1F,
    /// Reserved – used for test.
    ReservedTest0x20 = 0x20,
    /// Reserved – used for test.
    ReservedTest0x21 = 0x21,
    /// Reserved – used for test.
    ReservedTest0x22 = 0x22,
    /// Reserved – used for test.
    ReservedTest0x23 = 0x23,
    /// Wake-on-radio event 0.
    WorEvent0 = 0x24,
    /// Wake-on-radio event 1.
    WorEvent1 = 0x25,
    /// 256-Hz clock output.
    Clock256Hz = 0x26,
    /// 32-kHz clock output.
    Clock32KHz = 0x27,
    /// Reserved – used for test.
    ReservedTest0x28 = 0x28,
    /// CHIP_RDYn indicator.
    ChipReady = 0x29,
    /// Reserved – used for test.
    ReservedTest0x2A = 0x2A,
    /// XOSC stable indicator.
    XoscStable = 0x2B,
    /// Reserved – used for test.
    ReservedTest0x2C = 0x2C,
    /// Reserved – used for test.
    ReservedTest0x2D = 0x2D,
    /// High impedance (3-state).
    HighImpedance = 0x2E,
    /// Hardware 0 (can become HW1 with GDOx_INV=1).
    HardwareZero = 0x2F,
    /// Crystal oscillator divided by 1 output.
    ClockXosc1 = 0x30,
    /// Crystal oscillator divided by 1.5 output.
    ClockXosc1_5 = 0x31,
    /// Crystal oscillator divided by 2 output.
    ClockXosc2 = 0x32,
    /// Crystal oscillator divided by 3 output.
    ClockXosc3 = 0x33,
    /// Crystal oscillator divided by 4 output.
    ClockXosc4 = 0x34,
    /// Crystal oscillator divided by 6 output.
    ClockXosc6 = 0x35,
    /// Crystal oscillator divided by 8 output.
    ClockXosc8 = 0x36,
    /// Crystal oscillator divided by 12 output.
    ClockXosc12 = 0x37,
    /// Crystal oscillator divided by 16 output.
    ClockXosc16 = 0x38,
    /// Crystal oscillator divided by 24 output.
    ClockXosc24 = 0x39,
    /// Crystal oscillator divided by 32 output.
    ClockXosc32 = 0x3A,
    /// Crystal oscillator divided by 48 output.
    ClockXosc48 = 0x3B,
    /// Crystal oscillator divided by 64 output.
    ClockXosc64 = 0x3C,
    /// Crystal oscillator divided by 96 output.
    ClockXosc96 = 0x3D,
    /// Crystal oscillator divided by 128 output.
    ClockXosc128 = 0x3E,
    /// Crystal oscillator divided by 192 output.
    ClockXosc192 = 0x3F,
}
