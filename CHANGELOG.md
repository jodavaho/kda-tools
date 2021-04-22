# Unreleased (Main Branch)

# Version 0.6.0

- Now handling zero-occurance case for both equipment and metrics (e.g., p(K=0|loadout))

# Version 0.5.0

- Removed kda-* except compare
- Broke out lots of common code to use [kvc crate](https://crates.io/crates/kvc)

# Version 0.1.1

- Rename kda-corr -> kda-regress to better reflect it's true nature

## Knwon issues

- Some build warnings persist in this early phase
- kda-correlate is unstable
- Output needs a lot of work
