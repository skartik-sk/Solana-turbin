// Constants
const SECONDS_IN_AN_HOUR: i64 = 3600;
const SECONDS_IN_A_MINUTE: i64 = 60;
const SECONDS_IN_A_DAY: i64 = 86400;
const MARKET_OPEN_TIME: i64 = 9 * SECONDS_IN_AN_HOUR ; // 9:00 UTC == 9:00 am
const MARKET_CLOSE_TIME: i64 = 17 * SECONDS_IN_AN_HOUR; // 17:00 UTC == 5:00 pm
pub const REWARD_IN_LAMPORTS: u64 = 10000000; // 0.001 SOL


pub fn is_transferring_allowed(unix_timestamp: i64) -> bool {
    let seconds_since_midnight = unix_timestamp % SECONDS_IN_A_DAY;
    let weekday = (unix_timestamp / SECONDS_IN_A_DAY + 4) % 7;
    // Check if it's a weekday (Monday = 0, ..., Friday = 4)
    if weekday >= 5 {
        return false;
    }
    // Check if current time is within market hours
    seconds_since_midnight >= MARKET_OPEN_TIME && seconds_since_midnight < MARKET_CLOSE_TIME
}