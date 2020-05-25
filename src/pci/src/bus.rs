// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::constants::MAX_DEVICE_NUMBER;
use crate::device::{PciDevice, PciRootComplex};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Errors for the Pci Bus.
#[derive(Debug)]
pub enum PciBusError {
    /// Could not find an available device slot on the PCI bus.
    NoPciDeviceSlotAvailable,
}

pub type Result<T> = std::result::Result<T, PciBusError>;

/// Each Bus must be assigned a unique bus number.
/// The initial Bus Number, Bus 0, is typically assigned to the Root Complex.
pub struct PciBus {
    devices: HashMap<u32, Arc<Mutex<dyn PciDevice>>>,
}

impl PciBus {
    pub fn new(pci_root_complex: PciRootComplex) -> Self {
        let mut devices: HashMap<u32, Arc<Mutex<dyn PciDevice>>> = HashMap::new();

        devices.insert(0, Arc::new(Mutex::new(pci_root_complex)));

        PciBus { devices }
    }

    pub fn add_device(&mut self, device: Arc<Mutex<dyn PciDevice>>) -> Result<()> {
        for device_no in 1..MAX_DEVICE_NUMBER {
            if !self.devices.contains_key(&device_no) {
                self.devices.insert(device_no, device);
                return Ok(());
            }
        }

        Err(PciBusError::NoPciDeviceSlotAvailable)
    }
}
