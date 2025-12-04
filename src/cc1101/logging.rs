use core::fmt::Debug;
use core::fmt::Write;

use flipperzero::debug;
use heapless::String;

/// Heapless string size used for register dumps.
pub const REGISTER_DUMP_BUFFER_SIZE: usize = 256;
