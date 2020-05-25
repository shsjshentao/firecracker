// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use devices::BusDevice;

/// Each Device must implement Function 0 and may contain a collection up to 8 Functions.
/// A Device might implement Functions 0, 2 and 7 and there is no need to be sequentially.
pub trait PciDevice: BusDevice {}

/// Emulate the PCI Root Complex node of the PCIe topology.
/// This component generates transaction requests on behalf of the processor.
/// This hardware component may contain different interfaces (CPU, DRAM) and chips.
pub struct PciRootComplex {}

impl PciRootComplex {
    pub fn new() -> Self {
        PciRootComplex {}
    }
}

impl BusDevice for PciRootComplex {}

impl PciDevice for PciRootComplex {}
