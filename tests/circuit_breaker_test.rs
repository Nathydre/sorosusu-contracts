use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Env, String, Symbol, Vec, i128, u64, u32,
    token, testutils::{Address as TestAddress, Ledger as TestLedger},
};
use sorosusu_contracts::{
    SoroSusu, SoroSusuClient,
    yield_oracle_circuit_breaker::{
        YieldOracleCircuitBreaker, CircuitBreakerConfig, CircuitBreakerState, 
        CircuitBreakerStatus, HealthMetrics, CircuitBreakerError
    },
};

#[contract]
pub struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn initialize(env: Env, admin: Address) {
        // Mock token initialization
    }
    
    pub fn balance(env: Env, addr: Address) -> i128 {
        // Mock balance
        1_000_000_000_000
    }
    
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        // Mock transfer
    }
}

#[test]
fn test_circuit_breaker_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let protected_vault = Address::generate(&env);
    
    // Initialize circuit breaker
    YieldOracleCircuitBreaker::initialize(env.clone(), admin.clone(), protected_vault.clone());
    
    // Check initial state
    let state = YieldOracleCircuitBreaker::get_circuit_breaker_status(env.clone());
    assert_eq!(state.status, CircuitBreakerStatus::Normal);
    assert_eq!(state.health_factor, 10000); // 100% health
    assert_eq!(state.emergency_unwind_count, 0);
    
    // Check configuration
    let config = CircuitBreakerConfig {
        min_health_factor: 7000,
        volatility_threshold: 1500,
        negative_yield_threshold: -500,
        stale_data_period: 3600,
        cooldown_period: 86400,
        auto_unwind_enabled: true,
        manual_override_allowed: true,
    };
    
    YieldOracleCircuitBreaker::update_config(env.clone(), admin.clone(), config);
    
    env.events().publish(
        (Symbol::new(&env, "circuit_breaker_initialized"),),
        (admin, protected_vault),
    );
}

#[test]
fn test_amm_registration_and_health_monitoring() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let protected_vault = Address::generate(&env);
    let amm_address = Address::generate(&env);
    
    // Initialize circuit breaker
    YieldOracleCircuitBreaker::initialize(env.clone(), admin.clone(), protected_vault.clone());
    
    // Create initial health metrics
    let initial_metrics = HealthMetrics {
        current_apy: 500, // 5% APY
        volatility_index: 1000, // 10% volatility
        liquidity_ratio: 8000, // 80% liquidity
        price_impact_score: 500, // 5% price impact
        yield_rate: 500, // Positive yield
        last_updated: env.ledger().timestamp(),
        is_healthy: true,
    };
    
    // Register AMM for monitoring
    YieldOracleCircuitBreaker::register_amm(env.clone(), admin.clone(), amm_address.clone(), initial_metrics.clone());
    
    // Verify registration
    let stored_metrics = YieldOracleCircuitBreaker::get_health_metrics(env.clone(), amm_address.clone());
    assert_eq!(stored_metrics.current_apy, 500);
    assert_eq!(stored_metrics.is_healthy, true);
    
    // Update health metrics with declining conditions
    let declining_metrics = HealthMetrics {
        current_apy: 200, // 2% APY (declining)
        volatility_index: 2000, // 20% volatility (increasing)
        liquidity_ratio: 6000, // 60% liquidity (decreasing)
        price_impact_score: 800, // 8% price impact (increasing)
        yield_rate: -300, // Negative yield
        last_updated: env.ledger().timestamp(),
        is_healthy: false,
    };
    
    YieldOracleCircuitBreaker::update_health_metrics(env.clone(), amm_address.clone(), declining_metrics);
    
    // Check if circuit breaker is triggered
    let state = YieldOracleCircuitBreaker::get_circuit_breaker_status(env.clone());
    assert!(state.status != CircuitBreakerStatus::Normal); // Should be warning or triggered
}

