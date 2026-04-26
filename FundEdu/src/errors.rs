use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FundEduError {
    // Pool errors
    InvalidSponsor = 1,
    PoolNotActive = 2,
    // Application errors
    DuplicateApplication = 3,
    UnauthorizedValidator = 4,
    NotPending = 5,
    AlreadyRejected = 6,
    // Validation errors
    InvalidFunding = 7,  // target_amount must be > 0
    PoolNotFound = 8,
    /// Validator address is not registered in the SchoolRegistry.
    UnrecognizedValidator = 9,
}

#[cfg(test)]
mod tests {
    use super::FundEduError;

    #[test]
    fn discriminants_are_correct() {
        assert_eq!(FundEduError::InvalidSponsor as u32, 1);
        assert_eq!(FundEduError::PoolNotActive as u32, 2);
        assert_eq!(FundEduError::DuplicateApplication as u32, 3);
        assert_eq!(FundEduError::UnauthorizedValidator as u32, 4);
        assert_eq!(FundEduError::NotPending as u32, 5);
        assert_eq!(FundEduError::AlreadyRejected as u32, 6);
        assert_eq!(FundEduError::InvalidFunding as u32, 7);
        assert_eq!(FundEduError::PoolNotFound as u32, 8);
        assert_eq!(FundEduError::UnrecognizedValidator as u32, 9);
    }

    #[test]
    fn discriminants_are_distinct() {
        let variants = [
            FundEduError::InvalidSponsor,
            FundEduError::PoolNotActive,
            FundEduError::DuplicateApplication,
            FundEduError::UnauthorizedValidator,
            FundEduError::NotPending,
            FundEduError::AlreadyRejected,
            FundEduError::InvalidFunding,
            FundEduError::PoolNotFound,
            FundEduError::UnrecognizedValidator,
        ];
        for i in 0..variants.len() {
            for j in (i + 1)..variants.len() {
                assert_ne!(variants[i], variants[j]);
            }
        }
    }

    #[test]
    fn ordering_follows_discriminant() {
        assert!(FundEduError::InvalidSponsor < FundEduError::PoolNotActive);
        assert!(FundEduError::PoolNotActive < FundEduError::DuplicateApplication);
        assert!(FundEduError::DuplicateApplication < FundEduError::UnauthorizedValidator);
        assert!(FundEduError::UnauthorizedValidator < FundEduError::NotPending);
        assert!(FundEduError::NotPending < FundEduError::AlreadyRejected);
        assert!(FundEduError::AlreadyRejected < FundEduError::InvalidFunding);
        assert!(FundEduError::InvalidFunding < FundEduError::PoolNotFound);
        assert!(FundEduError::PoolNotFound < FundEduError::UnrecognizedValidator);
    }
}
