# Bounties Module

The bounties module provide management for bounties. Generally speaking, it facilitate the collaborations between
funders, hunters and the council.

## Basic workflow

1. Funder create a bounty and wait for the council's review.
2. The council may accept or reject the bounty. If accepted, the bounty can be applied by hunters.
3. Funder assign one hunter to the bounty.
4. Hunter do the work and submit it.
5. Funder resolve the bounty and give remark to the hunter. Fund will be sent to hunter wile some fee will be charged by the council.

In the process of 3-4, either the council and funder can close the bounty. Currently no fund or reputation loss for funder to close bounty, we may add ways to setting this in future features.

## Interfaces

### Funder calls
- `create_bounty`: Create a bounty and deposit the fund, and this bounty will be reviewed by the council.
- `assign_bounty`: Assign the bounty to one applicant.
- `close_bounty`: Close the bounty.
- `resolve_bounty_and_remark`: Resolve the bounty and the fund will be sent to the assigned hunter, while some fee will be charged by the council.

### Hunter calls

- `hunt_bounty`: Apply a accepted bounty.
- `submit_bounty`: Submit the work result for the assigned bounty.
- `cancel_bounty`: Cancel the application for the bounty.
- `resign_from_bounty`: Resign from a assigned bounty.
- `remark_bounty_funder`: Remark the bounty funder after the funder resolve the bounty and give the remark to hunter.

### Council calls

- `examine_bounty`: Give the review result for a bounty.
- `force_close_bounty`: Force close a bounty. The reasons may include outdated description, longtime no applicants.

## Reputation

Some collaborations will bring behavior score to user. For instance:

- Bounty resolved by funder will bring hunter reputation grow.
- Funders and Hunters' Reputation will be affected by each other's remark.
