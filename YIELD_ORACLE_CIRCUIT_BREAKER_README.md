# Yield-Oracle Circuit Breaker Implementation

**Issue #295** - Security & Risk Management Feature

## 🚀 Overview

The Yield-Oracle Circuit Breaker is a critical security component that protects SoroSusu Protocol users during extreme market volatility or negative yield scenarios. It continuously monitors the health of connected AMMs through an oracle system and automatically triggers emergency unwinds when dangerous conditions are detected.

### Key Features

- **🛡️ Real-time Health Monitoring**: Continuous monitoring of AMM health metrics
- **⚡ Automatic Circuit Breaking**: Instant response to dangerous market conditions  
- **🔒 Emergency Unwind**: Secure fund withdrawal to protected vault
- **📊 Health Factor Calculation**: Sophisticated risk assessment algorithm
- **⚙️ Configurable Thresholds**: Customizable risk parameters
- **📝 Comprehensive Auditing**: Complete audit trail of all actions

---

## 🎯 Problem Statement

Traditional DeFi protocols expose users to significant risks during extreme market conditions:

1. **Extreme Volatility**: Rapid price movements can cause impermanent loss
2. **Negative Yield**: AMMs can experience negative yields during market stress
3. **Liquidity Crises**: AMM liquidity can evaporate during market panics
4. **Oracle Failures**: Stale or manipulated oracle data can lead to poor decisions
5. **Rug Pulls**: Malicious AMM operators can drain liquidity

## 💡 Solution Architecture

The Yield-Oracle Circuit Breaker implements a multi-layered defense system:

### 1. Health Monitoring System

```rust
pub struct HealthMetrics {
    pub current_apy: u32,           // Current APY in basis points
    pub volatility_index: u32,      // Volatility index in basis points  
    pub liquidity_ratio: u32,      // Liquidity ratio in basis points
    pub price_impact_score: u32,   // Price impact score in basis points
    pub yield_rate: i32,           // Current yield rate (can be negative)
    pub last_updated: u64,         // Last update timestamp
    pub is_healthy: bool,          // Overall health status
}
```

### 2. Health Factor Algorithm

The system calculates a comprehensive health factor (0-10000) using weighted metrics:

- **Yield Rate (200x weight)**: Negative yields heavily penalized
- **Volatility (2x weight)**: High volatility reduces health
- **Liquidity (0.5x weight)**: Low liquidity reduces health  
- **Price Impact (1x weight)**: High price impact reduces health

### 3. Trigger Conditions

Circuit breaker triggers when:
- Health factor < configured minimum (default 70%)
- Yield rate < negative threshold (default -5%)
- Volatility > configured maximum (default 15%)
- Oracle data is stale (default 1 hour)

---

## 🔧 Implementation Details

### Core Components

#### 1. Circuit Breaker Contract (`yield_oracle_circuit_breaker.rs`)

**Main Functions:**
- `initialize()` - Setup circuit breaker with admin and protected vault
- `register_amm()` - Register AMM for health monitoring
- `update_health_metrics()` - Update AMM health metrics
- `emergency_unwind()` - Execute emergency fund withdrawal
- `manual_trigger()` - Manual override for emergencies

#### 2. Integration with Main Contract

Enhanced existing yield delegation functions:
- Circuit breaker status check before delegation execution
- Automatic AMM registration when delegation starts
- Health monitoring integration throughout lifecycle

#### 3. Configuration Management

```rust
pub struct CircuitBreakerConfig {
    pub min_health_factor: u32,        // Minimum health factor (10000 = 100%)
    pub volatility_threshold: u32,      // Volatility threshold in bps
    pub negative_yield_threshold: i32, // Negative yield threshold
    pub stale_data_period: u64,        // Period before data considered stale
    pub cooldown_period: u64,          // Cooldown period after emergency unwind
    pub auto_unwind_enabled: bool,     // Enable automatic emergency unwind
    pub manual_override_allowed: bool, // Allow manual override
}
```

### State Management

The circuit breaker maintains several states:

