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
        let mut configuration_header = [0u32; CONFIGURATION_HEADER_SIZE];
        let device_specific_registers = [0u32; DEVICE_SPECIFIC_REGISTERS];
        let extended_coniguration_registers = [0u32; EXTENDED_CONFIGURATION_REGISTERS];

        // Identify the vendor and the device.
        configuration_header[0] = u32::from(device_id) << 16 | u32::from(vendor_id);

        // Get the programming interface, if any.
        let pi = if let Some(pi) = programming_interface {
            pi.get_register_value()
        } else {
            0
        };

        // Complete the Class Code and the Revision ID.
        configuration_header[2] = u32::from(base_class.get_register_value()) << 24
            | u32::from(subclass.get_register_value()) << 16
            | u32::from(pi) << 8
            | u32::from(revision_id);

        // Determine the layout of the header.
        match header_type {
            PciHeaderType::Type0 => {
                configuration_header[3] = 0x0000_0000;
            }

            PciHeaderType::Type1 => {
                configuration_header[3] = 0x0001_0000;
            }
        }

        // The SSID-SVID combination differentiates specific model, so identifies the card.
        configuration_header[11] = u32::from(subsystem_id) << 16 | u32::from(subsystem_vendor_id);

        PciFunction {
            configuration_header,
            device_specific_registers,
            extended_coniguration_registers,
        }
    }
}
