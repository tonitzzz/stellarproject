#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, token, Address, Env};

    fn setup_test_env<'a>() -> (Env, CafeTrustContractClient<'a>, Address, Address, token::Client<'a>, token::StellarAssetClient<'a>) {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, CafeTrustContract);
        let client = CafeTrustContractClient::new(&env, &contract_id);

        let cooperative = Address::generate(&env);
        let farmer = Address::generate(&env);

        let token_id = env.register_stellar_asset_contract(Address::generate(&env));
        let token_admin = token::StellarAssetClient::new(&env, &token_id);
        let token_client = token::Client::new(&env, &token_id);

        // Mint initial tokens to cooperative
        token_admin.mint(&cooperative, &1000);

        (env, client, cooperative, farmer, token_client, token_admin)
    }

    #[test]
    fn test_1_happy_path_escrow_lifecycle() {
        let (env, client, cooperative, farmer, token_client, _) = setup_test_env();

        // Create escrow
        client.create_escrow(&cooperative, &farmer, &token_client.address, &500);
        assert_eq!(token_client.balance(&cooperative), 500);
        assert_eq!(token_client.balance(&client.address), 500);

        // Release escrow
        client.release_funds(&cooperative);
        assert_eq!(token_client.balance(&farmer), 500);
        assert_eq!(token_client.balance(&client.address), 0);
    }

    #[test]
    #[should_panic(expected = "Funds already released")]
    fn test_2_edge_case_cannot_release_twice() {
        let (_, client, cooperative, farmer, token_client, _) = setup_test_env();

        client.create_escrow(&cooperative, &farmer, &token_client.address, &500);
        client.release_funds(&cooperative);
        
        // This second call must fail/panic
        client.release_funds(&cooperative);
    }

    #[test]
    fn test_3_state_verification() {
        let (_, client, cooperative, farmer, token_client, _) = setup_test_env();

        client.create_escrow(&cooperative, &farmer, &token_client.address, &500);
        
        let state = client.get_escrow(&cooperative);
        assert_eq!(state.farmer, farmer);
        assert_eq!(state.amount, 500);
        assert_eq!(state.is_released, false);
    }

    #[test]
    #[should_panic]
    fn test_4_insufficient_balance_fails() {
        let (_, client, cooperative, farmer, token_client, _) = setup_test_env();
        
        // Cooperative tries to lock 2000 tokens when they only possess 1000
        client.create_escrow(&cooperative, &farmer, &token_client.address, &2000);
    }

    #[test]
    fn test_5_multiple_independent_escrows() {
        let (env, client, cooperative_1, farmer, token_client, token_admin) = setup_test_env();
        let cooperative_2 = Address::generate(&env);
        token_admin.mint(&cooperative_2, &1000);

        client.create_escrow(&cooperative_1, &farmer, &token_client.address, &300);
        client.create_escrow(&cooperative_2, &farmer, &token_client.address, &400);

        assert_eq!(client.get_escrow(&cooperative_1).amount, 300);
        assert_eq!(client.get_escrow(&cooperative_2).amount, 400);
    }
}