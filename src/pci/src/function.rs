// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::constants::{
    PciBaseClass, PciBridgeSubclass, PciHeaderType, PciProgrammingInterface, PciSubclass,
    PCIE_DUMMY_DEVICE_ID, PCIE_DUMMY_VENDOR_ID,
};

/// The PCI Configuration Header Space has a length of 64 bytes, so 16 dwords.
pub const CONFIGURATION_HEADER_SIZE: usize = 16;

/// The PCI optional registers (device specific) has a length of 192 bytes, so 48 dwords.
pub const DEVICE_SPECIFIC_REGISTERS: usize = 48;

/// The PCIe Extended Configuration Registers Space has a length 3840 bytes, so 960 dwords.
pub const EXTENDED_CONFIGURATION_REGISTERS: usize = 960;

/// Functions are designed into every Device.
/// These Functions may include hard drive interfaces, display controllers, etc.
/// Each Function has its own configuration address space which size is 256 bytes (in PCI).
/// PCIe added a new space in configuration: Extended Configuration Registers Space of 960 dwords.
#[allow(dead_code)]
pub struct PciFunction {
    /// The number of the function within the device.
    number: usize,

    /// The PCI Configuration Header Space: Type0 or Type 1 layout.
    configuration_header: Vec<u32>,

    /// Optional registers (including Capability Structures) that are device specific.
    device_specific_registers: Vec<u32>,

    /// Extended Configuration Space, not visible to the legacy PCI.
    extended_coniguration_registers: Vec<u32>,
}

impl PciFunction {
    /// Create a PCI function.
    pub fn new(
        number: usize,
        device_id: u16,
        vendor_id: u16,
        base_class: PciBaseClass,
        subclass: &dyn PciSubclass,
        programming_interface: Option<&dyn PciProgrammingInterface>,
        revision_id: u8,
        header_type: PciHeaderType,
        subsystem_id: u16,
        subsystem_vendor_id: u16,
    ) -> PciFunction {
        let configuration_header: Vec<u32> = vec![0; CONFIGURATION_HEADER_SIZE];
        let device_specific_registers: Vec<u32> = vec![0; DEVICE_SPECIFIC_REGISTERS];
        let extended_coniguration_registers: Vec<u32> = vec![0; EXTENDED_CONFIGURATION_REGISTERS];

        let mut function = PciFunction {
            number,
            configuration_header,
            device_specific_registers,
            extended_coniguration_registers,
        };

        // Identify the vendor and the device.
        function.write_configuration_word(0, 1, device_id);
        function.write_configuration_word(0, 0, vendor_id);

        // Get the programming interface, if any.
        let pi = if let Some(pi) = programming_interface {
            pi.get_register_value()
        } else {
            0
        };

        // Complete the Class Code and the Revision ID.
        function.write_configuration_byte(1, 3, base_class.get_register_value());
        function.write_configuration_byte(1, 2, subclass.get_register_value());
        function.write_configuration_byte(1, 1, pi);
        function.write_configuration_byte(1, 0, revision_id);

        // Determine the layout of the header.
        match header_type {
            PciHeaderType::Type0 => {
                // Header type.
                function.write_configuration_byte(3, 2, 0x00);

                // The SSID-SVID combination differentiates specific model, so identifies the card.
                function.write_configuration_word(11, 1, subsystem_id);
                function.write_configuration_word(11, 0, subsystem_vendor_id);
            }

            PciHeaderType::Type1 => {
                // Header type.
                function.write_configuration_byte(6, 3, 0x00);

                // Secondary Latency Timer.
                function.write_configuration_byte(6, 3, 0x00);

                // Subordinate Bus Number.
                function.write_configuration_byte(6, 2, 0x00);

                // Secondary Bus Number.
                function.write_configuration_byte(6, 1, 0x00);

                // Primary Bus Number.
                function.write_configuration_byte(6, 0, 0x00);

                // Secondary Status.
                function.write_configuration_word(7, 1, 0x0000);

                // Memory Limit.
                function.write_configuration_word(8, 1, 0xFFFF);

                // Memory Base.
                function.write_configuration_word(8, 0, 0xFFFF);

                // Bridge Control.
                function.write_configuration_word(15, 1, 0xFFFF);
            }
        }

        function
    }

