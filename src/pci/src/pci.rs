// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::bus::PciBus;

pub const PCI_ROOT_COMPLEX_NUMBER: usize = 0;

/// Emulate the PCI Root Complex node of the PCIe topology.
/// This component generates transaction requests on behalf of the processor.
/// This hardware component may contain different interfaces (CPU, DRAM) and chips.
///
/// This component is the first PCI bus, the bus with number 0. It will be connected to the PIO.
pub struct PciRootComplex {
    bus: PciBus,
}

impl PciRootComplex {
    pub fn new() -> Self {
        PciRootComplex {
            bus: PciBus::new(PCI_ROOT_COMPLEX_NUMBER as u32),
        }
    }
}
