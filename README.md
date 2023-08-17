# GitCash: A Git based payment system

GitCash is a Git based payment system primarily targeted at hackerspaces and
similar groups. It is being developed by [Coredump](https://www.coredump.ch/) in
collaboration with other Swiss hackerspaces.

## High-Level Goals

- Money enters the system by putting cash into a cash box and crediting
  yourself that amount in the payment system. That credit can then be used to
  pay for goods.
- Decentralized storage format for transactions in a repository.
- Insecure but transparent: There is no authentication, but all transactions
  are transparent and public (if you have access to the repository).

## How it works

Transactions are stored as TOML snippets in Git commit messages. Every commit
contains 1 transaction (or none at all).

This has a few advantages:

- We an use a regular Git server (GitLab, Gitea or even just a repo reachable
  by SSH)
- The integrity is ensured through Git hashes
- The transactions are ordered in the Git DAG
- Transactions can be validated using a pre-push hook on the server
- Conflicts can be resoved by merging
- You could create transactions "by hand"
- Commits could be signed

Transactions in the repository can be created or analyzed by any node that has
access to the repository.

## Use cases for nodes

- **Fridge**: Pay for drinks at the fridge by scanning the barcode
- **Lasercutter**: Pay for lasercutter time by starting and stopping your
  session on a touchpanel
- **3D Printer**: Pay for your 3D prints by putting your print on an electronic
  scale and entering your name and the material on a touchpanel
- **Notification Monitor**: Whenever money is deduced from your account, you
  get a notification
- ...and many more!

Not all nodes might have the processing power to be a "full node" that can
parse the Git repository. For these use cases, a small server could accept new
transactions over a simple API and write them to the repository.

## Storage format / specification

See `docs/spec.md`.

## Demo repository

See <https://github.com/coredump-ch/gitcash-demo-repo>.

## Crates

- `libgitcash`: Library / SDK that can be used for processing a GitCash
  repository
- `gitcash`: A CLI client for GitCash

## License

Licensed under the AGPL version 3 or later. See `LICENSE.md` file.

    Copyright (C) 2023 Coredump Rapperswil-Jona

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as
    published by the Free Software Foundation, either version 3 of the
    License, or (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
