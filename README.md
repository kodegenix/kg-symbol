# kg-symbol

[![Latest Version](https://img.shields.io/crates/v/kg-symbol.svg)](https://crates.io/crates/kg-symbol)
[![Documentation](https://docs.rs/kg-symbol/badge.svg)](https://docs.rs/kg-symbol)
[![Build Status](https://travis-ci.org/Kodegenix/kg-symbol.svg?branch=master)](https://travis-ci.org/Kodegenix/kg-symbol)
[![Coverage Status](https://coveralls.io/repos/github/Kodegenix/kg-symbol/badge.svg?branch=master)](https://coveralls.io/github/Kodegenix/kg-symbol?branch=master)

Atomic strings in Rust.

This crate provides a `Symbol` type representing reference to an interned string. 
Since there can only exist one Symbol with a given name, symbols equality can be established simply from pointer comparison.

## Builds statuses for Rust channels

| stable            | beta              | nightly           |
|-------------------|-------------------|-------------------|
| [![Build1][3]][4] | [![Build2][2]][4] | [![Build3][1]][4] |

[1]: https://travis-matrix-badges.herokuapp.com/repos/kodegenix/kg-symbol/branches/master/1
[2]: https://travis-matrix-badges.herokuapp.com/repos/kodegenix/kg-symbol/branches/master/2
[3]: https://travis-matrix-badges.herokuapp.com/repos/kodegenix/kg-symbol/branches/master/3
[4]: https://travis-ci.org/kodegenix/kg-symbol


## License

Licensed under either of
* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

## Copyright

Copyright (c) 2018 Kodegenix Sp. z o.o. [http://www.kodegenix.pl](http://www.kodegenix.pl)
