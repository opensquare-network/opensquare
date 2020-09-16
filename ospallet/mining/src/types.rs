pub type SessionIndex = u32;

pub type MiningPower = u128;

pub trait MiningPowerBuilder<AccountId> {
    fn add_mining_power(target: &AccountId, power: MiningPower);

    fn add_session_total_mining_power(power: MiningPower);
}
