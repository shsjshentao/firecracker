// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::bus::PciBus;
use crate::device::PciDevice;
use devices::BusDevice;
use polly::event_manager::{EventManager, Subscriber};
use std::sync::{Arc, Mutex};
use utils::byte_order::{read_le_u16, read_le_u32};
use utils::epoll::EpollEvent;

pub const PCI_IO_PORT: usize = 0xCF8;
pub const PCI_IO_PORT_SIZE: usize = 0x8;

/// Offset of the CONFIG_ADDRESS port (port 0xCF8),
const OFFSET_ADDRESS: u64 = 0;
const OFFSET_ADDRESS_END: u64 = 3;

/// Offset of the CONFIG_DATA port (port 0xCFC),
const OFFSET_DATA: u64 = 4;
const OFFSET_DATA_END: u64 = 7;

/// Emulate the PCI Root Complex node of the PCIe topology.
/// This component generates transaction requests on behalf of the processor.
/// This hardware component may contain different interfaces (CPU, DRAM) and chips.
///
/// This component is the first PCI bus, the bus with number 0. It will be connected to the PIO.
pub struct PciRootComplex {
    /// The bus connected to the PCI Root Complex component (bus number 0).
    bus: Arc<Mutex<PciBus>>,

    /// The last value written to the port 0xCF8.
    config_address: u32,
}

impl PciRootComplex {
    /// Return a new PCI Root Complex node which does not have any device attached.
    pub fn new() -> Self {
        let mut bus = PciBus::new(0);

        // Add the Host Bridge device on bus 0, device 0, function 0.
        bus.add_device(PciDevice::new_dummy_host_bridge(0)).unwrap();

        PciRootComplex {
            bus: Arc::new(Mutex::new(bus)),
            config_address: 0x0000_0000,
        }
    }

    /// Return the last value written to the `0xCF8` port.
    pub fn get_configuration_address(&self) -> u32 {
        self.config_address
    }

    /// Store the last value written to the `0xCF8` port.
    /// - `offset` - offset from where to start writing within the address.
    /// - `data` - array of bytes to be written.
    pub fn set_configuration_address(&mut self, offset: u64, data: &[u8]) {
        // Make sure the boundary is respected.
        if offset as usize + data.len() > 4 {
            return;
        }

        let (mask, config_address): (u32, u32) = match data.len() {
            1 => (
                0x0000_00FF << (offset * 8),
                (data[0] as u32) << (offset * 8),
            ),
            2 => (0x0000_FFFF, (read_le_u16(data) as u32) << (offset * 16)),
            4 => (0xFFFF_FFFF, read_le_u32(data)),
            _ => return,
        };

        self.config_address = (self.config_address & !mask) | config_address;
    }

    /// Read a dword from the configuration space.
    /// Get the address from `self.config_address` field.
    pub fn read_configuration_space(&self) -> u32 {
        // Probe if the Enable Configuration Space Mapping is set, otherwise ignore transaction.
        let enabled = (self.config_address & 0x8000_0000) != 0;
        if !enabled {
            return 0xFFFF_FFFF;
        }

        let (bus, device, function, register) = self.parse_configuration_address();

        self.bus
            .lock()
            .unwrap()
            .read_configuration_register(bus, device, function, register)
            .unwrap_or(0xFFFF_FFFF)
    }

    /// Write to the configure space.
    /// - `offset`- offset from where to start writing the data.
    /// - `data` - array of bytes to be written.
    pub fn write_configuration_space(&mut self, offset: u64, data: &[u8]) {
        // Make sure the boundaries are respected.
        if offset as usize + data.len() > 4 {
            return;
        }

        // Probe if the Enable Configuration Space Mapping is set, otherwise ignore transaction.
        let enabled = (self.config_address & 0x8000_0000) != 0;
        if !enabled {
            return;
        }

        let (bus, device, function, register) = self.parse_configuration_address();

        self.bus.lock().unwrap().write_configuration_register(
            bus,
            device,
            function,
            register,
            offset as usize,
            data,
        )
    }

    /// Parse the stored configuration address (the last value written to `0xCF8`).
    /// Return a tuple of (bus, device, function, register pointer).
    pub fn parse_configuration_address(&self) -> (usize, usize, usize, usize) {
        const BUS_NUMBER_OFFSET: usize = 16;
        const BUS_NUMBER_MASK: u32 = 0x00FF;

        const DEVICE_NUMBER_OFFSET: usize = 11;
        const DEVICE_NUMBER_MASK: u32 = 0x1F;

        const FUNCTION_NUMBER_OFFSET: usize = 8;
        const FUNCTION_NUMBER_MASK: u32 = 0x07;

        const REGISTER_NUMBER_OFFSET: usize = 2;
        const REGISTER_NUMBER_MASK: u32 = 0x3F;

        (
            ((self.config_address >> BUS_NUMBER_OFFSET) & BUS_NUMBER_MASK) as usize,
            ((self.config_address >> DEVICE_NUMBER_OFFSET) & DEVICE_NUMBER_MASK) as usize,
            ((self.config_address >> FUNCTION_NUMBER_OFFSET) & FUNCTION_NUMBER_MASK) as usize,
            ((self.config_address >> REGISTER_NUMBER_OFFSET) & REGISTER_NUMBER_MASK) as usize,
        )
    }
}

impl BusDevice for PciRootComplex {
    /// Read from Address Port or Data Port deciding by the offset relative to 0xCF8.
    fn read(&mut self, offset: u64, data: &mut [u8]) {
        let result: u32 = match offset {
            // Return the configuration address.
            OFFSET_ADDRESS..=OFFSET_ADDRESS_END => self.config_address,
            // Return data from the device.
            OFFSET_DATA..=OFFSET_DATA_END => self.read_configuration_space(),
            // Error, return all ones.
            _ => 0xFFFF_FFFF,
        };

        // Allow only if the boundary is respected and start from the beginning of the space.
        let start = offset as usize % 4;
        let end = start + data.len();

        if end <= 4 {
            // The LSB comes first into result.
            for index in start..end {
                data[index - start] = (result >> (index * 8)) as u8;
            }
        } else {
            // Invalid read, fill with ones.
            for byte in data {
                *byte = 0xFF;
            }
        }
    }

    /// Write to Address Port or Data Port deciding by the offset relative to 0xCF8.
    fn write(&mut self, offset: u64, data: &[u8]) {
        match offset {
            // Set a new configuration address.
            OFFSET_ADDRESS..=OFFSET_ADDRESS_END => self.set_configuration_address(offset, data),
            // Make a write in a memory of a device.
            OFFSET_DATA..=OFFSET_DATA_END => self.write_configuration_space(offset - 4, data),
            _ => {}
        }
    }
}

impl Subscriber for PciRootComplex {
    /// This function is called when an event is available.
    fn process(&mut self, _event: &EpollEvent, _event_manager: &mut EventManager) {}

    /// Returns a list of `EpollEvent` that this subscriber is interested in.
    fn interest_list(&self) -> Vec<EpollEvent> {
        vec![]
    }
}
