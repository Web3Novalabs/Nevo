#[cfg(test)]
mod auth_bypass_tests {
    use super::*;

    #[test]
    fn test_mock_auth_cannot_bypass_admin_checks() {
        let mock_user = Address::from([0x01; 20]);
        let result = call_admin_function_as(mock_user);
        assert!(result.is_err(), "Non-admin address must not access admin functions");
    }

    #[test]
    fn test_require_auth_enforced() {
        let result = protected_action_without_auth();
        assert!(result.is_err(), "Unauthenticated calls must be rejected");
    }

    #[test]
    fn test_cross_user_authorization_prevented() {
        let user_a = Address::from([0x01; 20]);
        let user_b = Address::from([0x02; 20]);
        let result = access_user_data_as(user_a, user_b);
        assert!(result.is_err(), "User A must not access User B's data");
    }

    #[test]
    fn test_admin_only_functions_protected() {
        let regular_user = Address::from([0x03; 20]);
        let result = call_admin_only(regular_user);
        assert!(result.is_err(), "Admin-only functions must reject non-admin callers");
    }

    #[test]
    fn test_user_specific_data_isolated() {
        let user_a = create_user_with_data([0x01; 20], 100);
        let user_b = Address::from([0x02; 20]);
        let result = read_user_balance_as(user_a, user_b);
        assert!(result.is_err(), "User B must not read User A's balance");
    }
}
