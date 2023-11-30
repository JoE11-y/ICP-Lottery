#[macro_use]
extern crate serde;

mod service;
mod types;

use crate::service::*;
use crate::types::*;
use candid::{Nat, Principal};
use ic_cdk::api::call::CallResult;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::NumTokens;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use icrc_ledger_types::icrc2::transfer_from::{TransferFromArgs, TransferFromError};
use std::cell::RefCell;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static CONF_STORAGE: RefCell<LotteryConf>= RefCell::default();

    static LOTTERY_STORAGE: RefCell<StableBTreeMap<u32, Lottery, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
    ));
}

#[ic_cdk::init]
fn init(args: InitArgs) {
    CONF_STORAGE.with(|conf| conf.borrow_mut().init(args));
}

#[ic_cdk::update]
fn start_lottery() -> Result<(), String> {
    // check general state to make sure a new lottery can be started
    CONF_STORAGE
        .with(|conf| conf.borrow().check_state(ConfState::Inactive))
        .expect("Cannot start Lottery now");

    let lottery_instance = CONF_STORAGE.with(|conf| conf.borrow_mut().gen_lottery());

    insert_lottery(&lottery_instance);
    Ok(())
}

#[ic_cdk::update]
async fn get_estimated_amount(args: TicketQuery)-> NumTokens {
     // get estimated amount to be paid using number of tickets
    let amount =
     CONF_STORAGE.with(|conf| conf.borrow().calc_ticket_price(&args.no_of_tickets));

    amount + _get_ledger_fee().await.clone()
} 

#[ic_cdk::update]
async fn buy_tickets(args: BuyTicketArgs) -> Result<(), String> {
    match LOTTERY_STORAGE.with(|lottery| lottery.borrow().get(&args.lottery_id)) {
        Some(mut lottery) => {
            // check if lottery time has elapsed
            lottery.check_lottery_ongoing().expect("Lottery ended");

            // check lottery state
            lottery
                .check_state(LotteryState::Active)
                .expect("Cannot buy tickets as lottery is not active");

            // get total amount
            let amount =
                CONF_STORAGE.with(|conf| conf.borrow().calc_ticket_price(&args.no_of_tickets));

            // transfer the funds to canister and fail if error
            _do_transfer_to_canister(&amount)
                .await
                .map_err(|e| format!("failed to call ledger: {:?}", e))?
                .map_err(|e| format!("ledger transfer error {:?}", e))
                .expect("ERROR:");

            // increment lottery pool
            CONF_STORAGE.with(|conf| conf.borrow_mut().increment_pool(&amount));

            // register player tickets
            lottery.register_tickets(&args.no_of_tickets);
            insert_lottery(&lottery);
            Ok(())
        }
        None => Err(format!("Invalid lottery id")),
    }
}

#[ic_cdk::update]
async fn end_lottery(args: QueryArgs) -> Result<(), String> {
    match LOTTERY_STORAGE.with(|lottery| lottery.borrow().get(&args.lottery_id)) {
        Some(mut lottery) => {
            // check if lottery time has elapsed
            lottery
                .check_lottery_ended()
                .expect("Lottery still ongoing");

            // check lottery state
            lottery
                .check_state(LotteryState::Active)
                .expect("Cannot end lottery has lottery is not active");

            lottery
                .get_winning_ticket()
                .await
                .expect("Error in random generator");

            // reset general state to inactive
            CONF_STORAGE.with(|conf| conf.borrow_mut().reset_state());

            insert_lottery(&lottery);
            Ok(())
        }
        None => Err(format!("Invalid lottery id")),
    }
}

