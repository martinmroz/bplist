//
// Copyright 2020 bplist Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

mod de;
mod document;
mod error;

pub mod object;
pub use object::Object;

pub use de::{from_slice, Deserializer};
pub use error::{Error, Result};
