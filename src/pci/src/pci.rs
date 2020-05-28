// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::bus::PciBus;
use devices::BusDevice;
use polly::event_manager::{EventManager, Subscriber};
use std::io;
use utils::epoll::EpollEvent;

pub const PCI_ROOT_COMPLEX_NUMBER: usize = 0;

pub const PCI_IO_ADDRESS_PORT: usize = 0xCF8;
pub const PCI_IO_DATA_PORT: usize = 0xCFC;
pub const PCI_IO_PORT_SIZE: usize = 0x4;

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

impl BusDevice for PciRootComplex {
    fn read(&mut self, offset: u64, data: &mut [u8]) {}

    fn write(&mut self, offset: u64, data: &[u8]) {}

    fn interrupt(&self, irq_mask: u32) -> io::Result<()> {
        Ok(())
    }
}

impl Subscriber for PciRootComplex {
    /// This function is called when an event is available.
    fn process(&mut self, event: &EpollEvent, event_manager: &mut EventManager) {}

    /// Returns a list of `EpollEvent` that this subscriber is interested in.
    fn interest_list(&self) -> Vec<EpollEvent> {
        vec![]
    }
}

#[cfg(test)]
mod tests {}
