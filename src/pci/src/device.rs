// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::constants::MAX_FUNCTION_NUMBER;
use crate::PciFunction;
use std::option::Option;

/// Each Device must implement Function 0 and may contain a collection up to 8 Functions.
/// A Device might implement Functions 0, 2 and 7 and there is no need to be sequentially.
#[derive(Clone)]
pub struct PciDevice {
    functions: Vec<PciFunction>,
}

impl PciDevice {
    /// Create a PCI device with MAX_FUNCTION_NUMBER empty functions.
    pub fn empty() -> Self {
        PciDevice {
            functions: vec![PciFunction::empty(); MAX_FUNCTION_NUMBER],
        }
    }

    /// Return a reference to a function of the PciDevice if it exists at the given index.
    fn get_function(&self, function_index: usize) -> Option<&PciFunction> {
        self.functions.get(function_index)
    }

    /// Return a mutable reference to a function of the PciDevice if it exists at the given index.
    fn get_mut_function(&mut self, function_index: usize) -> Option<&mut PciFunction> {
        self.functions.get_mut(function_index)
    }

    /// Get a register from the configuration header space of a function of the device.
    /// * `function_index` - The index of the function of the device.
    /// * `register_index` - The index of the register within configuration header space.
    fn read_configuration_register(
        &self,
        function_index: usize,
        register_index: usize,
    ) -> Option<u32> {
        match self.get_function(function_index) {
            Some(function) => function.read_configuration_dword(register_index),
            None => None,
        }
    }

    /// Set a register in the configuration header space of a function of the device.
    /// * `function_index` - The index of the function of the device.
    /// * `register_index` - The index of the register within configuration header space.
    /// * `offset` - The offset within the register.
    /// * `data` - The actual bytes of data.
    fn write_configuration_register(
        &mut self,
        function_index: usize,
        register_index: usize,
        offset: usize,
        data: &[u8],
    ) {
        // Make sure to be protected against overflow.
        if offset + data.len() > 4 {
            return;
        }

        if let Some(function) = self.get_mut_function(function_index) {
            for index in 0..data.len() {
                function.write_configuration_byte(register_index, offset + index, data[index])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate utils;

    /// Test writing an u32 as a Little Endian byte array and reading and assembling it.
    #[test]
    fn device_configuration_read_write() {
        let value = utils::rand::xor_rng_u32();
        let mut device = PciDevice::empty();

        device.write_configuration_register(0, 1, 0, &value.to_le_bytes());
        assert_eq!(device.read_configuration_register(0, 1), Some(value));
    }

    #[test]
    fn device_get_function() {
        let device = PciDevice::empty();

        assert_eq!(device.functions.len(), MAX_FUNCTION_NUMBER);
        assert!(device.get_function(0).is_some());
        assert!(device.get_function(MAX_FUNCTION_NUMBER).is_none());
    }

    /// Test writing an u32 as a Little Endian byte array and reading it from function object.
    #[test]
    fn function_configuration_read_write() {
        let value = utils::rand::xor_rng_u32();
        let mut device = PciDevice::empty();

        device.write_configuration_register(0, 1, 0, &value.to_le_bytes());
        assert_eq!(
            device.get_function(0).unwrap().read_configuration_dword(1),
            Some(value)
        );
    }
}
