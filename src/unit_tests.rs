use cosmwasm_std::{
    coins, BankMsg, Coin, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128, Addr,
};
use crate::{contract::{instantiate, execute}, msg::{InstantiateMsg, ExecuteMsg}};
use crate::state::{Plot, PLOTS, TREASURY, POINTS, TOTAL_POINTS, ROYALTY_ADDRESS, INITIAL_PRICE};
use cosmwasm_std::testing::{mock_env, mock_info, mock_dependencies};

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_env, mock_info};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
    
        let msg = InstantiateMsg {
            initial_price: Uint128::new(100),
            royalty_address: "royalty_addr".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "sei"));
    
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    
        // Check if the initial price and royalty address are set correctly
        assert_eq!(INITIAL_PRICE.load(&deps.storage).unwrap(), Uint128::new(100));
        assert_eq!(ROYALTY_ADDRESS.load(&deps.storage).unwrap(), Addr::unchecked("royalty_addr"));
    }

    #[test]
    fn test_buy() {
        let mut deps = mock_dependencies();

        // Initialize with royalty address
        let init_msg = InstantiateMsg {
            initial_price: Uint128::new(100),
            royalty_address: "royalty_addr".to_string(),
        };
        let init_info = mock_info("creator", &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), init_info, init_msg).unwrap();

        let buy_msg = ExecuteMsg::Buy { coordinates: (0, 0) };
        let buy_info = mock_info("buyer", &coins(200, "sei"));
        let res = execute(deps.as_mut(), mock_env(), buy_info, buy_msg).unwrap();

        assert_eq!(res.messages.len(), 3); // Ensure there are 3 transfer messages
    }

    #[test]
    fn test_claim() {
        let mut deps = mock_dependencies();

        // Initialize with royalty address
        let init_msg = InstantiateMsg {
            initial_price: Uint128::new(100),
            royalty_address: "royalty_addr".to_string(),
        };
        let init_info = mock_info("creator", &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), init_info, init_msg).unwrap();

        // Simulate buying a plot
        let buy_msg = ExecuteMsg::Buy { coordinates: (0, 0) };
        let buy_info = mock_info("buyer", &coins(200, "sei"));
        let _buy_res = execute(deps.as_mut(), mock_env(), buy_info, buy_msg).unwrap();

        // Simulate claiming funds
        let claim_msg = ExecuteMsg::Claim {};
        let claim_info = mock_info("buyer", &[]);
        let claim_res = execute(deps.as_mut(), mock_env(), claim_info, claim_msg).unwrap();

        assert_eq!(claim_res.messages.len(), 1); // Ensure there is a transfer message
    }
}
