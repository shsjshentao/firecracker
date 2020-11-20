// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::PciFunction;
use std::collections::HashMap;
use std::option::Option;
use std::sync::{Arc, Mutex};
use utils::byte_order::{read_le_u16, read_le_u32};

/// A Device can have implemented up to 8 Functions (not necessarily sequentially).
pub const MAX_FUNCTION_NUMBER: usize = 8;

/// Errors for the Pci Bus.
#[derive(Debug)]
pub enum PciDeviceError {
    /// Invalid PCI function number provided.
    InvalidPciFunctionNumber(usize),
    /// Valid PCI function number but already used.
    AlreadyInUsePciFunctionSlot(usize),
}

pub type Result<T> = std::result::Result<T, PciDeviceError>;

/// Each Device must implement Function 0 and may contain a collection up to 8 Functions.
/// A Device might implement Functions 0, 2 and 7 and there is no need to be sequentially.
pub struct PciDevice {
    /// The number of the device within the bus.
    number: usize,

    /// The functions registered within this device.
    functions: HashMap<usize, Arc<Mutex<PciFunction>>>,
}

impl PciDevice {
    /// Create an empty PCI device.
    pub fn new(number: usize) -> PciDevice {
        PciDevice {
            number,
            functions: HashMap::new(),
        }
    }

    /// Create a dummy PCI device which contains a dummy host bridge as function 0.
    /// - `number` - the number of the device.
    pub fn new_dummy_host_bridge(number: usize) -> PciDevice {
        let mut device = PciDevice::new(number);

        device
            .add_function(PciFunction::new_dummy_host_bridge(0))
            .unwrap();

        device
    }

    /// Return the number of this device.
    pub fn get_number(&self) -> usize {
        self.number
    }

    /// Add a new function to this device.
    /// * `function` - The function that will be wrapped in an Arc-Mutex struct and added.
    pub fn add_function(&mut self, function: PciFunction) -> Result<()> {
        let function_number = function.get_number();

        if function_number >= MAX_FUNCTION_NUMBER {
            return Err(PciDeviceError::InvalidPciFunctionNumber(function_number));
        }

        if self.functions.contains_key(&function_number) {
            return Err(PciDeviceError::AlreadyInUsePciFunctionSlot(function_number));
        }

        self.functions
            .insert(function_number, Arc::new(Mutex::new(function)));

        Ok(())
    }

    /// Return a reference to the requested function if it exists.
    /// * `function` - The index of the function of the device.
    pub fn get_function(&self, function: usize) -> Option<&Arc<Mutex<PciFunction>>> {
        self.functions.get(&function)
    }

    /// Return a mutable reference to the requested function if it exists.
    /// * `function` - The index of the function of the device.
    pub fn get_mut_function(&mut self, function: usize) -> Option<&mut Arc<Mutex<PciFunction>>> {
        self.functions.get_mut(&function)
    }

    /// Remove the function from this device, returning the function object, if it exists.
    /// * `function` - The index of the function of the device.
    pub fn remove_function(&mut self, function: usize) -> Option<Arc<Mutex<PciFunction>>> {
        self.functions.remove(&function)
    }

    /// Get a register from the configuration header space of a function of the device.
    /// * `function` - The index of the function of the device.
    /// * `register` - The index of the register within configuration header space.
    pub fn read_configuration_register(&self, function: usize, register: usize) -> Option<u32> {
        if let Some(function) = self.get_function(function) {
            function.lock().unwrap().read_configuration_dword(register)
        } else {
            None
        }
    }

    /// Set a register in the configuration header space of a function of the device.
    /// * `function` - The index of the function of the device.
    /// * `register` - The index of the register within configuration header space.
    /// * `offset` - The offset within the register.
    /// * `data` - The actual bytes of data.
    pub fn write_configuration_register(
        &mut self,
        function: usize,
        register: usize,
        offset: usize,
        data: &[u8],
    ) {
        // Make sure to be protected against overflow.
        if offset + data.len() > 4 {
            return;
        }

        if let Some(function) = self.get_mut_function(function) {
            let mut function = function.lock().unwrap();

            match data.len() {
                1 => function.write_configuration_byte(register, offset, data[0]),
                2 => function.write_configuration_word(register, offset, read_le_u16(data)),
                4 => function.write_configuration_dword(register, read_le_u32(data)),
                _ => (),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::byte_order::read_le_u32;
    use utils::rand::xor_rng_u32;

    fn get_function(function: usize) -> PciFunction {
        PciFunction::new_dummy_host_bridge(function)
    }

    #[test]
    fn device_function_add_get_remove() {
        let mut device = PciDevice::new(0);

        for function in 0..MAX_FUNCTION_NUMBER {
            assert!(device.add_function(get_function(function)).is_ok());
            assert!(device.get_function(function).is_some());
        }

        assert!(device
            .add_function(PciFunction::new_dummy_host_bridge(MAX_FUNCTION_NUMBER))
            .is_err());

        for function in 0..MAX_FUNCTION_NUMBER {
            device.remove_function(function);
            assert!(device.get_function(function).is_none());
        }
    }

    #[test]
    fn device_configuration_normal_read() {
        let mut device = PciDevice::new(0);
        let data = [xor_rng_u32() as u8; 4];

        device.add_function(get_function(0)).unwrap();
        device.write_configuration_register(0, 1, 0, &data);

        assert_eq!(
            device.read_configuration_register(0, 1),
            Some(read_le_u32(&data))
        );
    }

    #[test]
    fn device_configuration_function_read() {
        let mut device = PciDevice::new(0);
        let data = [xor_rng_u32() as u8; 4];

        device.add_function(get_function(0)).unwrap();
        device.write_configuration_register(0, 1, 0, &data);

        let function = device.get_function(0).unwrap();
        assert_eq!(
            function.lock().unwrap().read_configuration_dword(1),
            Some(read_le_u32(&data))
        );
    }
}