#[test]
fn test_circuit_breaker_trigger_conditions() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let protected_vault = Address::generate(&env);
    let amm_address = Address::generate(&env);
    
    // Initialize circuit breaker
    YieldOracleCircuitBreaker::initialize(env.clone(), admin.clone(), protected_vault.clone());
    
    // Register AMM
    let initial_metrics = HealthMetrics {
        current_apy: 500,
        volatility_index: 1000,
        liquidity_ratio: 8000,
        price_impact_score: 500,
        yield_rate: 500,
        last_updated: env.ledger().timestamp(),
        is_healthy: true,
    };
    
    YieldOracleCircuitBreaker::register_amm(env.clone(), admin.clone(), amm_address.clone(), initial_metrics);
    
    // Test 1: Negative yield trigger
    let negative_yield_metrics = HealthMetrics {
        current_apy: 0,
        volatility_index: 1000,
        liquidity_ratio: 8000,
        price_impact_score: 500,
        yield_rate: -1000, // -10% yield (below -5% threshold)
        last_updated: env.ledger().timestamp(),
        is_healthy: false,
    };
    
    YieldOracleCircuitBreaker::update_health_metrics(env.clone(), amm_address.clone(), negative_yield_metrics);
    
    let state = YieldOracleCircuitBreaker::get_circuit_breaker_status(env.clone());
    assert_eq!(state.status, CircuitBreakerStatus::Triggered);
    
    // Reset for next test
    YieldOracleCircuitBreaker::reset_circuit_breaker(env.clone(), admin.clone());
    
    // Test 2: High volatility trigger
    let high_volatility_metrics = HealthMetrics {
        current_apy: 500,
        volatility_index: 2000, // 20% volatility (above 15% threshold)
        liquidity_ratio: 8000,
        price_impact_score: 500,
        yield_rate: 500,
        last_updated: env.ledger().timestamp(),
        is_healthy: false,
    };
    
    YieldOracleCircuitBreaker::update_health_metrics(env.clone(), amm_address.clone(), high_volatility_metrics);
    
    let state = YieldOracleCircuitBreaker::get_circuit_breaker_status(env.clone());
    assert_eq!(state.status, CircuitBreakerStatus::Triggered);
    
    // Reset for next test
    YieldOracleCircuitBreaker::reset_circuit_breaker(env.clone(), admin.clone());
    
    // Test 3: Low health factor trigger
    let low_health_metrics = HealthMetrics {
        current_apy: 100,
        volatility_index: 1200,
        liquidity_ratio: 4000, // 40% liquidity
        price_impact_score: 1200, // 12% price impact
        yield_rate: 100,
        last_updated: env.ledger().timestamp(),
        is_healthy: false,
    };
    
    YieldOracleCircuitBreaker::update_health_metrics(env.clone(), amm_address.clone(), low_health_metrics);
    
    let state = YieldOracleCircuitBreaker::get_circuit_breaker_status(env.clone());
    assert_eq!(state.status, CircuitBreakerStatus::Triggered);
}

#[test]
fn test_manual_trigger_and_reset() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let protected_vault = Address::generate(&env);
    let unauthorized_user = Address::generate(&env);
    
    // Initialize circuit breaker
    YieldOracleCircuitBreaker::initialize(env.clone(), admin.clone(), protected_vault.clone());
    
    // Test unauthorized manual trigger (should fail)
    let reason = String::from_str(&env, "Manual test trigger");
    std::panic::catch_unwind(|| {
        YieldOracleCircuitBreaker::manual_trigger_circuit_breaker(env.clone(), unauthorized_user.clone(), reason.clone());
    }).expect_err("Should panic for unauthorized user");
    
    // Test authorized manual trigger
    YieldOracleCircuitBreaker::manual_trigger_circuit_breaker(env.clone(), admin.clone(), reason.clone());
    
    let state = YieldOracleCircuitBreaker::get_circuit_breaker_status(env.clone());
    assert_eq!(state.status, CircuitBreakerStatus::Triggered);
    assert!(state.triggered_at.is_some());
    
    // Test reset before cooldown (should fail)
    std::panic::catch_unwind(|| {
        YieldOracleCircuitBreaker::reset_circuit_breaker(env.clone(), admin.clone());
    }).expect_err("Should panic during cooldown period");
    
    // Advance time past cooldown period
    env.ledger().set_timestamp(env.ledger().timestamp() + 86400 + 1);
    
    // Test reset after cooldown
    YieldOracleCircuitBreaker::reset_circuit_breaker(env.clone(), admin.clone());
    
    let state = YieldOracleCircuitBreaker::get_circuit_breaker_status(env.clone());
    assert_eq!(state.status, CircuitBreakerStatus::Normal);
    assert_eq!(state.health_factor, 10000);
}

