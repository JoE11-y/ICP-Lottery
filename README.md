# ICP-Lottery

Simple lottery caninster built on the ICP network,

- It allows any player to be able to start the lottery, buy tickets and even end the lottery.
- Players can then check to see if they're the lucky winners of that lottery.
- Winner gets half of the prizepool.

To learn more before you start working with icp_lottery, see the following documentation available online:

- [Quick Start](https://internetcomputer.org/docs/quickstart/quickstart-intro)
- [SDK Developer Tools](https://internetcomputer.org/docs/developers-guide/sdk-guide)
- [Rust Canister Devlopment Guide](https://internetcomputer.org/docs/rust-guide/rust-intro)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://internetcomputer.org/docs/candid-guide/candid-intro)
- [JavaScript API Reference](https://erxue-5aaaa-aaaab-qaagq-cai.raw.icp0.io)

If you want to start working on your project right away, you might want to try the following commands:

```bash
cd icp_lottery/
dfx help
dfx canister --help
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background --clean
```

Next we are going to create the following identities on our dfx instance, these identities make the creation of the ledger seamless. For more information about the identities check the [ICRC1 tutorial](https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/icrc1-ledger-setup)

```bash
# The minter identity
dfx identity new minter

# The archive controller
dfx identity new archive_controller
```

Then we proceed to deploy the ICRC1 Ledger, a script has been supplied for that. This sets up the ledger.

```bash
npm run deploy-ledger
```

Next we move to the lottery script located at `./scripts/lottery.sh`. This script contains all the functions available to the icp lottery canister and their respective implementations.

To view the available functions and arguments they accept, run the lottery help function below:

```bash
npm run lottery help
```

Output

```bash
Welcome to the Lottery Canister.

functions:
1.     deploy <ticket_price> <lottery_duration>
2.     run_faucet <account> <amount>
3.     get_account_balance
4.     start_lottery
5.     buy_tickets <lottery_id> <no_of_tickets>
6.     end_lottery <lottery_id>
7.     check_if_winner <lottery_id>
8.     get_lottery_info <lottery_id>
9.     get_no_of_tickets <lottery_id> <user>
10.    get_lottery_configuration
11.    help
```

To deploy the lottery canister we use the `deploy` function and pass in the ticket_price and the lottery duration(in minutes for the purpose of testing).

```bash
# npm run lottery deploy <ticket_price> <lottery_duration>
npm run lottery deploy 10_000 15
```

To view the set lottery configuration use the `get_lottery_configuration` function. This returns the set ticket price, lottery duration, amount in the pool and the next lottery id.

```bash
npm run lottery get_lottery_configuration
```

Next we are going to try the remaining functions

1. Getting test tokens from the minter account. This sends tokens from ledger to the default account.

    ```bash
    # npm run lottery run_faucet <amount> 
    npm run lottery run_faucet 900_000
    ```

    You can check the amount of tokens received by calling the `get_account_balance` function

    ```bash
    npm run lottery get_account_balance 
    ```

2. Next we start the lottery, no arguments are passed it uses the preset lottery configuration.

    ```bash
    npm run lottery start_lottery 
    ```

    To get the information about a lottery session use the `get_lottery_info` function

    ```bash
    # npm run lottery get_lottery_info <lottery_id>
    npm run lottery get_lottery_info 0
    ```

3. After starting the lottery the next thing is to try to buy tickets by calling the `buy_tickets` function and passing in the lottery id (in this case 0) and the number of tickets.

    ```bash
    # npm run lottery buy_tickets <lottery_id> <no_of_tickets>
    npm run lottery buy_tickets 0 5 
    ```

    This function triggers two function, the approve function which approves the lottery canister to be able to spend the users tokens and the transfer function that transfers the tokens to the lottery canister.

    N/B: Any transaction done  on the ledger costs #1000 tokens, this was set in the ledger.sh script. You can edit that on your own.

4. After the lottery session expires you can then try the `end_lottery` function which ends the lottery and uses the ic_cdk randomness function to select a random ticket to which its owner is assigned as the winner.

    ```bash
    # npm run lottery end_lottery <lottery_id>
    npm run lottery buy_tickets 0
    ```

5. Lastly for the user or winner to get the reward the user has to call the `check_if_winner` function which checks if the user owns the winning the ticket and fails if the user does not.

    ```bash
    # npm run lottery check_if_winner <lottery_id>
    npm run lottery check_if_winner 0
    ```