    /// Create a dummy PCI function.
    /// - `number` - the number of the function.
    pub fn new_dummy(number: usize) -> PciFunction {
        PciFunction::new(
            number,
            PCIE_DUMMY_DEVICE_ID,
            PCIE_DUMMY_VENDOR_ID,
            PciBaseClass::BridgeDevice,
            &PciBridgeSubclass::HostBridge,
            None,
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
    /// * `register` - The index of the register within the configuration header space.
    /// * `offset` - The offset within the register. It is in range 0-3 (byte align).
    pub fn read_configuration_byte(&self, register: usize, offset: usize) -> Option<u8> {
        if offset >= 4 {
            return None;
        }

        if let Some(value) = self.configuration_header.get(register) {
            Some((value >> (offset * 8)) as u8)
        } else {
            None
        }
    }

    /// Write a byte to the configuration space.
    /// * `register` - The index of the register within the configuration header space.
    /// * `offset` - The offset within the register. It is in range 0-3 (byte align).
    /// * `data` - The byte to be written.
    pub fn write_configuration_byte(&mut self, register: usize, offset: usize, data: u8) {
        if offset >= 4 {
            return;
        }

        if let Some(register) = self.configuration_header.get_mut(register) {
            // Clean the old value and write the new one.
            *register &= !(0xFF << (offset * 8));
            *register |= u32::from(data) << (offset * 8);
        }
    }

    /// Read a word from the configuration space.
    /// * `register` - The index of the register within the configuration header space.
    /// * `offset` - The offset within the register. It is in range 0-1 (word align).
    pub fn read_configuration_word(&self, register: usize, offset: usize) -> Option<u16> {
        if offset >= 2 {
            return None;
        }

        if let Some(value) = self.configuration_header.get(register) {
            Some((value >> (offset * 16)) as u16)
        } else {
            None
        }
    }

    /// Write a word to the configuration space.
    /// * `register` - The index of the register within the configuration header space.
    /// * `offset` - The offset within the register. It is in range 0-1 (word align).
    /// * `data` - The word to be written.
    pub fn write_configuration_word(&mut self, register: usize, offset: usize, data: u16) {
        if offset >= 2 {
            return;
        }

        if let Some(register) = self.configuration_header.get_mut(register) {
            // Clean the old value and write the new one.
            *register &= !(0xFFFF << (offset * 16));
            *register |= u32::from(data) << (offset * 16);
        }
    }

    /// Read a dword from the configuration space.
    /// * `register` - The index of the register within the configuration header space.
    pub fn read_configuration_dword(&self, register: usize) -> Option<u32> {
        if let Some(value) = self.configuration_header.get(register) {
            Some(*value)
        } else {
            None
        }
    }

    /// Write a dword to the configuration space.
    /// * `register` - The index of the register within the configuration header space.
    /// * `data` - The dword to be written.
    pub fn write_configuration_dword(&mut self, register: usize, data: u32) {
        if let Some(register) = self.configuration_header.get_mut(register) {
            *register = data;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::rand::xor_rng_u32;

    #[test]
    fn function_configuration_read_write_byte() {
        let values = vec![xor_rng_u32() as u8; 4];
        let mut function = PciFunction::new_dummy(0);

        for (pos, value) in values.iter().enumerate() {
            function.write_configuration_byte(pos, pos, *value);
        }

        for (pos, value) in values.iter().enumerate() {
            assert_eq!(function.read_configuration_byte(pos, pos), Some(*value));
        }

        assert!(function.read_configuration_byte(0, 4).is_none());
    }

    #[test]
    fn function_configuration_read_write_word() {
        let values = vec![xor_rng_u32() as u16; 2];
        let mut function = PciFunction::new_dummy(0);

        for (pos, value) in values.iter().enumerate() {
            function.write_configuration_word(pos, pos, *value);
        }

        for (pos, value) in values.iter().enumerate() {
            assert_eq!(function.read_configuration_word(pos, pos), Some(*value));
        }

        assert!(function.read_configuration_word(0, 2).is_none());
    }

    #[test]
    fn function_configuration_read_write_dword() {
        let value = xor_rng_u32();
        let mut function = PciFunction::new_dummy(0);

        function.write_configuration_dword(0, value);
        assert_eq!(function.read_configuration_dword(0), Some(value));

        assert!(function
            .read_configuration_dword(CONFIGURATION_HEADER_SIZE)
            .is_none());
    }

    #[test]
    fn function_configuration_read_write_all() {
        let value = xor_rng_u32();
        let mut function = PciFunction::new_dummy(0);

        // Write `value` as u8 pieces in the first register.
        for index in 0..4 {
            function.write_configuration_byte(0, index, value.to_le_bytes()[index]);
        }

        // Write `value` as u16 pieces in the second register.
        function.write_configuration_word(1, 0, (value & 0xFFFF) as u16);
        function.write_configuration_word(1, 1, ((value >> 16) & 0xFFFF) as u16);

        assert_eq!(
            function.read_configuration_dword(0),
            function.read_configuration_dword(1)
        );
    }
}
