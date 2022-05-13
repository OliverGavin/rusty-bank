
# Rusty Bank!
Rusty Bank is a ficticious banking solution for handling transactions from a CSV.
The command line tool outputs the state of each client's account as a CSV.
It is responsible for updating client accounts and handling disputes & chargebacks.

## Development
### Building and Executing
Build as normal: `cargo build`

Run with a single argument and handle stdout: `cargo run -- transactions.csv > accounts.csv`

Format and lint: `cargo fmt && cargo clippy`

#### Logging
Some very basic logging is configured with the WARN level by default.
Logging levels can be set with environment variables. For example `RUST_LOG=debug`.

### Documentation
Just run `cargo doc --open`.


-------------


## Rough work

### Implementation notes/ideas
Input:
- Schema: type (str), client (u16), tx (u32), and amount (decimal, up to 4 places)
- Client and transaction IDs are not guaranteed to be ordered, but transactions occur chronologically.

Output:
- Schema: client (u16), available (decimal, up to 4 places), held (decimal, up to 4 places), total (decimal, up to 4 places), locked (bool)
- `available == total - held` (The total funds that are available for trading, staking, withdrawal, etc)
- `held == total - available` (The total funds that are held for dispute)
- `total == available + held` (The total funds that are available or held)
- An account is locked if a charge back occurs
- Spacing, displaying decimals, row ordering does not matter.

Transaction Types:
- Deposit: increase available and total funds of client account
- Withdrawal: decrease available and total funds of client account
	- should fail (i.e. no change) without sufficient funds
- Dispute: claim that transaction was erroneous, could be reversed
	- funds should be held; decrease available, increase held, total unchanged
	- should fail (i.e. ignore) if transaction ID does not exist
- Resolve: resolution of dispute
	- increase available, decrease held, total unchanged
	- should fail (i.e. ignore) if transaction ID does not exist or is not under dispute
- Chargeback: client reversing transaction, funds held have been withdrawn
	- decrease held and total, available unchanged
	- account is frozen/locked
	- should fail (i.e. ignore) if transaction ID does not exist or is not under dispute

Concern:
- Need to make sure that there are no precision errors on amounts

It is probably worth logging the quiet failures due to errors on our partner's side.


### Implementation checklist
- [X] Scaffolding
- [ ] Argument parsing and validation
- [ ] Define integ tests based on requirements
- [ ] CSV reading and writing
- [ ] Representations for client and transaction IDs
- [ ] Correct precision handling
- [ ] Representation for each transaction type along with serdes
- [ ] Support transaction type: deposit
- [ ] Support transaction type: withdrawal
- [ ] Support transaction type: dispute
- [ ] Support transaction type: resolve
- [ ] Support transaction type: chargeback


## Design rationale and approach
The requirements offer a great degree of creative freedom for sovling this problem.
This comes with an equal opportunity to make things complicated.
In order to keep the apporach sane and given that I lack insight
  into how this could be realistically extended in the future,
  I am opting to go with the most simple approach where possible.

- Favour readability and testability
- Avoid over engineering the problem in an attempt to generalize
  - focus on solving the problem while spearating concerns
  - refactoring can be done for any future requirement for a different input source/format
    - let's not create abstractions for a problem space I don't understand yet
- Avoid hiding too much of my dependencies in my APIs
  - this is a binary, not a library
  - for simplicity keep existing types (errors, etc) rather than trying to hide behind my own
- Make the code friendly for future changes
  - docs/tests
  - make the main execution flow simple so it can be changed (i.e. if it needs to become async/multi-threaded)
  - push as much logic as possible to respective modules so that changes don't span further than necessary
