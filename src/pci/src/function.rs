// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

/// The PCIe Configuration Header Space has a length of 64 bytes, so 16 dwords.
pub const CONFIGURATION_HEADER_SIZE: usize = 16;

/// The PCIe optional registers (device specific) has a length of 192 bytes, so 48 dwords.
pub const CAPABILITY_REGISTERS_SIZE: usize = 48;

/// The PCIe Extended Configuration Registers Space has a length 3840 bytes, so 960 dwords.
pub const EXTENDED_CONFIGURATION_SIZE: usize = 960;

pub const CONFIGURATION_SPACE_SIZE: usize =
    CONFIGURATION_HEADER_SIZE + CAPABILITY_REGISTERS_SIZE + EXTENDED_CONFIGURATION_SIZE;

// Configuration space meanings as registers and offsets.
pub const VENDOR_ID_REGISTER: usize = 0;
pub const VENDOR_ID_OFFSET: usize = 0;

pub const DEVICE_ID_REGISTER: usize = 0;
pub const DEVICE_ID_OFFSET: usize = 2;

pub const COMMAND_REGISTER: usize = 1;
pub const COMMAND_OFFSET: usize = 0;

pub const STATUS_REGISTER: usize = 1;
pub const STATUS_OFFSET: usize = 2;

pub const CLASS_CODE_REGISTER: usize = 2;

pub const REVISION_ID_REGISTER: usize = 2;
pub const REVISION_ID_OFFSET: usize = 0;

pub const HEADER_TYPE_REGISTER: usize = 3;
pub const HEADER_TYPE_OFFSET: usize = 2;

pub const SUBSYSTEM_ID_REGISTER: usize = 11;
pub const SUBSYSTEM_ID_OFFSET: usize = 2;

pub const SUBSYSTEM_VENDOR_ID_REGISTER: usize = 11;
pub const SUBSYSTEM_VENDOR_ID_OFFSET: usize = 0;

// https://pci-ids.ucw.cz/read/PC/1d94/1452
pub const VENDOR_ID_DUMMY_HOST_BRIDGE: u16 = 0x1D94;
pub const DEVICE_ID_DUMMY_HOST_BRIDGE: u16 = 0x1452;

/// Type 0 is required for every Function, except for the Bridge Functions.
/// Type 1 is required for the Bridge (Switch) Functions.
#[derive(Clone, Copy)]
pub enum PciHeaderType {
    Type0,
    Type1,
}

/// Return an u32, which the first 3 upper bytes are:
/// - `Base Class` - the upper byte, which broadly classifies the type of function.
/// - `Sub-Class` - the middle byte, which more specifically identifies the type of function.
/// - `Programming Interface` - the lower byte, which identifies the specific interface (if any).
///
/// More information at:
/// https://pcisig.com/sites/default/files/files/PCI_Code-ID_r_1_11__v24_Jan_2019.pdf
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum PciClassCode {
    // Base Class - 0x00 (Unclassified Devices).
    AllImplementedExceptVGACompatible = 0x00_00_00_00,
    VGAComptabile = 0x00_01_00_00,

    // Base Class - 0x06 (Bridge Devices).
    HostBridge = 0x06_00_00_00,
    IsaBridge = 0x06_01_00_00,
    EisaBRidge = 0x06_02_00_00,
    McaBridge = 0x06_03_00_00,
    PciToPciBridge = 0x06_04_00_00,
    SubstractivePciToPciBridge = 0x06_04_01_00,
    PcmciaBridge = 0x06_05_00_00,
    NuBusBridge = 0x06_06_00_00,
    CardBusBridge = 0x06_07_00_00,
    RacewayBridge = 0x06_08_00_00,
    PrimaryProcessorPciToPciBridge = 0x06_09_40_00,
    SecondaryProcessorPciToPciBridge = 0x06_09_80_00,
    InfiniBandToPciHostBridge = 0x06_0A_00_00,
    AdvancedSwitchingToPciHostCustom = 0x06_0B_00_00,
    AdvancedSwitchingToPciHostAsiSig = 0x06_0B_01_00,
    OtherBridgeDevice = 0x06_80_00_00,

    // Base Class - 0x09 (Input Devices).
    KeyboardController = 0x09_00_00_00,
    DigitizerPen = 0x09_01_00_00,
    MouseController = 0x09_02_00_00,
    ScannerController = 0x09_03_00_00,
    GameportControllerGeneric = 0x09_04_00_00,
    GameportController = 0x09_04_10_00,
    OtherInputController = 0x09_80_00_00,
}

