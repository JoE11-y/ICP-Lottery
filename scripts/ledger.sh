#!/usr/bin/env bash

dfx identity use minter
export MINTER_ACCOUNT=$(dfx identity get-principal)

dfx identity use archive_controller
export ARCHIVE_CONTROLLER=$(dfx identity get-principal)

dfx identity use default
export DEFAULT_ACCOUNT=$(dfx identity get-principal)

TOKEN="Lottery Token"                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       
TOKEN_SYMBOL="LT"

PRE_MINTED_TOKENS=100_000
TRANSFER_FEE=1_000

TRIGGER_THRESHOLD=2000
NUM_OF_BLOCK_TO_ARCHIVE=1000
CYCLE_FOR_ARCHIVE_CREATION=10000000000000
FEATURE_FLAGS=true

dfx deploy icrc1_ledger --specified-id mxzaz-hqaaa-aaaar-qaada-cai --argument "(variant {Init = 
record {
     token_symbol = \"${TOKEN_SYMBOL}\";
     token_name = \"${TOKEN_NAME}\";
     minting_account = record { owner = principal \"${MINTER_ACCOUNT}\" };
     transfer_fee = ${TRANSFER_FEE};
     metadata = vec {};
     feature_flags = opt record{icrc2 = ${FEATURE_FLAGS}};
     initial_balances = vec { record { record { owner = principal \"${DEFAULT_ACCOUNT}\"; }; ${PRE_MINTED_TOKENS}; }; };
     archive_options = record {
         num_blocks_to_archive = ${NUM_OF_BLOCK_TO_ARCHIVE};
         trigger_threshold = ${TRIGGER_THRESHOLD};
         controller_id = principal \"${ARCHIVE_CONTROLLER}\";
         cycles_for_archive_creation = opt ${CYCLE_FOR_ARCHIVE_CREATION};
     };
 }
})"