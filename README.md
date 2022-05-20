
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

### Tests
Run all unit and integration tests with `cargo test`.

Run just the unit tests with `cargo test --lib`.

Run just the integration tests with `cargo test --package rusty-bank --test integration_test`.

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


### Assumptions
- If there is an issue deserializing a transaction, ignore it and continue processing.
- A dispute followed by a chargeback may succeed with insufficient funds available resulting in a negative balance.
  - Chargebacks are not within Rusty Bank's control and chargebacks must be honoured.


### Implementation checklist
- [X] Scaffolding
- [X] Argument parsing and validation
- [X] Define integ tests based on requirements
- [X] Representations for client and transaction IDs
- [X] Correct precision handling
- [X] Representation for each transaction type along with serdes
- [X] CSV reading and writing
- [X] Basic transaction processor
- [ ] Improve transaction serde for better type safety and add validation
- [ ] Support transaction type: deposit
- [ ] Support transaction type: withdrawal
- [ ] Support transaction type: dispute
- [ ] Support transaction type: resolve
- [ ] Support transaction type: chargeback


### Transaction processing algorithm & data structures
Important constraint: transaction IDs are not guaranteed to be ordered, but transactions occur chronologically.
This means we need to process the transactions sequentially.

It also means that a deposit should occur before a dispute and a dispute should occur before a resolve/chargeback. A decent approach would be to process deposits and withrawals transaction by transaction but cache each deposit transaction in case a dispute should arise. Additionally, any valid dispute should also be cached and represented in one of two states; open or closed (a dispute may not be re-opened once closed).
Deposit transactions could be stored in a map. Dispute statuses could be stored in a map or in two sets using just the transaction ID. The balance of each account must also be maintained. If we are processing transactions sequentially then each account may be accessed at random so a map would be suitable.

The above is a stategy to process the transactions sequentially, storing only a minimum amount of information.
Of course, if there were millions of transactions for millions of customers we would consider a more optimal solution. This could involve first grouping transactions by client ID into their own files and then processing each customer file sequentially - this is also a decent approach to parallelize the solution.

If processing customer transactions individually is still not feasible and, for example, if there was a need to be able to dispute any previous transaction then we could explore some options that don't require all the data to be kept in memory. If there was no flexibility to add extra metadata such as transaction time, we could consider putting a limit on the number of records in each file and index them based on their minimum and maximum transaction IDs (there could be minimal overlap). With a suitable caching/paging strategy any previous deposit could be disputed without any significant memory overhead.


### Future ideas
- Consider benchmarking `csv-async` for larger files.
  - `rust-csv` does not support async reads/writes.
- Consider alternatives to `rust-csv` for parsing CSV files to serdes.
  - `rust-csv` does not support internally-tagged enums which limits serde functionality and thus our ability to parse `Transaction` records such that they best use the type system; ideally `amount` would not be optional but rather not exist for dispute, resolve and chargeback variants.
- Consider using `thiserror` over `anyhow` for library internals.
  - hiding library specific errors from other components is preferred.
  - but `anyhow::Error` requires `Send + Sync + 'static` which is a bit ugly if not required.
- Consider if `CsvTransactionReader` and `CsvAccountWriter` are abstractions worth keeping versus directly depending on `rust-csv` throughout.


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
