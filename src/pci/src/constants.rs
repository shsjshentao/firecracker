// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

/// There are up to 256 Bus numbers that can be assigned.
const MAX_BUS_NUMBER: usize = 256;

/// There are up to 32 Device attachments on a single PCI Bus.
const MAX_DEVICE_NUMBER: usize = 32;

/// A Device can have implemented up to 8 Functions (not necessarily sequentially).
const MAX_FUNCTION_NUMBER: usize = 8;

/// The PCI Configuration Header Space has a length of 64 bytes, so 16 dwords.
const CONFIGURATION_HEADER_SIZE: usize = 16;

/// The PCI optional registers (device specific) has a length of 192 bytes, so 48 dwords.
const DEVICE_SPECIFIC_REGISTERS: usize = 48;

/// The PCIe Extended Configuration Registers Space has a length 3840 bytes, so 960 dwords.
const EXTENDED_CONFIGURATION_REGISTERS: usize = 960;
