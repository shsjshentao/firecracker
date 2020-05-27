// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::constants::{PciBaseClass, PciHeaderType, PciProgrammingInterface, PciSubclass};

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
#[derive(Clone)]
pub struct PciFunction {
    /// The PCI Configuration Header Space: Type0 or Type 1.
    configuration_header: Vec<u32>,

    /// Optional registers (including Capability Structures) that are device specific.
    device_specific_registers: Vec<u32>,

    /// Extended Configuration Space, not visible to the legacy PCI.
    extended_coniguration_registers: Vec<u32>,
}

impl PciFunction {
    /// Create a PCI function having the configuration space full of zeroes.
    pub fn empty() -> Self {
        let configuration_header: Vec<u32> = vec![0; CONFIGURATION_HEADER_SIZE as usize];
        let device_specific_registers: Vec<u32> = vec![0; DEVICE_SPECIFIC_REGISTERS as usize];
        let extended_coniguration_registers: Vec<u32> =
            vec![0; EXTENDED_CONFIGURATION_REGISTERS as usize];

        PciFunction {
            configuration_header,
            device_specific_registers,
            extended_coniguration_registers,
        }
    }

    pub fn new(
        device_id: u16,
        vendor_id: u16,
        base_class: PciBaseClass,
        subclass: &dyn PciSubclass,
        programming_interface: Option<&dyn PciProgrammingInterface>,
        revision_id: u8,
        header_type: PciHeaderType,
        subsystem_id: u16,
        subsystem_vendor_id: u16,
    ) -> Self {
        let mut pci_function = PciFunction::empty();

        // Identify the vendor and the device.
        pci_function.write_configuration_word(0, 1, device_id);
        pci_function.write_configuration_word(0, 0, vendor_id);

        // Get the programming interface, if any.
        let pi = if let Some(pi) = programming_interface {
            pi.get_register_value()
        } else {
            0
        };

        // Complete the Class Code and the Revision ID.
        pci_function.write_configuration_byte(1, 3, base_class.get_register_value());
        pci_function.write_configuration_byte(1, 2, subclass.get_register_value());
        pci_function.write_configuration_byte(1, 1, pi);
        pci_function.write_configuration_byte(1, 0, revision_id);

        // Determine the layout of the header.
        match header_type {
            PciHeaderType::Type0 => {
                // Header type.
                pci_function.write_configuration_byte(3, 2, 0x00);

                // The SSID-SVID combination differentiates specific model, so identifies the card.
                pci_function.write_configuration_word(11, 1, subsystem_id);
                pci_function.write_configuration_word(11, 0, subsystem_vendor_id);
            }

            PciHeaderType::Type1 => {
                // Header type.
                pci_function.write_configuration_byte(6, 3, 0x00);

                // Secondary Latency Timer.
                pci_function.write_configuration_byte(6, 3, 0x00);

                // Subordinate Bus Number.
                pci_function.write_configuration_byte(6, 2, 0x00);

                // Secondary Bus Number.
                pci_function.write_configuration_byte(6, 1, 0x00);

                // Primary Bus Number.
                pci_function.write_configuration_byte(6, 0, 0x00);

                // Secondary Status.
                pci_function.write_configuration_word(7, 1, 0x0000);

                // Memory Limit.
                pci_function.write_configuration_word(8, 1, 0xFFFF);

                // Memory Base.
                pci_function.write_configuration_word(8, 0, 0xFFFF);

                // Bridge Control.
                pci_function.write_configuration_word(15, 1, 0xFFFF);
            }
        }

        pci_function
    }

    /// Read a byte from `offset` of the `index`-th register. Offset is in range 0-3 (byte align).
    pub fn read_configuration_byte(&self, index: usize, offset: usize) -> Option<u8> {
        if offset >= 4 {
            eprintln!("The offset is out of bounds: {}", offset);
            return None;
        }

        match self.configuration_header.get(index) {
            Some(value) => {
                // Convert to a bit-offset.
                let offset = offset % 4 * 8;

                let byte = *value & (0xFF << offset);
                Some((byte >> offset) as u8)
            }
            None => {
                eprintln!("The index is out of bounds: {}", index);
                None
            }
        }
    }

