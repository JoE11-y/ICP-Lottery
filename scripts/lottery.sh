#!/bin/bash
############################################################
# help                                                     #
############################################################
help()
{
   echo "Welcome to the Lottery Canister."
   echo
   echo "functions:"
   echo "1.     deploy <ticket_price> <lottery_duration>"
   echo "2.     run_faucet <account> <amount>"
   echo "3.     get_account_balance"
   echo "4.     start_lottery"
   echo "5.     buy_tickets <lottery_id> <no_of_tickets>"
   echo "6.     end_lottery <lottery_id>"
   echo "7.     check_if_winner <lottery_id>"
   echo "8.     get_lottery_info <lottery_id>"
   echo "9.     get_no_of_tickets <lottery_id> <user>"
   echo "10.    get_lottery_configuration"
   echo "11.    help"
   echo
} 

############################################################
# Main program                                             #
############################################################

deploy(){
    if [ $1 -le 0 ] || [ $2 -ge 60 ]; then
        echo "ERROR: ticket_price must be greater than 0 and lottery_duration must be less than 60"
        exit 1
    fi
    dfx deploy icp_lottery --argument "(record { ticket_price = $1; lottery_duration = $2 })"
}

run_faucet(){
    account=$(dfx identity get-principal)
    current_id=$(dfx identity whoami)
    dfx identity use minter
    dfx canister call icrc1_ledger icrc1_transfer "(record { to = record { owner = principal \"$account\" };  amount = $1; })"
    dfx identity use $current_id
}

get_account_balance(){
    # return wallet balance
    account=$(dfx identity get-principal)
    dfx canister call icrc1_ledger icrc1_balance_of "(record { owner = principal \"$account\" })"
}

start_lottery(){
    dfx canister call icp_lottery start_lottery
}

buy_tickets(){
    if [ $2 -le 0 ]; then
        echo "ERROR: no_of_tickets must be greater than 0"
        exit 1
    fi
    # get canister principal
    canister=$(dfx canister call icp_lottery get_canister_principal)
    # calculate amount to be paid to the canister for tickets
    estimated_amount=$(dfx canister call icp_lottery get_estimated_amount "(record { no_of_tickets = $2 })")
    # approve the canister to be able to spend amount from user account
    dfx canister call icrc1_ledger icrc2_approve "(record { amount = $estimated_amount; spender = record{ owner = $canister } })"
    # call the buy lottery canister function
    dfx canister call icp_lottery buy_tickets "(record { lottery_id = $1; no_of_tickets = $2 })"
}

end_lottery(){
    # call the end lottery canister function
    dfx canister call icp_lottery end_lottery "(record {lottery_id = $1})"
}

check_if_winner(){
    # call the end lottery canister function
    dfx canister call icp_lottery check_if_winner "(record {lottery_id = $1})"
}

get_lottery_info(){
    # call the end lottery canister function
    dfx canister call icp_lottery get_lottery_info "(record {lottery_id = $1})"
}

get_lottery_configuration(){
    dfx canister call icp_lottery get_lottery_configuration
}

get_no_of_tickets(){
    account=$(dfx identity get-principal)
    # call the end lottery canister function
    dfx canister call icp_lottery get_no_of_tickets "(record { lottery_id = $1; user = principal \"$account\" })"
}


"$@"