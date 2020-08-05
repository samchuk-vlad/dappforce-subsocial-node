pub mod currency {
    use node_primitives::Balance;

    pub const DOTS: Balance = 10_000_000_000;
    pub const DOLLARS: Balance = DOTS / 100;       // 100_000_000
    pub const CENTS: Balance = DOLLARS / 100;      // 1_000_000
    pub const MILLICENTS: Balance = CENTS / 1_000; // 1_000
}

pub mod time {
    use node_primitives::{Moment, BlockNumber};

    pub const MILLISECS_PER_BLOCK: Moment = 4000;
    pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

    // These time units are defined in number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;
}