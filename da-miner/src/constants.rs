use zg_encoder::constants::{BLOB_COL_N, BLOB_UNIT};

pub const NUM_SUBLINES: usize = 32;
pub const LINE_BYTES: usize = BLOB_COL_N * BLOB_UNIT;
pub const SUBLINE_BYTES: usize = LINE_BYTES / NUM_SUBLINES;