    /// Write byte at `offset` inside of `index`-th register. Offset is in range 0-3  (byte align).
    pub fn write_configuration_byte(&mut self, index: usize, offset: usize, value: u8) {
        if offset >= 4 {
            eprintln!("The offset is out of bounds: {}", offset);
            return;
        }

        match self.configuration_header.get_mut(index) {
            Some(element) => {
                // Convert to a bit-offset.
                let offset = offset % 4 * 8;

                // Clean the old value and write the new one.
                *element &= !(0xFF << offset);
                *element |= u32::from(value) << offset;
            }
            None => {
                eprintln!("The index is out of bounds: {}", index);
            }
        }
    }

    /// Read a word from `offset` of the `index`-th register. Offset is in range 0-1 (word align).
    pub fn read_configuration_word(&self, index: usize, offset: usize) -> Option<u16> {
        if offset >= 2 {
            eprintln!("The offset is out of bounds: {}", offset);
            return None;
        }

        match self.configuration_header.get(index) {
            Some(value) => {
                // Convert to a bit-offset.
                let offset = offset % 2 * 16;

                let word = *value & (0xFFFF << offset);
                Some((word >> offset) as u16)
            }

            None => {
                eprintln!("The index is out of bounds: {}", index);
                None
            }
        }
    }

    /// Write word at `offset` inside of `index`-th register. Offset is in range 0-1  (word align).
    pub fn write_configuration_word(&mut self, index: usize, offset: usize, value: u16) {
        if offset >= 2 {
            eprintln!("The offset is out of bounds: {}", offset);
            return;
        }

        match self.configuration_header.get_mut(index) {
            Some(element) => {
                // Convert to a bit-offset.
                let offset = offset % 2 * 16;

                // Clean the old value and write the new one.
                *element &= !(0xFFFF << offset);
                *element |= u32::from(value) << offset;
            }
            None => {
                eprintln!("The index is out of bounds: {}", index);
            }
        }
    }

    /// Read a word from `offset` of the `index`-th register.
    pub fn read_configuration_dword(&self, index: usize) -> Option<u32> {
        match self.configuration_header.get(index) {
            Some(value) => Some(*value),
            None => {
                eprintln!("The index is out of bounds: {}", index);
                None
            }
        }
    }

    /// Write a dword inside of the `index`-th register.
    pub fn write_configuration_dword(&mut self, index: usize, value: u32) {
        match self.configuration_header.get_mut(index) {
            Some(element) => *element = value,
            None => {
                eprintln!("The index is out of bounds: {}", index);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate utils;

    /// Test combination between read/write methods, composing a dword from 4 bytes.
    #[test]
    fn function_configuration_read_write() {
        let value = utils::rand::xor_rng_u32();
        let mut pci_function = PciFunction::empty();

        for index in 0..4 {
            pci_function.write_configuration_byte(0, index, value.to_le_bytes()[index]);
        }
        assert_eq!(pci_function.read_configuration_dword(0), Some(value));
    }

    #[test]
    fn function_configuration_read_write_byte() {
        let mut pci_function = PciFunction::empty();

        for index in 0..4 {
            let value = utils::rand::xor_rng_u32() as u8;
            pci_function.write_configuration_byte(0, index, value);
            assert_eq!(pci_function.read_configuration_byte(0, index), Some(value));
        }
        assert_eq!(pci_function.read_configuration_byte(0, 4), None);
    }

    #[test]
    fn function_configuration_read_write_word() {
        let mut pci_function = PciFunction::empty();

        for index in 0..2 {
            let value = utils::rand::xor_rng_u32() as u16;
            pci_function.write_configuration_word(1, index, value);
            assert_eq!(pci_function.read_configuration_word(1, index), Some(value));
        }
        assert_eq!(pci_function.read_configuration_word(1, 2), None);
    }

    #[test]
    fn function_configuration_read_write_dword() {
        let value = utils::rand::xor_rng_u32();
        let mut pci_function = PciFunction::empty();

        pci_function.write_configuration_dword(2, value);
        assert_eq!(pci_function.read_configuration_dword(2), Some(value));
    }
}
