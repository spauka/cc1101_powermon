Low-Power Sub-1 GHz RF Transceiver: 0
 Applications: 2
 Product Description: 12
  Key Features: 24
  RF Performance: 26
  Analog Features: 37
  Digital Features: 44
  Low-Power Features: 56
  General: 63
  Improved Range using CC1190: 72
  Reduced Battery Current using TPS62730: 82
 Abbreviations: 97
 Table of Contents: 170
 1 Absolute Maximum Ratings: 365
 2 Operating Conditions: 387
 3 General Characteristics: 398
 4 Electrical Specifications: 412
  4.1 Current Consumption: 414
  4.2 RF Receive Section: 487
   315 MHz: 498
   433 MHz: 507
   868/915 MHz: 520
 4.3 RF Transmit Section: 591
  4.4 Crystal Oscillator: 645
  4.5 Low Power RC Oscillator: 659
  4.6 Frequency Synthesizer Characteristics: 673
  4.7 Analog Temperature Sensor: 696
  4.8 DC Characteristics: 712
  4.9 Power-On Reset: 727
 5 Pin Configuration: 738
 6 Circuit Description: 775
 7 Application Circuit: 793
  7.1 Bias Resistor: 801
  7.2 Balun and RF Matching: 807
  7.3 Crystal: 817
  7.4 Reference Signal: 833
  7.5 Additional Filtering: 837
  7.6 Power Supply Decoupling: 845
  7.7 Antenna Considerations: 849
  7.8 PCB Layout Recommendations: 906
 8 Configuration Overview: 934
 9 Configuration Software: 962
 10 4-wire Serial Configuration and Data Interface: 972
  10.1 Chip Status Byte: 1006
  10.2 Register Access: 1024
  10.3 SPI Read: 1032
  10.4 Command Strobes: 1036
  10.5 FIFO Access: 1054
  10.6 PATABLE Access: 1074
 11 Microcontroller Interface and Pin Configuration: 1092
  11.1 Configuration Interface: 1100
  11.2 General Control and Status Pins: 1104
  11.3 Optional Radio Control Feature: 1114
 12 Data Rate Programming: 1138
  13 Receiver Channel Filter Bandwidth: 1170
 14 Demodulator, Symbol Synchronizer, and Data Decision: 1192
  14.1 Frequency Offset Compensation: 1196
  14.2 Bit Synchronization: 1212
 15 Packet Handling Hardware Support: 1222
  15.1 Data Whitening: 1265
  15.2 Packet Format: 1277
   15.2.1 Arbitrary Length Field Configuration: 1306
   15.2.2 Packet Length > 255: 1310
  15.3 Packet Filtering in Receive Mode: 1328
   15.3.1 Address Filtering: 1332
   15.3.2 Maximum Length Filtering: 1338
   15.3.3 CRC Filtering: 1342
  15.4 Packet Handling in Transmit Mode: 1348
  15.5 Packet Handling in Receive Mode: 1360
  15.6 Packet Handling in Firmware: 1374
   a) Interrupt Driven Solution: 1378
   b) SPI Polling: 1382
  16 Modulation Formats: 1394
   16.1 Frequency Shift Keying: 1404
   16.2 Minimum Shift Keying: 1433
   16.3 Amplitude Modulation: 1443
 17 Received Signal Qualifiers and Link Quality Information: 1453
  17.1 Sync Word Qualifier: 1464
  17.2 Preamble Quality Threshold (PQT): 1481
  17.3 RSSI: 1492
  17.4 Carrier Sense (CS): 1531
   17.4.1 CS Absolute Threshold: 1544
   17.4.2 CS Relative Threshold: 1586
  17.5 Clear Channel Assessment (CCA): 1590
  17.6 Link Quality Indicator (LQI): 1605
 18 Forward Error Correction with Interleaving: 1609
  18.1 Forward Error Correction (FEC): 1611
  18.2 Interleaving: 1625
 19 Radio Control: 1639
  19.1 Power-On Start-Up Sequence: 1649
   19.1.1 Automatic POR: 1653
   19.1.2 Manual Reset: 1663
  19.2 Crystal Control: 1680
  19.3 Voltage Regulator Control: 1692
  19.4 Active Modes (RX and TX): 1698
  19.5 Wake On Radio (WOR): 1729
   19.5.1 RC Oscillator and Timing: 1759
  19.6 Timing: 1765
   19.6.1 Overall State Transition Times: 1767
   19.6.2 Frequency Synthesizer Calibration Time: 1798
  19.7 RX Termination Timer: 1813
 20 Data FIFO: 1828
 21 Frequency Programming: 1886
 22 VCO: 1906
  22.1 VCO and PLL Self-Calibration: 1910
 23 Voltage Regulators: 1922
 24 Output Power Programming: 1934
 25 Shaping and PA Ramping: 1991
 26 General Purpose / Test Output Control Pins: 2003
 27 Asynchronous and Synchronous Serial Operation: 2078
  27.1 Asynchronous Serial Operation: 2082
  27.2 Synchronous Serial Operation: 2096
 28 System Considerations and Guidelines: 2112
  28.1 SRD Regulations: 2114
  28.2 Frequency Hopping and Multi-Channel Systems: 2122
  28.3 Wideband Modulation when not Using Spread Spectrum: 2140
  28.4 Wireless MBUS: 2146
  28.5 Data Burst Transmissions: 2152
  28.6 Continuous Transmissions: 2158
  28.7 Battery Operated Systems: 2162
  28.8 Increasing Range: 2166
 29 Configuration Registers: 2176
  29.1 Configuration Register Details – Registers with preserved values in SLEEP state: 2282
   0x00: IOCFG2 – GDO2 Output Pin Configuration: 2284
   0x01: IOCFG1 – GDO1 Output Pin Configuration: 2292
   0x02: IOCFG0 – GDO0 Output Pin Configuration: 2300
   0x03: FIFOTHR – RX FIFO and TX FIFO Thresholds: 2308
   0x04: SYNC1 – Sync Word, High Byte: 2388
   0x05: SYNC0 – Sync Word, Low Byte: 2394
   0x06: PKTLEN – Packet Length: 2400
   0x07: PKTCTRL1 – Packet Automation Control: 2406
   0x08: PKTCTRL0 – Packet Automation Control: 2450
   0x09: ADDR – Device Address: 2507
   0x0A: CHANNR – Channel Number: 2513
   0x0B: FSCTRL1 - Frequency Synthesizer Control: 2519
   0x0C: FSCTRL0 - Frequency Synthesizer Control: 2527
   0x0D: FREQ2 - Frequency Control Word, High Byte: 2533
   0x0E: FREQ1 - Frequency Control Word, Middle Byte: 2540
   0x0F: FREQ0 - Frequency Control Word, Low Byte: 2546
   0x10: MDMCFG4 - Modem Configuration: 2552
   0x11: MDMCFG3 - Modem Configuration: 2560
   0x12: MDMCFG2 – Modem Configuration: 2567
   0x13: MDMCFG1- Modem Configuration: 2646
   0x14: MDMCFG0- Modem Configuration: 2690
   0x15: DEVIATN - Modem Deviation Setting: 2696
   0x16: MCSM2 – Main Radio Control State Machine Configuration: 2745
   0x17: MCSM1– Main Radio Control State Machine Configuration: 2786
   0x18: MCSM0– Main Radio Control State Machine Configuration: 2844
   0x19: FOCCFG – Frequency Offset Compensation Configuration: 2914
   0x1A: BSCFG – Bit Synchronization Configuration: 2976
   0x1B: AGCCTRL2 – AGC Control: 3058
   0x1C: AGCCTRL1 – AGC Control: 3137
   0x1D: AGCCTRL0 - AGC Control: 3197
   0x1E: WOREVT1 – High Byte Event0 Timeout: 3279
   0x1F: WOREVT0 –Low Byte Event0 Timeout: 3285
   0x20: WORCTRL – Wake On Radio Control: 3291
   0x21: FREND1 – Front End RX Configuration: 3355
   0x22: FREND0 – Front End TX Configuration: 3364
   0x23: FSCAL3 – Frequency Synthesizer Calibration: 3373
   0x24: FSCAL2 – Frequency Synthesizer Calibration: 3381
   0x25: FSCAL1 – Frequency Synthesizer Calibration: 3389
   0x26: FSCAL0 – Frequency Synthesizer Calibration: 3396
   0x27: RCCTRL1 – RC Oscillator Configuration: 3403
   0x28: RCCTRL0 – RC Oscillator Configuration: 3410
  29.2 Configuration Register Details – Registers that Loose Programming in SLEEP State: 3419
   0x29: FSTEST – Frequency Synthesizer Calibration Control: 3421
   0x2A: PTEST – Production Test: 3427
   0x2B: AGCTEST – AGC Test: 3433
   0x2C: TEST2 – Various Test Settings: 3439
   0x2D: TEST1 – Various Test Settings: 3445
   0x2E: TEST0 – Various Test Settings: 3453
  29.3 Status Register Details: 3461
   0x30 (0xF0): PARTNUM – Chip ID: 3463
   0x31 (0xF1): VERSION – Chip ID: 3469
   0x32 (0xF2): FREQEST – Frequency Offset Estimate from Demodulator: 3475
   0x33 (0xF3): LQI – Demodulator Estimate for Link Quality: 3481
   0x34 (0xF4): RSSI – Received Signal Strength Indication: 3488
   0x35 (0xF5): MARCSTATE – Main Radio Control State Machine State: 3494
   0x36 (0xF6): WORTIME1 – High Byte of WOR Time: 3582
   0x37 (0xF7): WORTIME0 – Low Byte of WOR Time: 3588
   0x38 (0xF8): PKTSTATUS – Current GDOx Status and Packet Status: 3594
   0x39 (0xF9): VCO\_VC\_DAC – Current Setting from PLL Calibration Module: 3607
   0x3A (0xFA): TXBYTES – Underflow and Number of Bytes: 3613
   0x3B (0xFB): RXBYTES – Overflow and Number of Bytes: 3620
   0x3C (0xFC): RCCTRL1\_STATUS – Last RC Oscillator Calibration Result: 3627
   0x3D (0xFD): RCCTRL0\_STATUS – Last RC Oscillator Calibration Result: 3634
 30 Soldering Information: 3641
 31 Development Kit Ordering Information: 3645
 32 References: 3658
 33 General Information: 3687
  33.1 Document History: 3689
 PACKAGING INFORMATION: 3706
