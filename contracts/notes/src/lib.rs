#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Escrow(Address), // Maps cooperative to its EscrowState
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowState {
    pub farmer: Address,
    pub cooperative: Address,
    pub token: Address,
    pub amount: i128,
    pub is_released: bool,
}

@contract
pub struct CafeTrustContract;

#[contractimpl]
impl CafeTrustContract {
    /// Initializes and funds an escrow agreement for a crop delivery.
    pub fn create_escrow(
        env: Env,
        cooperative: Address,
        farmer: Address,
        token: Address,
        amount: i128,
    ) {
        cooperative.require_auth();
        
        // Transfer the USDC token deposit from cooperative into this contract
        let client = token::Client::new(&env, &token);
        client.transfer(&cooperative, &env.current_contract_address(), &amount);

        let state = EscrowState {
            farmer,
            cooperative: cooperative.clone(),
            token,
            amount,
            is_released: false,
        };

        env.storage().persistent().set(&DataKey::Escrow(cooperative), &state);
    }

    /// Releases escrowed funds to the farmer after successful crop physical validation.
    pub fn release_funds(env: Env, cooperative: Address) {
        cooperative.require_auth();

        let key = DataKey::Escrow(cooperative.clone());
        let mut state: EscrowState = env.storage().persistent().get(&key).unwrap();
        
        assert!(!state.is_released, "Funds already released");

        // Transfer contract locked funds straight to the farmer
        let client = token::Client::new(&env, &state.token);
        client.transfer(&env.current_contract_address(), &state.farmer, &state.amount);

        state.is_released = true;
        env.storage().persistent().set(&key, &state);
    }

    /// Reads the state of an escrow agreement.
    pub fn get_escrow(env: Env, cooperative: Address) -> EscrowState {
        env.storage().persistent().get(&DataKey::Escrow(cooperative)).unwrap()
    }
}