impl PciClassCode {
    pub fn get_register_value(self) -> u32 {
        self as u32
    }
}

/// Functions are designed into every Device.
/// These Functions may include hard drive interfaces, display controllers, etc.
/// Each Function has its own configuration address space which size is 256 bytes (in PCI).
/// PCIe added a new space in configuration: Extended Configuration Registers Space of 960 dwords.
#[allow(dead_code)]
pub struct PciFunction {
    /// The number of the function within the device.
    number: usize,

    /// The PCIe Configuration Space. It has 1024 dwords (so 4KB) and contains:
    /// - `PCI Configuration Header` - 16 dwords.
    /// - `PCI Device-specific & New Capability registers` - 48 dwords.
    /// - `PCIe Extended Configuration Register Space` - 960 dwords.
    configuration_space: Vec<u32>,
}

impl PciFunction {
    /// Create a PCI function.
    pub fn new(
        number: usize,
        device_id: u16,
        vendor_id: u16,
        class_code: PciClassCode,
        revision_id: u8,
        header_type: PciHeaderType,
        subsystem_id: u16,
        subsystem_vendor_id: u16,
    ) -> PciFunction {
        let mut function = PciFunction {
            number,
            configuration_space: vec![0; CONFIGURATION_SPACE_SIZE],
        };

        function.write_configuration_word(DEVICE_ID_REGISTER, DEVICE_ID_OFFSET, device_id);
        function.write_configuration_word(VENDOR_ID_REGISTER, VENDOR_ID_OFFSET, vendor_id);

        function.write_configuration_word(COMMAND_REGISTER, COMMAND_OFFSET, 0xFFFF);
        function.write_configuration_word(STATUS_REGISTER, STATUS_OFFSET, 0xFFFF);

        function.write_configuration_dword(CLASS_CODE_REGISTER, class_code.get_register_value());
        function.write_configuration_byte(REVISION_ID_REGISTER, REVISION_ID_OFFSET, revision_id);

        match header_type {
            PciHeaderType::Type0 => {
                function.write_configuration_byte(HEADER_TYPE_REGISTER, HEADER_TYPE_OFFSET, 0x00);

                function.write_configuration_word(
                    SUBSYSTEM_ID_REGISTER,
                    SUBSYSTEM_ID_OFFSET,
                    subsystem_id,
                );

                function.write_configuration_word(
                    SUBSYSTEM_VENDOR_ID_REGISTER,
                    SUBSYSTEM_VENDOR_ID_OFFSET,
                    subsystem_vendor_id,
                );
            }

            PciHeaderType::Type1 => {
                function.write_configuration_byte(HEADER_TYPE_REGISTER, HEADER_TYPE_OFFSET, 0x01);
            }
        }

        function
    }

    /// Create a dummy PCI Host Bridge function.
    /// - `number` - the number of the function.
    pub fn new_dummy_host_bridge(number: usize) -> PciFunction {
        PciFunction::new(
            number,
            DEVICE_ID_DUMMY_HOST_BRIDGE,
            VENDOR_ID_DUMMY_HOST_BRIDGE,
            PciClassCode::HostBridge,
            0,
            PciHeaderType::Type0,
            0,
            0,
        )
    }

    /// Return the number of this function.
    pub fn get_number(&self) -> usize {
        self.number
    }

    /// Read a byte from the configuration space.
    /// * `register` - The index of the register within the given space.
    /// * `offset` - The offset within the register. It is in range 0-3 (byte align).
    pub fn read_configuration_byte(&self, register: usize, offset: usize) -> Option<u8> {
        if offset > 3 {
            return None;
        }

        if let Some(value) = self.configuration_space.get(register) {
            Some((value >> (offset * 8)) as u8)
        } else {
            None
        }
    }

    /// Read a word from the configuration space.
    /// * `register` - The index of the register within the given space.
    /// * `offset` - The offset within the register. It is in range 0-2 (byte align).
    pub fn read_configuration_word(&self, register: usize, offset: usize) -> Option<u16> {
        if offset > 2 {
            return None;
        }

        if let Some(value) = self.configuration_space.get(register) {
            Some((value >> (offset * 8)) as u16)
        } else {
            None
        }
    }

    /// Read a dword from the configuration space.
    /// * `register` - The index of the register within the given space.
    pub fn read_configuration_dword(&self, register: usize) -> Option<u32> {
        if let Some(value) = self.configuration_space.get(register) {
            Some(*value)
        } else {
            None
        }
    }

