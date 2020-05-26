// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::constants::MAX_FUNCTION_NUMBER;
use crate::PciFunction;
use devices::BusDevice;
use std::option::Option;

/// Each Device must implement Function 0 and may contain a collection up to 8 Functions.
/// A Device might implement Functions 0, 2 and 7 and there is no need to be sequentially.
pub trait PciDevice: BusDevice {
    /// Return a mutable reference to a function of the current PciDevice.
    /// If the index is out of bounds return None.
    fn get_function(&self, function_index: usize) -> Option<&PciFunction>;
    fn get_mut_function(&mut self, function_index: usize) -> Option<&mut PciFunction>;

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

/// Emulate the PCI Root Complex node of the PCIe topology.
/// This component generates transaction requests on behalf of the processor.
/// This hardware component may contain different interfaces (CPU, DRAM) and chips.
pub struct PciRootComplex {
    functions: Vec<PciFunction>,
}

impl PciRootComplex {
    pub fn new() -> Self {
        let functions: Vec<PciFunction> = vec![PciFunction::empty(); MAX_FUNCTION_NUMBER];

        PciRootComplex { functions }
    }
}

impl BusDevice for PciRootComplex {}

impl PciDevice for PciRootComplex {
    fn get_function(&self, function_index: usize) -> Option<&PciFunction> {
        self.functions.get(function_index)
    }

    fn get_mut_function(&mut self, function_index: usize) -> Option<&mut PciFunction> {
        self.functions.get_mut(function_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate utils;

    #[test]
    /// Test writing a u32 as a Little Endian byte array and reading and assembling it.
    fn register_read_write() {
        let value = utils::rand::xor_rng_u32();
        let mut pci_root_complex = PciRootComplex::new();

        pci_root_complex.write_configuration_register(0, 1, 0, &value.to_le_bytes());
        assert_eq!(
            pci_root_complex.read_configuration_register(0, 1),
            Some(value)
        );
    }
}