```rust
pub enum CircuitBreakerStatus {
    Normal,          // Operating normally
    Warning,         // Health factor declining
    Triggered,       // Circuit breaker activated
    EmergencyUnwind, // Emergency unwind in progress
    Cooldown,        // Cooldown period after unwind
}
```

---

## 🛡️ Security Features

### Access Controls

- **Admin-only Operations**: Configuration updates, manual triggers, resets
- **Public Operations**: Health metric updates (oracle-style), status queries
- **Authorization Checks**: Strict verification for all privileged operations

### Threat Protection

1. **Oracle Manipulation**: Multiple metrics, stale data detection, cross-validation
2. **Front-Running**: Time-based cooldowns, batch processing
3. **DoS Attacks**: Gas cost design, rate limiting
4. **Governance Attacks**: Configuration validation, multi-sig requirements

### Emergency Procedures

- **Manual Override**: Admin can manually trigger in emergencies
- **Emergency Unwind**: Automated fund withdrawal to safe vault
- **Recovery Process**: Cooldown period, admin reset, health verification

---

## 📊 Usage Examples

### Basic Setup

```rust
// Initialize circuit breaker
let protected_vault = Address::generate(&env);
circuit_breaker.initialize(admin, protected_vault);

// Register AMM for monitoring
let amm_address = Address::generate(&env);
let initial_metrics = HealthMetrics {
    current_apy: 500, // 5% APY
    volatility_index: 1000, // 10% volatility
    liquidity_ratio: 8000, // 80% liquidity
    price_impact_score: 500, // 5% price impact
    yield_rate: 500, // Positive yield
    last_updated: env.ledger().timestamp(),
    is_healthy: true,
};

circuit_breaker.register_amm(admin, amm_address, initial_metrics);
```

### Health Monitoring

```rust
// Update health metrics (called by oracle)
let updated_metrics = HealthMetrics {
    current_apy: 200, // Declining APY
    volatility_index: 1800, // High volatility
    liquidity_ratio: 6000, // Lower liquidity
    price_impact_score: 900, // Higher price impact
    yield_rate: -300, // Negative yield!
    last_updated: env.ledger().timestamp(),
    is_healthy: false,
};

circuit_breaker.update_health_metrics(amm_address, updated_metrics);

// Circuit breaker automatically triggers if conditions are met
```

### Emergency Unwind

```rust
// Manual trigger (if auto-unwind disabled)
let reason = String::from_str(&env, "Market crisis detected");
circuit_breaker.manual_trigger(admin, reason);

// Emergency unwind (automatic or manual)
let result = circuit_breaker.emergency_unwind(circle_id, amm_address);
match result {
    Ok(()) => {
        // Funds successfully moved to protected vault
        println!("Emergency unwind completed successfully");
    }
    Err(error) => {
        // Handle error
        println!("Emergency unwind failed: {:?}", error);
    }
}
```

---

## 🧪 Testing

Comprehensive test suite covering:

### Unit Tests
- Circuit breaker initialization
- AMM registration and health monitoring
- Trigger condition testing
- Manual override functionality
- Emergency unwind process

### Integration Tests  
- End-to-end circuit breaker scenarios
- Integration with yield delegation
- Multi-circle emergency scenarios
- Recovery procedures

### Security Tests
- Unauthorized access attempts
- Oracle manipulation attempts
- Stale data handling
- Configuration validation

### Performance Tests
- High-frequency metric updates
- Multiple AMM monitoring
- Emergency unwind performance

Run tests:
```bash
cargo test --test circuit_breaker_test
```

---

## 📈 Monitoring and Alerting

### Real-time Monitoring

- **Health Factor Tracking**: Continuous monitoring of all registered AMMs
- **Volatility Detection**: Real-time volatility analysis and alerts
- **Yield Rate Monitoring**: Continuous yield rate tracking with negative yield alerts

### Alert System

- **Warning States**: Pre-trigger warnings for early intervention
- **Critical Alerts**: Immediate notification of circuit breaker activation  
- **Recovery Notifications**: Status updates during recovery phases

### Event Emissions

All critical operations emit events for transparency:

