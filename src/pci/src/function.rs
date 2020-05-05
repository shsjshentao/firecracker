// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use constants::{PciHeaderType, CAPABILITY_STRUCTURES_SIZE, CONFIGURATION_HEADER_SIZE};

/// Functions are designed into every Device.
/// These Functions may include hard drive interfaces, display controllers, etc.
/// Each Function has its own configuration address space which size is 256 bytes (in PCI).
/// PCIe added a new space in configuration: Extended Configuration Registers Space of 960 dwords.
pub struct PciFunction {
    /// The PCI Configuration Header Space: Type0 or Type 1.
    configuration_header: [u32; CONFIGURATION_HEADER_SIZE],

    /// Optional registers (including Capability Structures) that are device specific.
    device_specific_registers: [u32; CAPABILITY_STRUCTURES_SIZE],

    /// Extended Configuration Space, not visible to the legacy PCI.
    extended_coniguration_registers: [u32; EXTENDED_CONFIGURATION_REGISTERS],
}

impl PciFunction {
    pub fn new() {
        let mut configuration_header = [0u32; CONFIGURATION_HEADER_SIZE];
        let mut device_specific_registers = [0u32; CAPABILITY_STRUCTURES_SIZE];
        let mut extended_coniguration_registers = [0u32; EXTENDED_CONFIGURATION_REGISTERS];

        PciFunction {
            configuration_header,
            capability_structures,
            extended_coniguration_registers,
        }
    }
}
