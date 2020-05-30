// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::constants::{MAX_BUS_NUMBER, MAX_DEVICE_NUMBER};
use crate::device::PciDevice;
use std::collections::HashMap;
use std::option::Option;
use std::sync::{Arc, Mutex};

/// Errors for the PciBus.
#[derive(Debug)]
pub enum PciBusError {
    /// Invalid PCI bus number provided.
    InvalidPciBusNumber(usize),
    /// Valid PCI bus number  but already used.
    AlreadyInUsePciBusSlot(usize),
    /// Invalid PCI device number provided.
    InvalidPciDeviceNumber(usize),
    /// Valid PCI device number but already used.
    AlreadyInUsePciDeviceSlot(usize),
}

pub type Result<T> = std::result::Result<T, PciBusError>;

/// Each Bus must be assigned a unique bus number.
/// The initial Bus Number, Bus 0, is typically assigned to the Root Complex.
pub struct PciBus {
    /// The number of the bus.
    number: usize,

    /// The other buses that are connected to this bus.
    buses: HashMap<usize, Arc<Mutex<PciBus>>>,

    /// The device that are connected to this bus.
    devices: HashMap<usize, Arc<Mutex<PciDevice>>>,
}

impl PciBus {
    pub fn new(number: usize) -> PciBus {
        PciBus {
            number,
            buses: HashMap::new(),
            devices: HashMap::new(),
        }
    }

    /// Create a dummy PCI bus which contains a dummy PCI device on the 0 slot.
    /// - `number` - the number of the bus.
    pub fn new_dummy(number: usize) -> PciBus {
        let mut bus = PciBus::new(number);

        bus.add_device(PciDevice::new_dummy(0)).unwrap();

        bus
    }

    /// Return the number of this device.
    pub fn get_number(&self) -> usize {
        self.number
    }

    /// Add a new bus to the current bus.
    /// * `bus` - The bus that will be wrapped in an Arc-Mutex struct and added.
    pub fn add_bus(&mut self, bus: PciBus) -> Result<()> {
        let bus_number = bus.get_number();

        if bus_number >= MAX_BUS_NUMBER {
            return Err(PciBusError::InvalidPciBusNumber(bus_number));
        }

        if self.buses.contains_key(&bus_number) {
            return Err(PciBusError::AlreadyInUsePciBusSlot(bus_number));
        }

        self.buses.insert(bus_number, Arc::new(Mutex::new(bus)));
        Ok(())
    }

    /// Return a reference to the requested bus if it exists.
    /// * `bus` - The index of the bus connected on the current bus.
    pub fn get_bus(&self, bus: usize) -> Option<&Arc<Mutex<PciBus>>> {
        self.buses.get(&bus)
    }

    /// Return a mutable reference to the requested bus if it exists.
    /// * `bus` - The index of the bus connected on the current bus.
    pub fn get_mut_bus(&mut self, bus: usize) -> Option<&mut Arc<Mutex<PciBus>>> {
        self.buses.get_mut(&bus)
    }

    /// Remove the bus from the current bus, returning the object, if it exists.
    /// * `bus` - The index of the bus connected on the current bus.
    pub fn remove_bus(&mut self, bus: usize) -> Option<Arc<Mutex<PciBus>>> {
        self.buses.remove(&bus)
    }

    /// Add a new device to the current bus.
    /// * `device` - The device that will be wrapped in an Arc-Mutex struct and added.
    pub fn add_device(&mut self, device: PciDevice) -> Result<()> {
        let device_number = device.get_number();

        if device_number >= MAX_DEVICE_NUMBER {
            return Err(PciBusError::InvalidPciDeviceNumber(device_number));
        }

        if self.devices.contains_key(&device_number) {
            return Err(PciBusError::AlreadyInUsePciDeviceSlot(device_number));
        }

        self.devices
            .insert(device_number, Arc::new(Mutex::new(device)));
        Ok(())
    }

