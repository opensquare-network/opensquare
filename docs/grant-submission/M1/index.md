# Submission for OpenSquare Network M1

## Name of the grant project

OpenSquare Network

## Link to the open-source code

https://github.com/opensquare-network/opensquare/tree/w3f-m1-0

## License

https://github.com/opensquare-network/opensquare/tree/w3f-m1-0#license

## Documentation

Currently we implement a basic workflow which support `OneFunderOneHunter` bounty collaboration. 
There are mainly 3 pallets about our business.

- [bounties](https://github.com/opensquare-network/opensquare/tree/w3f-m1-0/ospallet/bounties) pallet manages the lifecycle of a bounty.
- [reputation](https://github.com/opensquare-network/opensquare/tree/w3f-m1-0/ospallet/reputation) pallet manages each role's reputation.
- [mining](https://github.com/opensquare-network/opensquare/tree/w3f-m1-0/ospallet/mining) pallet manages users' session related mining power and handle the native token mint logic.

Functions in reputation and mining pallets will be called by some calls in bounties pallet. For example: 
- When [resolve_bounty_and_remark](https://github.com/opensquare-network/opensquare/blob/w3f-m1-0/ospallet/bounties/src/lib.rs#L201) is called, [add_behavior_score_by_behavior](https://github.com/opensquare-network/opensquare/blob/w3f-m1-0/ospallet/reputation/src/lib.rs#L49), 
[add_mining_power](https://github.com/opensquare-network/opensquare/blob/w3f-m1-0/ospallet/mining/src/lib.rs#L113) will be called to add reputation nd mining power to hunter.
- When [remark_bounty_funder](https://github.com/opensquare-network/opensquare/blob/w3f-m1-0/ospallet/bounties/src/lib.rs#L248) is called, [add_behavior_score_by_behavior](https://github.com/opensquare-network/opensquare/blob/w3f-m1-0/ospallet/reputation/src/lib.rs#L49) is called to add funder's reputation.

## Testing Guide

[bounty-workflow.md](./bounty-workflow.md) give a basic explanation about the bounty workflow.

Please refer to this [google doc](https://docs.google.com/document/d/1YfvERA_EilOEFTOd-tEivvWwn7DCk_Oj9cMror776mA/edit?usp=sharing) to check the business. 

## Additional Information

- We will have deeply collaboration with [candaq](http://candaq.com/) and [Patract labs](https://twitter.com/patractlabs).
- Some projects in Polkadot ecosystems express their interest for OpenSquare, like subsocial, Bifrost, Bandot, .etc.
And we may be the first funders on OpenSquare.

### Our future work

- The app and explorer of course.
- Implement more bounty related business, for example, OneFunderMultiHunter, MultiFunderMultiHunter, .etc.
- Make OpenSquare a platform for collaboration, not just crowdsourcing.
