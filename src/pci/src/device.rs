// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use constants::MAX_FUNCTION_NUMBER;
use function::PciFunction;

/// Each Device must implement Function 0 and may contain a collection up to 8 Functions.
/// A Device might implement Functions 0, 2 and 7 and there is no need to be sequentially.
pub struct PciDevice {
    functions: [PciFunction; MAX_FUNCTION_NUMBER],
}