    /// Return a reference to the requested device if it exists.
    /// * `device` - The index of the device connected on the current bus.
    pub fn get_device(&self, device: usize) -> Option<&Arc<Mutex<PciDevice>>> {
        self.devices.get(&device)
    }

    /// Return a mutable reference to the requested device if it exists.
    /// * `device` - The index of the device connected on the current bus.
    pub fn get_mut_device(&mut self, device: usize) -> Option<&mut Arc<Mutex<PciDevice>>> {
        self.devices.get_mut(&device)
    }

    /// Remove the device from the current bus, returning the object, if it exists.
    /// * `device` - The index of the device connected on the current bus.
    pub fn remove_device(&mut self, device: usize) -> Option<Arc<Mutex<PciDevice>>> {
        self.devices.remove(&device)
    }

    /// Get a register from the configuration header space of a function of the device.
    /// Check if the device is on this bus or maybe on other busses connected.
    /// * `bus` - The index of the bus.
    /// * `device` - The index of the device of the bus.
    /// * `function` - The index of the function of the device.
    /// * `register` - The index of the register within configuration header space.
    pub fn read_configuration_register(
        &self,
        bus: usize,
        device: usize,
        function: usize,
        register: usize,
    ) -> Option<u32> {
        // Check if the message is for a device on this bus or check the other buses.
        if bus == self.number {
            return match self.get_device(device) {
                Some(device) => device
                    .lock()
                    .unwrap()
                    .read_configuration_register(function, register),
                _ => None,
            };
        }

        if let Some(bridge) = self.get_bus(bus) {
            bridge
                .lock()
                .unwrap()
                .read_configuration_register(bus, device, function, register)
        } else {
            None
        }
    }

    /// Set a register in the configuration header space of a function of the device.
    /// Check if the device is on this bus or maybe on other busses connected.
    /// * `bus` - The index of the bus.
    /// * `device` - The index of the device of the bus.
    /// * `function` - The index of the function of the device.
    /// * `register` - The index of the register within configuration header space.
    /// * `offset` - The offset within the register.
    /// * `data` - The actual bytes of data.
    pub fn write_configuration_register(
        &mut self,
        bus: usize,
        device: usize,
        function: usize,
        register: usize,
        offset: usize,
        data: &[u8],
    ) {
        // Check if the message is for a device on this bus or check the other buses.
        if bus == self.number {
            if let Some(device) = self.get_device(device) {
                device
                    .lock()
                    .unwrap()
                    .write_configuration_register(function, register, offset, data);
            }

            return;
        }

        if let Some(bridge) = self.get_bus(bus) {
            bridge
                .lock()
                .unwrap()
                .write_configuration_register(bus, device, function, register, offset, data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bus_bus_add_get_remove() {
        let mut main_bus = PciBus::new(0);

        for bus in 0..MAX_BUS_NUMBER {
            assert!(main_bus.add_bus(PciBus::new_dummy(bus)).is_ok());
            assert!(main_bus.get_bus(bus).is_some());
        }

        assert!(main_bus.add_bus(PciBus::new_dummy(MAX_BUS_NUMBER)).is_err());

        for bus in 0..MAX_BUS_NUMBER {
            main_bus.remove_bus(bus);
            assert!(main_bus.get_bus(bus).is_none());
        }
    }

    #[test]
    fn bus_device_add_get_remove() {
        let mut bus = PciBus::new(0);

        for device in 0..MAX_DEVICE_NUMBER {
            assert!(bus.add_device(PciDevice::new_dummy(device)).is_ok());
            assert!(bus.get_device(device).is_some());
        }

        assert!(bus
            .add_device(PciDevice::new_dummy(MAX_DEVICE_NUMBER))
            .is_err());

        for device in 0..MAX_DEVICE_NUMBER {
            bus.remove_device(device);
            assert!(bus.get_device(device).is_none());
        }
    }
}
