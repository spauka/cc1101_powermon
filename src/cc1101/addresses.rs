use super::registers::*;

pub trait Register: Sized {
    const ADDRESS: u8;
    const SIZE_BYTES: usize;

    fn size_bytes() -> usize {
        Self::SIZE_BYTES
    }

    fn address() -> u8 {
        Self::ADDRESS
    }
}

impl Register for GDOCONFIG {
    const ADDRESS: u8 = 0x00;
    const SIZE_BYTES: usize = 3;
}

impl Register for FIFOTHR {
    const ADDRESS: u8 = 0x03;
    const SIZE_BYTES: usize = 1;
}

impl Register for SYNC {
    const ADDRESS: u8 = 0x04;
    const SIZE_BYTES: usize = 2;
}

impl Register for PKTLEN {
    const ADDRESS: u8 = 0x06;
    const SIZE_BYTES: usize = 1;
}

impl Register for PKTCTRL {
    const ADDRESS: u8 = 0x07;
    const SIZE_BYTES: usize = 2;
}

impl Register for ADDR {
    const ADDRESS: u8 = 0x09;
    const SIZE_BYTES: usize = 1;
}

impl Register for CHANNR {
    const ADDRESS: u8 = 0x0A;
    const SIZE_BYTES: usize = 1;
}

impl Register for FREQSYNTHCTRL {
    const ADDRESS: u8 = 0x0B;
    const SIZE_BYTES: usize = 2;
}

impl Register for FREQCTRL {
    const ADDRESS: u8 = 0x0D;
    const SIZE_BYTES: usize = 3;
}

impl Register for MODEMCONFIG {
    const ADDRESS: u8 = 0x10;
    const SIZE_BYTES: usize = 5;
}

impl Register for DEVIATN {
    const ADDRESS: u8 = 0x15;
    const SIZE_BYTES: usize = 1;
}

impl Register for MCSM {
    const ADDRESS: u8 = 0x16;
    const SIZE_BYTES: usize = 3;
}

impl Register for FREQOFFSETCOMP {
    const ADDRESS: u8 = 0x19;
    const SIZE_BYTES: usize = 1;
}

impl Register for BITSYNC {
    const ADDRESS: u8 = 0x1A;
    const SIZE_BYTES: usize = 1;
}

impl Register for AGCCTRL {
    const ADDRESS: u8 = 0x1B;
    const SIZE_BYTES: usize = 3;
}

impl Register for WOREVT {
    const ADDRESS: u8 = 0x1E;
    const SIZE_BYTES: usize = 2;
}

impl Register for WORCTRL {
    const ADDRESS: u8 = 0x20;
    const SIZE_BYTES: usize = 1;
}

impl Register for FRONTEND {
    const ADDRESS: u8 = 0x21;
    const SIZE_BYTES: usize = 2;
}

impl Register for FREQSYNTHCAL {
    const ADDRESS: u8 = 0x23;
    const SIZE_BYTES: usize = 4;
}

impl Register for RCCRTL {
    const ADDRESS: u8 = 0x27;
    const SIZE_BYTES: usize = 2;
}

impl Register for FSTEST {
    const ADDRESS: u8 = 0x29;
    const SIZE_BYTES: usize = 1;
}

impl Register for PTEST {
    const ADDRESS: u8 = 0x2A;
    const SIZE_BYTES: usize = 1;
}

impl Register for AGCTEST {
    const ADDRESS: u8 = 0x2B;
    const SIZE_BYTES: usize = 1;
}

impl Register for TESTSETTINGS {
    const ADDRESS: u8 = 0x2C;
    const SIZE_BYTES: usize = 3;
}

impl Register for PARTNUM {
    const ADDRESS: u8 = 0x30;
    const SIZE_BYTES: usize = 1;
}

impl Register for VERSION {
    const ADDRESS: u8 = 0x31;
    const SIZE_BYTES: usize = 1;
}

impl Register for FREQEST {
    const ADDRESS: u8 = 0x32;
    const SIZE_BYTES: usize = 1;
}

impl Register for RSSI {
    const ADDRESS: u8 = 0x34;
    const SIZE_BYTES: usize = 1;
}

impl Register for MARCSTATE {
    const ADDRESS: u8 = 0x35;
    const SIZE_BYTES: usize = 1;
}

impl Register for WORTIME {
    const ADDRESS: u8 = 0x36;
    const SIZE_BYTES: usize = 2;
}

impl Register for PKTSTATUS {
    const ADDRESS: u8 = 0x38;
    const SIZE_BYTES: usize = 1;
}

impl Register for VCO_VC_DAC {
    const ADDRESS: u8 = 0x39;
    const SIZE_BYTES: usize = 1;
}

impl Register for TXBYTES {
    const ADDRESS: u8 = 0x3A;
    const SIZE_BYTES: usize = 1;
}

impl Register for RXBYTES {
    const ADDRESS: u8 = 0x3B;
    const SIZE_BYTES: usize = 1;
}

impl Register for RCCTRL_STATUS {
    const ADDRESS: u8 = 0x3C;
    const SIZE_BYTES: usize = 2;
}
