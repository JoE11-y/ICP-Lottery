use candid::Principal;
use icrc_ledger_types::icrc1::transfer::NumTokens;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LotteryState {
    Active,
    Payout,
    Closed,
}

pub type Salt = [u8; 32];

#[derive(candid::CandidType, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConfState {
    Inactive,
    Active,
}

impl Default for ConfState {
    fn default() -> Self {
        ConfState::Inactive
    }
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
pub struct TransferPayload {
    pub to: Principal,
    pub amount: NumTokens,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
pub struct LotteryConf {
    pub next_lottery_id: u32,
    pub ticket_price: NumTokens,
    pub lottery_duration: u64,
    pub lottery_pool: NumTokens,
    pub state: ConfState,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
pub struct InitArgs {
    pub ticket_price: NumTokens,
    pub lottery_duration: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
pub struct BuyTicketArgs {
    pub lottery_id: u32,
    pub no_of_tickets: u32,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
pub struct QueryArgs {
    pub lottery_id: u32,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
pub struct UserArgs {
    pub lottery_id: u32,
    pub user: Principal,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
pub struct TicketQuery {
    pub no_of_tickets: u32,
}