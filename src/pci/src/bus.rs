// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use constants::MAX_DEVICE_NUMBER;
use device::PciDevice;

/// Each Bus must be assigned a unique bus number.
/// The initial Bus Number, Bus 0, is typically assigned to the Root Complex.
#[derive(Copy, Clone)]
pub struct PciBus {
    devices: [PciDevice; MAX_DEVICE_NUMBER],
}

impl PciBus {
    pub fn new() -> Self {
        let devices = [PciDevice::new(); MAX_DEVICE_NUMBER];

        PciBus { devices }
    }
}

/// Emulate the PCI Root Complex node of the PCIe topology.
/// This component generates transaction requests on behalf of the processor.
/// This hardware component may contain different interfaces (CPU, DRAM) and chips.
pub struct PciRootComplex {
    bus: PciBus,
}

impl PciRootComplex {
    pub fn new() -> Self {
        let bus = PciBus::new();

        PciRootComplex { bus }
    }
}
