CDA
---

Simple Continuous Double Auction engine for simulating
a real CDA.
The input is from the redis queue `orders`.

Orders are formatted `amount:price:user_id:order_id`
where `amount` is positive for a _ask_, and negative for a _bid_.

Filled order are displayed in the queues `filled_ask` and `filled_bid`.

The redis keys `bid` and `ask` contains the minimal and maximal prices
of the order book.

Install
--------------

First, you need rustc (the rust compiler) and
cargo (the package manager for rust).

I recommend installing rustc through the rustc website https://www.rust-lang.org/en-US/install.html with:
```
curl https://sh.rustup.rs -sSf | sh
```

Clone this git repository and run cargo:
```
clone https://github.com/jeremycochoy/cda.git
cd cda
cargo run --release
```