#[test]
fn test_emergency_unwind_integration() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let user1 = Address::generate(&env);
    let protected_vault = Address::generate(&env);
    let amm_address = Address::generate(&env);
    
    // Setup main contract
    let contract_id = env.register_contract(None, SoroSusu);
    let client = SoroSusuClient::new(&env, &contract_id);
    
    // Initialize main contract
    client.init(&admin);
    
    // Initialize circuit breaker
    client.initialize_circuit_breaker(&admin, &protected_vault);
    
    // Create a circle and yield delegation
    let usdc_address = Address::generate(&env);
    let circle_id = client.create_circle(
        &creator,
        &1_000_000_000,
        &2,
        &usdc_address,
        &86400,
        &100,
        &Address::generate(&env),
        &admin,
    );
    
    client.join_circle(&creator, &circle_id, &1, &None);
    client.join_circle(&user1, &circle_id, &1, &None);
    
    // Complete first cycle to have funds
    client.deposit(&creator, &circle_id);
    client.deposit(&user1, &circle_id);
    client.finalize_round(&creator, &circle_id);
    
    // Start yield delegation
    client.propose_yield_delegation(
        &creator,
        &circle_id,
        &5000, // 50% delegation
        &amm_address,
        &sorosusu_contracts::YieldPoolType::StellarLiquidityPool,
    );
    
    // Vote for delegation
    client.vote_yield_delegation(&user1, &circle_id, &sorosusu_contracts::YieldVoteChoice::For);
    client.vote_yield_delegation(&creator, &circle_id, &sorosusu_contracts::YieldVoteChoice::For);
    
    // Approve and execute delegation
    client.approve_yield_delegation(&circle_id);
    client.execute_yield_delegation(&circle_id);
    
    // Register AMM for monitoring
    let initial_metrics = HealthMetrics {
        current_apy: 500,
        volatility_index: 1000,
        liquidity_ratio: 8000,
        price_impact_score: 500,
        yield_rate: 500,
        last_updated: env.ledger().timestamp(),
        is_healthy: true,
    };
    
    client.register_amm_for_monitoring(&admin, &amm_address, &initial_metrics);
    
    // Trigger circuit breaker with negative yield
    let negative_yield_metrics = HealthMetrics {
        current_apy: 0,
        volatility_index: 1000,
        liquidity_ratio: 8000,
        price_impact_score: 500,
        yield_rate: -1000, // -10% yield
        last_updated: env.ledger().timestamp(),
        is_healthy: false,
    };
    
    client.update_amm_health_metrics(&amm_address, &negative_yield_metrics);
    
    // Verify circuit breaker is triggered
    let status = client.get_circuit_breaker_status();
    assert_eq!(status.status, CircuitBreakerStatus::Triggered);
    
    // Execute emergency unwind
    let result = client.emergency_unwind(&circle_id, &amm_address);
    assert!(result.is_ok());
    
    // Verify emergency unwind record
    let records = YieldOracleCircuitBreaker::get_emergency_unwind_records(env.clone(), 10);
    assert!(records.len() > 0);
    
    let last_record = records.last().unwrap();
    assert_eq!(last_record.circle_id, circle_id);
    assert_eq!(last_record.success, true);
    assert!(last_record.unwind_amount > 0);
}

#[test]
fn test_stale_data_handling() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let protected_vault = Address::generate(&env);
    let amm_address = Address::generate(&env);
    
    // Initialize circuit breaker
    YieldOracleCircuitBreaker::initialize(env.clone(), admin.clone(), protected_vault.clone());
    
    // Register AMM with old timestamp
    let old_timestamp = env.ledger().timestamp() - 7200; // 2 hours ago
    let stale_metrics = HealthMetrics {
        current_apy: 500,
        volatility_index: 1000,
        liquidity_ratio: 8000,
        price_impact_score: 500,
        yield_rate: 500,
        last_updated: old_timestamp,
        is_healthy: true,
    };
    
    YieldOracleCircuitBreaker::register_amm(env.clone(), admin.clone(), amm_address.clone(), stale_metrics);
    
    // Update with stale data
    YieldOracleCircuitBreaker::update_health_metrics(env.clone(), amm_address.clone(), stale_metrics);
    
    // Should be in warning state due to stale data
    let state = YieldOracleCircuitBreaker::get_circuit_breaker_status(env.clone());
    assert_eq!(state.status, CircuitBreakerStatus::Warning);
}

