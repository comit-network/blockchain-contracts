{
    // Load received secret size
    calldatasize

    // Check if secret is zero length
    iszero

    // If secret is zero length, jump to branch that checks if expiry time has been reached
    check_expiry
    jumpi

    // Load expected secret size
    32

    // Load received secret size
    calldatasize

    // Compare secret size
    eq
    iszero

    // If passed secret is wrong size, jump to exit contract
    invalid_secret
    jumpi

    // Load secret into memory
    calldatacopy(0, 0, 32)

    // Hash secret with SHA-256 (pre-compiled contract 0x02)
    call(72, 0x02, 0, 0, 32, 33, 32)

    // Placeholder for correct secret hash
    0x1000000000000000000000000000000000000000000000000000000000000001

    // Load hashed secret from memory
    mload(33)

    // Compare hashed secret with existing one
    eq

    // Combine `eq` result with `call` result
    and

    // Jump to redeem if hashes match
    redeem
    jumpi

    // Continue to invalid secret if no match
invalid_secret:
    // return "invalidSecret" = 0x696e76616c69645365637265740000000000000000000000000000000000000000
    mstore(0, "invalidSecret")
    revert(0, 32)

check_expiry:
    // Timestamp of the current block in seconds since the epoch
    timestamp

    // Placeholder for refund timestamp
    0x20000002

    // Compare refund timestamp with current timestamp
    lt

    // Jump to refund if time is expired
    refund
    jumpi

    // return "too early" = 0x746f6f4561726c79000000000000000000000000000000000000000000000000
    mstore(0, "tooEarly")
    revert(0, 32)


redeem:
    // log ascii to hex of "redeemed"
    // 0x72656465656d6564000000000000000000000000000000000000000000000000
    log1(0, 32, "redeemed")
    selfdestruct(0x3000000000000000000000000000000000000003) 

refund:
    // log ascii to hex of "refunded"
    // 0x726566756e646564000000000000000000000000000000000000000000000000
    log1(0, 0, "refunded")
    selfdestruct(0x4000000000000000000000000000000000000004)
}
