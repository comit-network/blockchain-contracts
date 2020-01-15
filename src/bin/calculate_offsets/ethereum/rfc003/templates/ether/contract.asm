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

    invalid_secret
    jump

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

    mstore(0, 0xBBAD9D5BF43FC68B6AB3D56342306BFC459ABE19DD1D361DBCAB75C00400B85C)
    revert(0, 32)

invalid_secret:
    mstore(0, 0x05F03EBF077F616C9D02B91C7FCBAC32BEEF85527AEDFF9CF81357A5A00C8C41)
    revert(0, 32)

redeem:
    log1(0, 32, 0xB8CAC300E37F03AD332E581DEA21B2F0B84EAAADC184A295FEF71E81F44A7413) // log keccak256(Redeemed(<secret>))
    selfdestruct(0x3000000000000000000000000000000000000003) 

refund:
    log1(0, 0, 0x5D26862916391BF49478B2F5103B0720A842B45EF145A268F2CD1FB2AED55178) // log keccak256(Refunded())
    selfdestruct(0x4000000000000000000000000000000000000004)
}