#[test]
fn test_health_factor_calculation() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let protected_vault = Address::generate(&env);
    let amm_address = Address::generate(&env);
    
    // Initialize circuit breaker
    YieldOracleCircuitBreaker::initialize(env.clone(), admin.clone(), protected_vault.clone());
    
    // Test various health factor scenarios
    
    // Scenario 1: Perfect health
    let perfect_metrics = HealthMetrics {
        current_apy: 1000, // 10% APY
        volatility_index: 500, // 5% volatility
        liquidity_ratio: 9000, // 90% liquidity
        price_impact_score: 300, // 3% price impact
        yield_rate: 1000, // 10% yield
        last_updated: env.ledger().timestamp(),
        is_healthy: true,
    };
    
    YieldOracleCircuitBreaker::register_amm(env.clone(), admin.clone(), amm_address.clone(), perfect_metrics);
    
    let state = YieldOracleCircuitBreaker::get_circuit_breaker_status(env.clone());
    assert!(state.health_factor >= 9000); // Should be very high
    assert_eq!(state.status, CircuitBreakerStatus::Normal);
    
    // Scenario 2: Moderate risk
    let moderate_risk_metrics = HealthMetrics {
        current_apy: 300, // 3% APY
        volatility_index: 1200, // 12% volatility
        liquidity_ratio: 7000, // 70% liquidity
        price_impact_score: 800, // 8% price impact
        yield_rate: 200, // 2% yield
        last_updated: env.ledger().timestamp(),
        is_healthy: true,
    };
    
    YieldOracleCircuitBreaker::update_health_metrics(env.clone(), amm_address.clone(), moderate_risk_metrics);
    
    let state = YieldOracleCircuitBreaker::get_circuit_breaker_status(env.clone());
    assert!(state.health_factor < 9000 && state.health_factor >= 7000); // Moderate range
    assert_eq!(state.status, CircuitBreakerStatus::Normal);
    
    // Scenario 3: High risk
    let high_risk_metrics = HealthMetrics {
        current_apy: 100, // 1% APY
        volatility_index: 1800, // 18% volatility
        liquidity_ratio: 5000, // 50% liquidity
        price_impact_score: 1200, // 12% price impact
        yield_rate: -200, // -2% yield
        last_updated: env.ledger().timestamp(),
        is_healthy: false,
    };
    
    YieldOracleCircuitBreaker::update_health_metrics(env.clone(), amm_address.clone(), high_risk_metrics);
    
    let state = YieldOracleCircuitBreaker::get_circuit_breaker_status(env.clone());
    assert!(state.health_factor < 7000); // Should trigger circuit breaker
    assert_eq!(state.status, CircuitBreakerStatus::Triggered);
}

#[test]
fn test_yield_delegation_block_during_circuit_breaker() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let user1 = Address::generate(&env);
    let protected_vault = Address::generate(&env);
    let amm_address = Address::generate(&env);
    
    // Setup main contract
    let contract_id = env.register_contract(None, SoroSusu);
    let client = SoroSusuClient::new(&env, &contract_id);
    
    // Initialize contracts
    client.init(&admin);
    client.initialize_circuit_breaker(&admin, &protected_vault);
    
    // Manually trigger circuit breaker first
    let reason = String::from_str(&env, "Pre-emptive circuit breaker activation");
    client.manual_trigger_circuit_breaker(&admin, &reason);
    
    // Create a circle
    let usdc_address = Address::generate(&env);
    let circle_id = client.create_circle(
        &creator,
        &1_000_000_000,
        &2,
        &usdc_address,
        &86400,
        &100,
        &Address::generate(&env),
        &admin,
    );
    
    client.join_circle(&creator, &circle_id, &1, &None);
    client.join_circle(&user1, &circle_id, &1, &None);
    
    // Complete first cycle
    client.deposit(&creator, &circle_id);
    client.deposit(&user1, &circle_id);
    client.finalize_round(&creator, &circle_id);
    
    // Start yield delegation
    client.propose_yield_delegation(
        &creator,
        &circle_id,
        &5000,
        &amm_address,
        &sorosusu_contracts::YieldPoolType::StellarLiquidityPool,
    );
    
    // Vote for delegation
    client.vote_yield_delegation(&user1, &circle_id, &sorosusu_contracts::YieldVoteChoice::For);
    client.vote_yield_delegation(&creator, &circle_id, &sorosusu_contracts::YieldVoteChoice::For);
    
    // Approve delegation
    client.approve_yield_delegation(&circle_id);
    
    // Try to execute delegation (should fail due to circuit breaker)
    std::panic::catch_unwind(|| {
        client.execute_yield_delegation(&circle_id);
    }).expect_err("Should panic when circuit breaker is active");
    
    // Verify circuit breaker is still active
    let status = client.get_circuit_breaker_status();
    assert_eq!(status.status, CircuitBreakerStatus::Triggered);
}
