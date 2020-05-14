// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use constants::{
    PciBaseClass, PciHeaderType, PciProgrammingInterface, PciSubclass, CONFIGURATION_HEADER_SIZE,
    DEVICE_SPECIFIC_REGISTERS, EXTENDED_CONFIGURATION_REGISTERS,
};

/// Functions are designed into every Device.
/// These Functions may include hard drive interfaces, display controllers, etc.
/// Each Function has its own configuration address space which size is 256 bytes (in PCI).
/// PCIe added a new space in configuration: Extended Configuration Registers Space of 960 dwords.
#[derive(Copy, Clone)]
pub struct PciFunction {
    /// The PCI Configuration Header Space: Type0 or Type 1.
    configuration_header: [u32; CONFIGURATION_HEADER_SIZE],

    /// Optional registers (including Capability Structures) that are device specific.
    device_specific_registers: [u32; DEVICE_SPECIFIC_REGISTERS],

    /// Extended Configuration Space, not visible to the legacy PCI.
    extended_coniguration_registers: [u32; EXTENDED_CONFIGURATION_REGISTERS],
}

impl PciFunction {
    pub fn empty() -> Self {
        let configuration_header = [0u32; CONFIGURATION_HEADER_SIZE];
        let device_specific_registers = [0u32; DEVICE_SPECIFIC_REGISTERS];
        let extended_coniguration_registers = [0u32; EXTENDED_CONFIGURATION_REGISTERS];

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
        pci_function.write_configuration_byte(
            3,
            2,
            match header_type {
                PciHeaderType::Type0 => 0x00,
                PciHeaderType::Type1 => 0x01,
            },
        );

        // The SSID-SVID combination differentiates specific model, so identifies the card.
        pci_function.write_configuration_word(11, 1, subsystem_id);
        pci_function.write_configuration_word(11, 0, subsystem_vendor_id);

        pci_function
    }

    /// Read a byte from `offset` of the `index`-th register. Offset is in range 0-3 (byte align).
    fn read_configuration_byte(&self, index: usize, offset: usize) -> Option<u8> {
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
    fn write_configuration_byte(&mut self, index: usize, offset: usize, value: u8) {
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
    fn read_configuration_word(&self, index: usize, offset: usize) -> Option<u16> {
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
    fn write_configuration_word(&mut self, index: usize, offset: usize, value: u16) {
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
    fn read_configuration_dword(&self, index: usize) -> Option<u32> {
        match self.configuration_header.get(index) {
            Some(value) => Some(*value),
            None => {
                eprintln!("The index is out of bounds: {}", index);
                None
            }
        }
    }

    /// Write a dword inside of the `index`-th register.
    fn write_configuration_dword(&mut self, index: usize, value: u32) {
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

    #[test]
    fn configuration_read_write() {
        let mut pci_function = PciFunction::empty();

        // Test read_configuration_byte and write_configuration_byte.
        for index in 0..4 {
            let value = utils::rand::xor_rng_u32() as u8;
            pci_function.write_configuration_byte(0, index, value);
            assert_eq!(pci_function.read_configuration_byte(0, index), Some(value));
        }
        assert_eq!(pci_function.read_configuration_byte(0, 4), None);

        for index in 0..2 {
            // Test read_configuration_word and write_configuration_word.
            let value = utils::rand::xor_rng_u32() as u16;
            pci_function.write_configuration_word(1, index, value);
            assert_eq!(pci_function.read_configuration_word(1, index), Some(value));
        }
        assert_eq!(pci_function.read_configuration_word(1, 2), None);

        // Test read_configuration_dword and write_configuration_dword.
        let value = utils::rand::xor_rng_u32();
        pci_function.write_configuration_dword(2, value);
        assert_eq!(pci_function.read_configuration_dword(2), Some(value));

        // Test combination between them. Compose a dword from 4 bytes.
        let mut v: u32 = 0;
        for index in 0..4 {
            v += u32::from(pci_function.read_configuration_byte(0, index).unwrap()) << (index * 8);
        }
        assert_eq!(pci_function.read_configuration_dword(0), Some(v));
    }
}
