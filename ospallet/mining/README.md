# Mining Module

## Terminology

- Session: Each session contains a fixed number of blocks(currently 43200).
- SessionIndex: It begins with 0 and grows 1 after each 43200 blocks.

## Mining logic

One session is a mining period, while the max of the total mint token will be 1% of the total issuance. The mint token 
can be claimed after the session, and each miners' mining power will decide how much token they can claim.

For example, total issuance is 10,000, and total mining power is 100, while Alice's mining power is 10. So Alice can mine 
10 / 100 * 10000 * 0.01 = 10 in this session.

The mining power is decided by the fee the council charged. For example, the bounty fund total 1000 OSN and 50 will be charged by the council. 
The total mining power for this bounty will be 50 * OSN_currency_ration. Each currency has a currency ratio, and currency ratio of OSN is 1.
So 50 will be the total mining power for the bounty. Funder and hunter will get 90% and 10% of the mining power, so funder get 45 and hunter get 5.

## Interfaces

- `claim(session_index: SessionIndex)`: Claim the native token based on the caller's mining power for the target session.
- `add_mining_power(target: &T::AccountId, power: MiningPower)`: Called by other modules to add mining power for one target account.
- `add_session_total_mining_power(power: MiningPower)`: Called by other modules to add the session total mining power.
