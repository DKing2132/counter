// src/contract.rs
use cosmwasm_std::{
    to_binary, BankMsg, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{Plot, PLOTS, TREASURY, POINTS, TOTAL_POINTS, ROYALTY};

// setting a version for the contract
const CONTRACT_NAME: &str = "crates.io:sei-land";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    
    // Initialize treasury and points
    TREASURY.save(deps.storage, &Uint128::zero())?;
    TOTAL_POINTS.save(deps.storage, &Uint128::zero())?;
    
    // Initialize plots with initial price
    for x in 0..100 {
        for y in 0..100 {
            let plot = Plot {
                coordinates: (x, y),
                price: msg.initial_price,
                owner: _info.sender.clone(),
            };
            PLOTS.save(deps.storage, (x, y), &plot)?;
        }
    }

    Ok(Response::new().add_attribute("method", "instantiate"))
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Buy { coordinates } => try_buy(deps, env, info, coordinates),
        ExecuteMsg::Claim {} => try_claim(deps, info),
    }
}

pub fn try_buy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    coordinates: (i32, i32),
) -> Result<Response, ContractError> {
    let plot = PLOTS.load(deps.storage, coordinates.clone())?;
    let sei_amount = info.funds.iter().find(|coin| coin.denom == "sei").map_or(Uint128::zero(), |coin| coin.amount);
    let treasury_addr = TREASURY.load(deps.storage)?;
    let royalty_addr = ROYALTY.load(deps.storage)?;
    
    // Check if the sent amount of SEI is sufficient
    if sei_amount < plot.price {
        return Err(ContractError::InsufficientFunds {});
    }

    let sell_price = plot.price;
    let treasury_amount = sell_price * Uint128::new(5) / Uint128::new(100);
    let royalty_amount = sell_price * Uint128::new(5) / Uint128::new(100);
    let owner_amount = sell_price - treasury_amount - royalty_amount;

    // Update treasury balance
    TREASURY.update(deps.storage, |balance| -> StdResult<_> {
        Ok(balance + treasury_amount)
    })?;

    // Update points for the buyer
    POINTS.update(deps.storage, info.sender.clone(), |points| -> StdResult<_> {
        Ok(points.unwrap_or_default() + Uint128::new(100))
    })?;

    // Update total points issued
    TOTAL_POINTS.update(deps.storage, |total| -> StdResult<_> {
        Ok(total + Uint128::new(100))
    })?;

    // Update plot details
    PLOTS.save(deps.storage, coordinates.clone(), &Plot {
        coordinates,
        price: sell_price * Uint128::new(2),
        owner: info.sender.clone(),
    })?;

    // Prepare the response
    let mut response = Response::new()
        .add_attribute("action", "buy")
        .add_attribute("buyer", info.sender.to_string())
        .add_attribute("seller", plot.owner.to_string())
        .add_attribute("coordinates", format!("{},{}", coordinates.0, coordinates.1))
        .add_attribute("sell_price", sell_price.to_string());

    // Conditionally create transfer messages if the buyer is not the current owner
    if plot.owner != info.sender {
        let messages = vec![
            BankMsg::Send {
                to_address: treasury_addr.to_string(),
                amount: vec![Coin { denom: "sei".to_string(), amount: treasury_amount }],
            },
            BankMsg::Send {
                to_address: royalty_addr.to_string(),
                amount: vec![Coin { denom: "sei".to_string(), amount: royalty_amount }],
            },
            BankMsg::Send {
                to_address: plot.owner.to_string(),
                amount: vec![Coin { denom: "sei".to_string(), amount: owner_amount }],
            },
        ];
        response = response.add_messages(messages);
    }

    Ok(response)
}


pub fn try_claim(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let points = POINTS.load(deps.storage, info.sender.clone())?;
    let total_points = TOTAL_POINTS.load(deps.storage)?;
    let treasury_balance = TREASURY.load(deps.storage)?;

    let user_share = (points * treasury_balance) / total_points;

    TREASURY.save(deps.storage, &(treasury_balance - user_share))?;
    POINTS.save(deps.storage, info.sender.clone(), &Uint128::zero())?;

    Ok(Response::new()
        .add_attribute("action", "claim")
        .add_attribute("claimer", info.sender.clone())
        .add_attribute("amount_claimed", user_share)
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: "sei".to_string(),
                amount: user_share,
            }],
        }))
}