    /// Write a byte to the configuration space.
    /// * `register` - The index of the register within the given space.
    /// * `offset` - The offset within the register. It is in range 0-3 (byte align).
    /// * `data` - The byte to be written.
    pub fn write_configuration_byte(&mut self, register: usize, offset: usize, data: u8) {
        if offset > 3 {
            return;
        }

        if let Some(register) = self.configuration_space.get_mut(register) {
            // Clean the old value and write the new one.
            *register &= !(0xFF << (offset * 8));
            *register |= (data as u32) << (offset * 8);
        }
    }

    /// Write a word to the configuration space.
    /// * `register` - The index of the register within the given space.
    /// * `offset` - The offset within the register. It is in range 0-2 (byte align).
    /// * `data` - The word to be written.
    pub fn write_configuration_word(&mut self, register: usize, offset: usize, data: u16) {
        if offset > 2 {
            return;
        }

        if let Some(register) = self.configuration_space.get_mut(register) {
            // Clean the old value and write the new one.
            *register &= !(0xFFFF << (offset * 8));
            *register |= (data as u32) << (offset * 8);
        }
    }

    /// Write a dword to the configuration space.
    /// * `register` - The index of the register within the given space.
    /// * `data` - The dword to be written.
    pub fn write_configuration_dword(&mut self, register: usize, data: u32) {
        if let Some(register) = self.configuration_space.get_mut(register) {
            *register = data;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PciFunction, CLASS_CODE_REGISTER, CONFIGURATION_SPACE_SIZE};
    use utils::rand::xor_rng_u32;

    fn get_function() -> PciFunction {
        PciFunction::new_dummy_host_bridge(0)
    }

    #[test]
    fn function_configuration_read_write_invalid() {
        let mut function = get_function();
        let value = xor_rng_u32();

        // Test the register index - out of bounds.
        assert!(function
            .read_configuration_dword(CONFIGURATION_SPACE_SIZE)
            .is_none());

        // Test invalid offsets.
        function.write_configuration_dword(0, value);
        function.write_configuration_byte(0, 4, xor_rng_u32() as u8);
        function.write_configuration_word(0, 3, xor_rng_u32() as u16);
        function.write_configuration_word(0, 4, xor_rng_u32() as u16);
        assert_eq!(function.read_configuration_dword(0), Some(value));
    }

    #[test]
    fn function_configuration_read_write_byte() {
        let mut function = get_function();
        let values = vec![xor_rng_u32() as u8; 4];

        for (pos, value) in values.iter().enumerate() {
            function.write_configuration_byte(pos, pos, *value);
        }

        for (pos, value) in values.iter().enumerate() {
            assert_eq!(function.read_configuration_byte(pos, pos), Some(*value));
        }
    }

    #[test]
    fn function_configuration_read_write_word() {
        let mut function = get_function();
        let values = vec![xor_rng_u32() as u16; 2];

        function.write_configuration_word(0, 1, values[0]);
        function.write_configuration_word(1, 2, values[1]);

        assert_eq!(function.read_configuration_word(0, 1), Some(values[0]));
        assert_eq!(function.read_configuration_word(1, 2), Some(values[1]));
    }

    #[test]
    fn function_configuration_read_write_dword() {
        let mut function = get_function();
        let value = xor_rng_u32();

        function.write_configuration_dword(0, value);
        assert_eq!(function.read_configuration_dword(0), Some(value));
    }

    #[test]
    fn function_configuration_read_write_all() {
        let mut function = get_function();
        let value = xor_rng_u32();

        // Write `value` as u8 pieces in the first register.
        for index in 0..4 {
            function.write_configuration_byte(0, index, value.to_le_bytes()[index]);
        }

        // Write `value` as u16 pieces in the second register.
        function.write_configuration_word(1, 0, (value & 0xFFFF) as u16);
        function.write_configuration_word(1, 2, ((value >> 16) & 0xFFFF) as u16);

        assert_eq!(
            function.read_configuration_dword(0),
            function.read_configuration_dword(1)
        );
    }

    #[test]
    fn dummy_class_code() {
        let function = get_function();

        assert_eq!(
            function
                .read_configuration_byte(CLASS_CODE_REGISTER, 3)
                .unwrap(),
            0x06
        );

        assert_eq!(
            function
                .read_configuration_byte(CLASS_CODE_REGISTER, 2)
                .unwrap(),
            0x00
        );

        assert_eq!(
            function
                .read_configuration_byte(CLASS_CODE_REGISTER, 1)
                .unwrap(),
            0x00
        );
    }
}
