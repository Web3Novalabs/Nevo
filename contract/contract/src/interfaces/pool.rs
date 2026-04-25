use soroban_sdk::{Address, Env};

use crate::base::types::{PoolConfig, PoolDetails};

/// Trait defining the core functionality for managing Scholarship Pools.
/// This trait ensures consistent interfaces for pool operations across different implementations.
#[allow(dead_code)]
pub trait PoolTrait {
    /// Creates a new scholarship pool with the given configuration.
    ///
    /// This function initializes a new pool on the blockchain, storing the pool configuration
    /// and setting up initial state and metrics. The pool ID is typically auto-generated.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `config` - The pool configuration including name, description, target amount, etc.
    /// * `creator` - The address of the pool creator
    ///
    /// # Returns
    /// The unique ID of the newly created pool
    fn create_pool(env: Env, config: PoolConfig, creator: Address) -> u64;

    /// Pauses or unpauses a scholarship pool.
    ///
    /// When a pool is paused, contributions are temporarily disabled until unpaused.
    /// Only the pool creator or authorized administrators can pause/unpause pools.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `pool_id` - The unique ID of the pool to pause/unpause
    /// * `pause` - true to pause the pool, false to unpause
    fn pause_pool(env: Env, pool_id: u64, pause: bool);

    /// Retrieves detailed information about a scholarship pool.
    ///
    /// Returns comprehensive pool data including configuration, current state,
    /// metrics, and metadata for display or processing.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `pool_id` - The unique ID of the pool to query
    ///
    /// # Returns
    /// A PoolDetails struct containing all pool information
    fn get_pool_details(env: Env, pool_id: u64) -> PoolDetails;
}
