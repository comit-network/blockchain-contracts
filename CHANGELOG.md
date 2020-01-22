# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **Breaking Ether and Erc20 HTLCs:** A transaction to the HTLCs (Ether or Erc20) now `reverts` with an error message if someone tries to (1) refund too early or (2) redeem with a wrong secret. Additionally, the log messages have changed. For more details, checkout this PR: https://github.com/comit-network/blockchain-contracts/pull/37 .


## 0.1.0 - 2019-10-14
### Added
- Add implementation of COMIT RFC-003 for Bitcoin.
- Add implementation of COMIT RFC-003 for Ether.
- Add implementation of COMIT RFC-003 for ERC-20.

[Unreleased]: https://github.com/coblox/blockchain-contracts/compare/0.1.0...HEAD
[0.1.0]: https://github.com/coblox/blockchain-contracts/compare/ab341e430ca514576ac9ca553a35ba339f293cc3...0.1.0
