# Contract getter conventions

Every pool-scoped getter fails with `ContractError::PoolNotFound` (`Error(Contract, #1)`) when the pool does not exist.

An empty value (`0`, `""`, an empty list or `None`) means the pool exists and has no data for that field. Clients should use the generated `try_` form, or handle the failed simulation, to tell the two cases apart.
