#[cfg(test)]
mod token_transfer_error_tests {
    use super::*;

    #[test]
    fn test_insufficient_balance_handled_gracefully() {
        // Arrange: user with zero balance attempts transfer
        // Assert: returns Err, state unchanged
        let result = transfer_tokens(0, 100);
        assert!(result.is_err(), "Should fail on insufficient balance");
        assert_eq!(get_state_balance(), 0, "State must not be corrupted");
    }

    #[test]
    fn test_invalid_token_contract_rejected() {
        let result = transfer_to_contract(Address::zero(), 50);
        assert!(result.is_err(), "Zero address contract must be rejected");
    }

    #[test]
    fn test_transfer_failure_does_not_corrupt_state() {
        let balance_before = get_state_balance();
        let _ = transfer_tokens(balance_before + 1, 999);
        assert_eq!(get_state_balance(), balance_before, "State must remain intact after failed transfer");
    }

    #[test]
    fn test_partial_transfers_prevented() {
        let result = partial_transfer(50, 100);
        assert!(result.is_err(), "Partial transfers must not be allowed");
    }

    #[test]
    fn test_error_messages_are_clear() {
        let err = transfer_tokens(0, 100).unwrap_err();
        let msg = format!("{:?}", err);
        assert!(msg.contains("insufficient") || msg.contains("balance"),
            "Error message must be descriptive: got {}", msg);
    }
}
