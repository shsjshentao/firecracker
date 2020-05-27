// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

/// There are up to 256 Bus numbers that can be assigned.
pub const MAX_BUS_NUMBER: usize = 256;

/// There are up to 32 Device attachments on a single PCI Bus.
pub const MAX_DEVICE_NUMBER: usize = 32;

/// A Device can have implemented up to 8 Functions (not necessarily sequentially).
pub const MAX_FUNCTION_NUMBER: usize = 8;

/// Type 0 is required for every Function, except for the Bridge Functions.
/// Type 1 is required for the Bridge (Switch) Functions.
#[derive(Clone, Copy)]
pub enum PciHeaderType {
    Type0,
    Type1,
}

/// The upper byte of a Class Code.
/// Classifies the type of functionality that the Function of the Device provides.
/// https://pcisig.com/sites/default/files/files/PCI_Code-ID_r_1_11__v24_Jan_2019.pdf
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum PciBaseClass {
    UnclassifiedDevice = 0x00,
    MassStorageController = 0x01,
    NetworkController = 0x02,
    DisplayController = 0x03,
    MultimediaController = 0x04,
    MemoryController = 0x05,
    BridgeDevice = 0x06,
    SimpleCommunicationController = 0x07,
    BaseSystemPeripheral = 0x08,
    InputDeviceController = 0x09,
    DockingStation = 0x0A,
    Processor = 0x0B,
    SerialBusController = 0x0C,
    WirelessController = 0x0D,
    IntelligentIOController = 0x0E,
    SatelliteCommunicationController = 0x0F,
    EncryptionController = 0x10,
    SignalProcessingController = 0x11,
    ProcessingAccelerator = 0x12,
    NonEssentialInstrumentation = 0x13,
    Other = 0xFF,
}

impl PciBaseClass {
    pub fn get_register_value(self) -> u8 {
        self as u8
    }
}

/// The middle byte of a Class Code.
/// Identifies the type of functionality that the the Function of the Device provides.
/// https://pcisig.com/sites/default/files/files/PCI_Code-ID_r_1_11__v24_Jan_2019.pdf
pub trait PciSubclass {
    fn get_register_value(&self) -> u8;
}

/// Subclasses of the Bridge Device.
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum PciBridgeSubclass {
    HostBridge = 0x00,
    IsaBridge = 0x01,
    EisaBridge = 0x02,
    McaBridge = 0x03,
    PciToPciBridge = 0x04,
    PcmciaBridge = 0x05,
    NuBusBridge = 0x06,
    CardBusBridge = 0x07,
    RACEwayBridge = 0x08,
    PciToPciSemiTransparentBridge = 0x09,
    InfiniBandToPciHostBridge = 0x0A,
    OtherBridgeDevice = 0x80,
}

impl PciSubclass for PciBridgeSubclass {
    fn get_register_value(&self) -> u8 {
        *self as u8
    }
}

/// The lower byte of a Class Code.
/// Identifies the specific register-level interface (if any) of a Function.
/// https://pcisig.com/sites/default/files/files/PCI_Code-ID_r_1_11__v24_Jan_2019.pdf
pub trait PciProgrammingInterface {
    fn get_register_value(&self) -> u8;
}
