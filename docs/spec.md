# GitCash Specification

GitCash is a Git based payment system primarily targeted at hackerspaces and
similar groups. It is being developed by [Coredump](https://www.coredump.ch/) in
collaboration with other Swiss hackerspaces.

## Account types

There are multiple account types:

- **User** (prefix `user:`): Can both send money (pay for goods) and receive
  money (deposit money into account, person-to-person payments)
- **Point of Sale** (prefix `pos:`): Can only receive money
- **Source** (prefix `source:`): Special type of account that can be used to
  deposit money into the system

Accounts come into existence by usage, but you can also explicitly create a new
account by transferring an amount of 0 to that account.

## Storage format

Transactions are stored as TOML snippets in Git commit messages. Every commit
contains 1 transaction (or none at all).

If a commit message starts with a special prefix, it will be parsed:

- `Transaction: <description>`: Create a transaction
- `Revert: <description>`: Revert a transaction

The `<description>` is a human-readable description of the transaction.

The data of the transaction (in TOML format) is inserted between two markers
(`---` on a dedicated line). This allows adding more metadata before or after
the data section, which won't be parsed.

### TOML keys

The TOML section can use the following keys:

- `from` (required): The source account
- `to` (required): The destination account
- `amount` (required): The amount (must fit in i32, i.e. between `-2147483648`
  and `2147483647` inclusive)
- `description` (optional): A free-form string to describe the transaction
- `meta` (optional): A table containing meta information

The following meta keys may be used, all of them are optional:

- `class`: The product class as a string, e.g. "softdrink".
- `ean`: The EAN code as an unsigned integer.

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
to = "pos:fridge"
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
