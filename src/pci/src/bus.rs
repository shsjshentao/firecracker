// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::constants::{MAX_BUS_NUMBER, MAX_DEVICE_NUMBER};
use crate::device::PciDevice;

/// Errors for the Pci Bus.
#[derive(Debug)]
pub enum PciBusError {
    /// Could not find an available device slot on the PCI bus.
    NoPciDeviceSlotAvailable,
}

pub type Result<T> = std::result::Result<T, PciBusError>;

/// Each Bus must be assigned a unique bus number.
/// The initial Bus Number, Bus 0, is typically assigned to the Root Complex.
#[derive(Clone)]
pub struct PciBus {
    number: u32,

    buses: Vec<PciBus>,
    devices: Vec<PciDevice>,
}

impl PciBus {
    pub fn new(number: u32) -> Self {
        PciBus {
            number,
            buses: Vec::with_capacity(MAX_BUS_NUMBER),
            devices: Vec::with_capacity(MAX_DEVICE_NUMBER),
        }
    }

    /// Return the bus number.
    pub fn get_number(&self) -> u32 {
        self.number
    }

    /// Return a reference to the array of buses.
    pub fn get_buses(&self) -> &Vec<PciBus> {
        &self.buses
    }

    /// Return a mutable reference to the array of buses.
    pub fn get_mut_buses(&mut self) -> &mut Vec<PciBus> {
        &mut self.buses
    }

    /// Return a reference to the array of devices.
    pub fn get_devices(&self) -> &Vec<PciDevice> {
        &self.devices
    }

    /// Return a mutable reference to the array of devices.
    pub fn get_mut_devices(&mut self) -> &mut Vec<PciDevice> {
        &mut self.devices
    }

    pub fn add_device(&mut self, device: PciDevice) -> Result<()> {
        if self.devices.len() == MAX_DEVICE_NUMBER {
            return Err(PciBusError::NoPciDeviceSlotAvailable);
        }

        self.devices.push(device);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the functions that works with the devices of the bus.
    #[test]
    fn bus_devices() {
        let mut bus = PciBus::new(0);
        let device = PciDevice::empty();

        assert_eq!(bus.get_buses().len(), 0);
        assert_eq!(bus.get_devices().len(), 0);

        for _ in 0..MAX_DEVICE_NUMBER {
            assert!(bus.add_device(device.clone()).is_ok());
        }
        assert!(bus.add_device(device.clone()).is_err());
    }
}
