use soroban_sdk::{contractimpl, Env, Address, Symbol};

pub struct SusuLending;

/// Lending logic between Susu groups
#[contractimpl]
impl SusuLending {
    /// Initiate a loan from Group A to Group B
    pub fn lend_reserve(env: Env, lender: Address, borrower: Address, amount: i128, interest_rate: i128, duration: u64) {
        // Collateralization check: borrower must have high reliability index
        let reliability: i128 = env.storage().get(&Symbol::short("reliability_index")).unwrap_or(0);
        if reliability < 70 {
            panic!("Borrower reliability too low");
        }

        // Deduct from lender reserve
        let lender_reserve: i128 = env.storage().get(&lender).unwrap_or(0);
        if lender_reserve < amount {
            panic!("Insufficient reserve");
        }
        env.storage().set(&lender, &(lender_reserve - amount));

        // Credit borrower reserve
        let borrower_reserve: i128 = env.storage().get(&borrower).unwrap_or(0);
        env.storage().set(&borrower, &(borrower_reserve + amount));

        // Record loan terms
        let loan_key = (lender.clone(), borrower.clone());
        env.storage().set(&loan_key, &(amount, interest_rate, duration, env.ledger().timestamp()));

        // Emit event
        env.events().publish(
            (Symbol::short("loan_created"),),
            (lender, borrower, amount, interest_rate, duration),
        );
    }

    /// Repay loan with interest
    pub fn repay_loan(env: Env, lender: Address, borrower: Address) {
        let loan_key = (lender.clone(), borrower.clone());
        let (amount, interest_rate, duration, start_time): (i128, i128, u64, u64) =
            env.storage().get(&loan_key).unwrap();

        let now = env.ledger().timestamp();
        let elapsed = now - start_time;

        // Simple interest calculation
        let interest = (amount * interest_rate * elapsed as i128) / (duration as i128 * 100);

        let total_due = amount + interest;

        // Deduct from borrower
        let borrower_reserve: i128 = env.storage().get(&borrower).unwrap_or(0);
        if borrower_reserve < total_due {
            panic!("Borrower cannot repay");
        }
        env.storage().set(&borrower, &(borrower_reserve - total_due));

        // Credit lender
        let lender_reserve: i128 = env.storage().get(&lender).unwrap_or(0);
        env.storage().set(&lender, &(lender_reserve + total_due));

        // Clear loan record
        env.storage().remove(&loan_key);

        // Emit event
        env.events().publish(
            (Symbol::short("loan_repaid"),),
            (lender, borrower, total_due),
        );
    }

    /// Check outstanding loan terms
    pub fn get_loan(env: Env, lender: Address, borrower: Address) -> Option<(i128, i128, u64, u64)> {
        let loan_key = (lender, borrower);
        env.storage().get(&loan_key)
    }
}
