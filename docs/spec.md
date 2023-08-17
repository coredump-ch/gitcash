# GitCash Specification

GitCash is a Git based payment system primarily targeted at hackerspaces and
similar groups. It is being developed by [Coredump](https://www.coredump.ch/) in
collaboration with other Swiss hackerspaces.

## Account types

There are multiple account types:

- **User**: Can both send money (pay for goods) and receive money (deposit
  money into account, person-to-person payments)
- **Point of Payment**: Can only receive money
- **Source**: Special type of account that can be used to deposit money into
  the system

## Storage format

Transactions are stored as TOML snippets in Git commit messages. Every commit
contains 1 transaction (or none at all).

If a commit message starts with a special prefix, it will be parsed:

- `Transaction: <description>`: Create a transaction
- `Revert: <description>`: Revert a transaction

The `<description>` is a human-readable desription of the transaction.

The data of the transaction (in TOML format) is inserted between two markers
(`---` on a dedicated line). This allows adding more metadata before or after
the data section, which won't be parsed.

### Example commits

Deposit some cash:

```
commit 7a3a5654271661620480d8f9275cbf818a69c7ac
Author: Fridge Laptop <fridge@coredump.ch>
Date:   Thu Jan 23 11:34:42 2020 +0100

Transaction: Cash Deposit for Danilo (20.00 CHF)

---
from = "source:cash"
to = "user:danilo"
amount = 2000
---
```

Buy a drink:

```
commit 7a3a5654271661620480d8f9275cbf818a69c7ac
Author: Fridge Laptop <fridge@coredump.ch>
Date:   Thu Jan 23 11:34:42 2020 +0100

Transaction: User danilo buys "Vivi Kola 33cl" (2.50 CHF)

---
from = "user:danilo"
to = "pop:fridge"
amount = 250
description = "Vivi Kola 33cl"

[meta]
class = "softdrink"
ean = 7610867035003
---
```

Person-to-person payment:

```
commit 7a3a5654271661620480d8f9275cbf818a69c7ac
Author: Fridge Laptop <fridge@coredump.ch>
Date:   Thu Jan 23 11:34:42 2020 +0100

Transaction: User danilo pays 24.80 CHF to user rnestler

---
from = "user:danilo"
to = "user:rnestler"
amount = 2480
description = "Lunch"
---
```

### Reverting transactions

To revert a transaction, simply repeat the commit message, but use the prefix
`Revert: ` instead of `Transaction: `.

## Configuration

There's a global configuration file (`gitcash.toml`):

```toml
name = "Coredump"

[currency]
code = "CHF"
divisor = 100
```

The amounts are always specified as integers. The "divisor" in the
configuration determines, how the value is converted into the currency (e.g.
the amount `3450` with code `CHF` and divisor `100` equals `34.50 CHF`).
