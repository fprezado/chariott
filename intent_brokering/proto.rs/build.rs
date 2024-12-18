// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{error::Error, path::Path};
use tonic_build::configure;

fn main() -> Result<(), Box<dyn Error>> {
    compile_with_common("../proto/intent_brokering/runtime/v1/runtime.proto")?;
    compile_with_common("../proto/intent_brokering/provider/v1/provider.proto")?;
    compile_with_common("../proto/intent_brokering/streaming/v1/streaming.proto")?;

    Ok(())
}

fn compile_with_common(path: &str) -> Result<(), Box<dyn Error>> {
    configure().compile_protos(&[Path::new(path)], &[Path::new("../proto/")])?;

    Ok(())
}
