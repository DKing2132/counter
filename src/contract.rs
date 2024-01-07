#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    to_binary, BankMsg, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, entry_point,
};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{Plot, PLOTS, TREASURY, POINTS, TOTAL_POINTS, ROYALTY_ADDRESS, INITIAL_PRICE};

// setting a version for the contract
const CONTRACT_NAME: &str = "crates.io:sei-land";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    deps.api.debug("So far, so good!");

    let initial_price = msg.initial_price;
    INITIAL_PRICE.save(deps.storage, &initial_price)?;

    let royalty_addr = deps.api.addr_validate(&msg.royalty_address)?;
    ROYALTY_ADDRESS.save(deps.storage, &royalty_addr)?;

    TREASURY.save(deps.storage, &Uint128::zero())?;
    TOTAL_POINTS.save(deps.storage, &Uint128::zero())?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
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
    env: Env,
    info: MessageInfo,
    coordinates: (i32, i32),
) -> Result<Response, ContractError> {
    deps.api.debug("This is a debug message");

    let mut response = Response::new();

    let sei_amount = info.funds.iter().find(|coin| coin.denom == "sei").map_or(Uint128::zero(), |coin| coin.amount);
    deps.api.debug("So far, the buyer has sent some sei");

    // Log the sei amount sent by the buyer
    let sei_log = format!("SEI amount sent by buyer: {}", sei_amount);
    response = response.add_attribute("sei_amount", sei_log);

    let treasury_addr = match TREASURY.load(deps.storage) {
        Ok(addr) => {
            deps.api.debug(&format!("Treasury address loaded: {}", addr));
            addr
        },
        Err(e) => {
            deps.api.debug(&format!("Failed to load Treasury address: {}", e));
            return Err(ContractError::CustomError { val: "Failed to load Treasury address".to_string() });
        }
    };

    let royalty_addr = match ROYALTY_ADDRESS.load(deps.storage) {
        Ok(addr) => {
            deps.api.debug(&format!("Royalty address loaded: {}", addr));
            addr
        },
        Err(e) => {
            deps.api.debug(&format!("Failed to load Royalty address: {}", e));
            return Err(ContractError::CustomError { val: "Failed to load Royalty address".to_string() });
        }
    };

    deps.api.debug("So far so good, the treasury is initialized");

    let plot_result = PLOTS.may_load(deps.storage, coordinates.clone());
    let mut plot = match plot_result {
        Ok(Some(existing_plot)) => existing_plot,
        Ok(None) => {
            // Plot does not exist, create a new one with initial price
            let initial_price = INITIAL_PRICE.load(deps.storage)?;
            Plot {
                coordinates,
                price: initial_price,
                owner: info.sender.clone(), // Consider if this is the correct logic for your use case
            }
        },
        Err(_) => {
            // Error loading plot data
            return Err(ContractError::CustomError { val: "Failed to load plot".to_string() });
        }
    };

    deps.api.debug("So far so good, the plot coordinates are valid");

    let sell_price = plot.price;
    let treasury_amount = sell_price * Uint128::new(5) / Uint128::new(100);
    let royalty_amount = sell_price * Uint128::new(5) / Uint128::new(100);
    let owner_amount = sell_price - treasury_amount - royalty_amount;
    // Log amounts being transferred
    let amounts_log = format!("Treasury Amount: {}, Royalty Amount: {}, Owner Amount: {}", treasury_amount, royalty_amount, owner_amount);
    response = response.add_attribute("amounts_being_transferred", amounts_log);


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

    // Add details to the response
    response = response.add_attribute("action", "buy")
        .add_attribute("buyer", info.sender.to_string())
        .add_attribute("seller", plot.owner.to_string())
        .add_attribute("coordinates", format!("{},{}", coordinates.0, coordinates.1))
        .add_attribute("sell_price", sell_price.to_string());

    // Create transfer messages if the buyer is not the current owner
    deps.api.debug("So far so good, the transfer messages are being created");
    if plot.owner != info.sender {
        deps.api.debug("The buyer is not the owner, so transfer messages will be created");
        let messages = vec![
            BankMsg::Send {
                to_address: treasury_addr.to_string(),  // Need to change to address of the SC
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

    // Log the updated plot details
    let updated_plot_log = format!("Updated plot: ({}, {}) with new price: {} and new owner: {}", coordinates.0, coordinates.1, sell_price * Uint128::new(2), info.sender);
    response = response.add_attribute("updated_plot_details", updated_plot_log);

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