```rust
// Circuit breaker triggered
env.events().publish(
    (Symbol::new(&env, "circuit_breaker_triggered"),),
    (reason, health_factor, timestamp),
);

// Emergency unwind completed
env.events().publish(
    (Symbol::new(&env, "emergency_unwind_completed"),),
    (circle_id, amount, protected_vault),
);
```

---

## 🔧 Configuration

### Default Configuration

```rust
const DEFAULT_MIN_HEALTH_FACTOR: u32 = 7000;      // 70% health factor threshold
const DEFAULT_VOLATILITY_THRESHOLD: u32 = 1500;  // 15% volatility threshold  
const DEFAULT_NEGATIVE_YIELD_THRESHOLD: i32 = -500; // -5% yield threshold
const DEFAULT_STALE_DATA_PERIOD: u64 = 3600;      // 1 hour
const DEFAULT_COOLDOWN_PERIOD: u64 = 86400;      // 24 hours
```

### Custom Configuration

```rust
let custom_config = CircuitBreakerConfig {
    min_health_factor: 8000,        // Higher threshold (80%)
    volatility_threshold: 1200,     // Lower volatility threshold (12%)
    negative_yield_threshold: -300, // Higher negative threshold (-3%)
    stale_data_period: 1800,        // Shorter stale period (30 min)
    cooldown_period: 43200,         // Shorter cooldown (12 hours)
    auto_unwind_enabled: true,
    manual_override_allowed: true,
};

circuit_breaker.update_config(admin, custom_config);
```

---

## 🚀 Deployment

### Pre-deployment Checklist

- [ ] Security audit completed
- [ ] All tests passing
- [ ] Configuration parameters verified
- [ ] Admin keys secured
- [ ] Protected vault deployed and funded
- [ ] Oracle sources configured
- [ ] Monitoring systems active

### Deployment Steps

1. **Deploy Circuit Breaker Contract**
2. **Initialize with Admin and Protected Vault**
3. **Configure Parameters**
4. **Register AMMs for Monitoring**
5. **Enable Health Monitoring**
6. **Test Emergency Procedures**

### Post-deployment Monitoring

- Monitor health factor trends
- Watch for false positives
- Track emergency unwind events
- Verify fund protection effectiveness

---

## 🔮 Future Enhancements

### Planned Features

1. **Multi-Oracle Support**: Aggregate data from multiple oracle sources
2. **AI-Powered Detection**: Machine learning for anomaly detection
3. **Dynamic Thresholds**: Adaptive thresholds based on market conditions
4. **Cross-Chain Support**: Monitor AMMs across multiple chains
5. **Insurance Integration**: Integration with DeFi insurance protocols

### Research Areas

1. **Predictive Analytics**: Early warning systems for market stress
2. **Optimized Unwinding**: More sophisticated unwinding strategies
3. **Economic Modeling**: Better understanding of systemic risks
4. **Governance Integration**: DAO-based decision making for parameters

---

## 📚 Documentation

- [Security Documentation](./CIRCUIT_BREAKER_SECURITY.md)
- [API Reference](./docs/api.md)
- [Integration Guide](./docs/integration.md)
- [Troubleshooting Guide](./docs/troubleshooting.md)

---

## 🤝 Contributing

Contributions to the Yield-Oracle Circuit Breaker are welcome! Please:

1. Review the security documentation
2. Write comprehensive tests
3. Follow the coding standards
4. Submit PRs with clear descriptions
5. Participate in security reviews

---

## 📄 License

This implementation is part of the SoroSusu Protocol and follows the same licensing terms.

---

## ⚠️ Important Notes

1. **Security Critical**: This is a security-critical component - thorough testing required
2. **Admin Trust**: Admin has significant power - consider multi-sig implementation
3. **Oracle Reliability**: System depends on reliable oracle data sources
4. **Configuration Sensitivity**: Poor configuration can cause false positives or missed risks
5. **Economic Impact**: Circuit breaker activation has economic consequences

---

## 🆘 Support

For issues or questions about the Yield-Oracle Circuit Breaker:

1. Check the troubleshooting guide
2. Review test cases for examples
3. Consult the security documentation
4. Contact the development team

---

**Built with ❤️ for the SoroSusu Protocol Community**
