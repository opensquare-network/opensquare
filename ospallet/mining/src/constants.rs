/// One session stays about 30 days.
/// OpenSquare issue new tokens at the end of each session.
/// If there are zero mining powers in this session, the issuance of this session will be zero.
pub const DEFAULT_BLOCKS_PER_SESSION: u32 = 432000;

/// One Era equal to 10 sessions.
/// Total issuance in each era will not exceed 10% of the total issuance before the era.
pub const DEFAULT_SESSIONS_PER_ERA: u32 = 10;
