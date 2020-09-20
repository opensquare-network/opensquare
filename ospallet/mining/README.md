# Mining Module

## Terminology

- Session: Each session contains a fixed number of blocks(currently 43200).
- SessionIndex: It begins with 0 and grows 1 after each 43200 blocks.

## Mining logic

One session is a mining period, while the max of the total mint token will be 1% of the total issuance. The mint token 
can be claimed after the session, and each miners' mining power will decide how much token they can claim.

For example, total issuance is 10,000, and total mining power is 100, while Alice's mining power is 10. So Alice can mine 
10 / 100 * 10000 * 0.01 = 10 in this session.

## Interfaces

- `claim(session_index: SessionIndex)`: Claim the native token based on the caller's mining power for the target session.
- `add_mining_power(target: &T::AccountId, power: MiningPower)`: Called by other modules to add mining power for one target account.
- `add_session_total_mining_power(power: MiningPower)`: Called by other modules to add the session total mining power.