#[ic_cdk::update]
async fn check_if_winner(args: QueryArgs) -> Result<(), String> {
    match LOTTERY_STORAGE.with(|lottery| lottery.borrow().get(&args.lottery_id)) {
        Some(mut lottery) => {
            // check if lottery time has elapsed
            lottery
                .check_lottery_ended()
                .expect("Lottery still ongoing");

            // check lottery state
            lottery
                .check_state(LotteryState::Payout)
                .expect("Lottery not yet in payout state");

            // check if winner
            lottery.check_winner().expect("payout error");

            // calculate prize money
            let prize = CONF_STORAGE.with(|conf| conf.borrow().get_prize());

            // transfer the funds from canister back to user
            _transfer(&prize)
                .await
                .map_err(|e| format!("failed to call ledger: {:?}", e))?
                .map_err(|e| format!("ledger transfer error {:?}", e))
                .expect("ERROR:");

            // decrement prize pool
            CONF_STORAGE.with(|conf| conf.borrow_mut().decrement_pool(&prize));

            insert_lottery(&lottery);
            Ok(())
        }
        None => Err(format!("Invalid lottery id")),
    }
}

#[ic_cdk::query]
fn get_lottery_info(args: QueryArgs) -> Result<LotteryData, String> {
    let lottery = LOTTERY_STORAGE.with(|lottery| lottery.borrow().get(&args.lottery_id));

    match lottery {
        Some(data) => Ok(data.get_lottery()),
        None => return Err(format!("Invalid lottery id")),
    }
}

#[ic_cdk::query]
fn get_lottery_configuration() -> Result<LotteryConf, String> {
    let config = CONF_STORAGE.with(|conf: &RefCell<LotteryConf>| conf.borrow().get_configuration());
    Ok(config)
}

#[ic_cdk::query]
fn get_no_of_tickets(args: UserArgs) -> Result<u32, String> {
    match LOTTERY_STORAGE.with(|lottery| lottery.borrow().get(&args.lottery_id)) {
        Some(lottery) => Ok(lottery.get_player_ticket_count(&args.user)),
        None => Err(format!("Invalid lottery id")),
    }
}

#[ic_cdk::query]
fn get_canister_principal() -> Principal {
    ic_cdk::id()
}

#[ic_cdk::query]
fn get_caller_principal() -> Principal {
    ic_cdk::caller()
}

////////////////////////////////////////// HELPER FUNCTIONS ////////////////////////////////////////////////////////

// helper to add new lottery instance
fn insert_lottery(lottery: &Lottery) {
    LOTTERY_STORAGE.with(|service| service.borrow_mut().insert(lottery.id, lottery.clone()));
}

// helper method to transfer tokens from ledger to canister
async fn _do_transfer_to_canister(
    amount: &NumTokens,
) -> CallResult<Result<Nat, TransferFromError>> {
    let ledger_id = Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap();
    // The request object of the `icrc1_name` endpoint is empty.

    let args = TransferFromArgs {
        spender_subaccount: None,
        from: Account {
            owner: ic_cdk::caller(),
            subaccount: None,
        },
        to: Account {
            owner: ic_cdk::id(),
            subaccount: None,
        },
        fee: None,
        created_at_time: None,
        memo: None,
        amount: amount.clone(),
    };
    let (result,): (Result<Nat, TransferFromError>,) =
        ic_cdk::call(ledger_id, "icrc2_transfer_from", (args,)).await?;

    Ok(result)
}

// a helper method to transfer the reward amount to creator
async fn _transfer(amount: &NumTokens) -> CallResult<Result<Nat, TransferError>> {
    let ledger_id = Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap();
    // The request object of the `icrc1_name` endpoint is empty.

    let args = TransferArg {
        from_subaccount: None,
        to: Account {
            owner: ic_cdk::caller(),
            subaccount: None,
        },
        fee: None,
        created_at_time: None,
        memo: None,
        amount: amount.clone() - _get_ledger_fee().await.clone(),
    };
    let (result,): (Result<Nat, TransferError>,) =
        ic_cdk::call(ledger_id, "icrc1_transfer", (args,)).await?;

    Ok(result)
}

// helper function to get transfer fee from ledger
async fn _get_ledger_fee() -> Nat {
    let ledger_id = Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap();
    // The request object of the `icrc1_name` endpoint is empty.
    let req = ();
    let (res,): (Nat,) = ic_cdk::call(ledger_id, "icrc1_fee", (req,)).await.unwrap();
    res
}

// need this to generate candid
ic_cdk::export_candid!();
