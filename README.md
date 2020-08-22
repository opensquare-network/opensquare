# opensquare

<table cellspacing="0" cellpadding="0" style="border: none;">
  <tr>
    <td><img src="/logo.png"  alt="1"></td>
    <td><img src="/docs/web3_foundation_grants_badge_black.png" alt="1"></td>
   </tr>
</table>

OpenSquare Network aim to build a blockchain based collaboration and reputation platform, while it will support funders
and hunters to collaborate on bounties, with the Council to deal with the disputes.

Users' reputation and skill proof will be built by their daily activities.

## Build and Run
this project need rust tool chain, install rust from this [link](https://www.rust-lang.org/learn/get-started)

if you ensure rust tool chain is ready, do follow to init environment.
1. init environment
    ```bash
    > git clone https://github.com/opensquare-network/opensquare.git
    > cd opensquare/scripts
    > bash init.sh
    ```
2. compile:
    ```bash
    > WASM_BUILD_TYPE=release cargo build
    > # or we advice you to set `WASM_BUILD_TYPE=release` in your global environment variables, so that you do not need to put every time before `cargo`
    > # if you need build release type, do follow
    > WASM_BUILD_TYPE=release cargo build --release
    ```
3. run:

following parameters means:
* `--dev`: means start with dev mode, would provide default private key to start the chain, and running a independent node.
* `-d <directory>`: means blockchain database stored in the `<directory>`, if you what to clear all data, just stop the
node and delete this directory. If you do not use this parameter, the node would use default directory to store data.
* `--execution=<STRATEGY>`: substrate provide `Native` and `WASM` execution strategy. for test and develop, we suggest to
use `Native`

you could launch node with following commands:

1. run dev mode:

    dev mode provide default private key `Alice` to run a single node
    ```bash
    ./target/debug/opensquare --dev -d .sub --execution=Native
    ```
2. local testnet mode:
    1. run tow nodes in a single machine

        run alice:
        ```bash
        ./target/debug/opensquare --chain=local -d .alice --name=alice --alice --execution=Native
        ```
        run bob
        ```bash
        ./target/debug/opensquare --chain=local -d .bob --name=bob --bob --execution=Native
        ```
    2. run tow nodes in different machine under a LAN
        run alice in one machine:
        ```bash
        ./target/debug/opensquare --chain=local -d .alice --name=alice --alice --execution=Native --ws-external --rpc-external --rpc-cors=all
        ```
        notice if not point `--port`, it would use default `30333` as p2p port.

        and lookup logs, could find a log like
        ```bash
        Local node identity is: 12D3KooWC6ojeA28QDf2GBupCWbqsc2W8JwFUy6GW9Zjwoppz1wb (legacy representation: QmUaXtahadUKyosAnpdefPRdxM3CkHeb9uh6QZW6hNQcPz)
        ```
        pick up the identity `12D3KooWC6ojeA28QDf2GBupCWbqsc2W8JwFUy6GW9Zjwoppz1wb` and assemble it like:
        ```bash
        /ip4/<alice run machine ip in LAN>/tcp/<alice node p2p port>/p2p/12D3KooWC6ojeA28QDf2GBupCWbqsc2W8JwFUy6GW9Zjwoppz1wb
        # like
        # /ip4/192.168.122.1/tcp/30333/p2p/12D3KooWC6ojeA28QDf2GBupCWbqsc2W8JwFUy6GW9Zjwoppz1wb
        ```
        this string would be the bootnode for bob.

        and run bob in another machine:
        ```bash
        ./target/debug/opensquare --chain=local -d .bob --name=bob --bob --execution=Native --ws-external --rpc-external --rpc-cors=all --bootnode=<bootnode above>
        ```

        bootnode could get from another way: call rpc method `system_localListenAddresses` for alice, and would list like:
        ```json
        {
            "jsonrpc": "2.0",
            "result": [
                "/ip4/127.0.0.1/tcp/30333/p2p/12D3KooWC6ojeA28QDf2GBupCWbqsc2W8JwFUy6GW9Zjwoppz1wb",
                "/ip4/10.0.0.9/tcp/30333/p2p/12D3KooWC6ojeA28QDf2GBupCWbqsc2W8JwFUy6GW9Zjwoppz1wb",
                "/ip4/192.168.122.1/tcp/30333/p2p/12D3KooWC6ojeA28QDf2GBupCWbqsc2W8JwFUy6GW9Zjwoppz1wb",
                "/ip4/172.17.0.1/tcp/30333/p2p/12D3KooWC6ojeA28QDf2GBupCWbqsc2W8JwFUy6GW9Zjwoppz1wb",
                "/ip6/::1/tcp/30333/p2p/12D3KooWC6ojeA28QDf2GBupCWbqsc2W8JwFUy6GW9Zjwoppz1wb",
                "/ip4/127.0.0.1/tcp/30333/p2p/12D3KooWC6ojeA28QDf2GBupCWbqsc2W8JwFUy6GW9Zjwoppz1wb",
                "/ip4/10.0.0.9/tcp/30333/p2p/12D3KooWC6ojeA28QDf2GBupCWbqsc2W8JwFUy6GW9Zjwoppz1wb",
                "/ip4/192.168.122.1/tcp/30333/p2p/12D3KooWC6ojeA28QDf2GBupCWbqsc2W8JwFUy6GW9Zjwoppz1wb",
                "/ip4/172.17.0.1/tcp/30333/p2p/12D3KooWC6ojeA28QDf2GBupCWbqsc2W8JwFUy6GW9Zjwoppz1wb",
                "/ip6/::1/tcp/30333/p2p/12D3KooWC6ojeA28QDf2GBupCWbqsc2W8JwFUy6GW9Zjwoppz1wb"
            ],
            "id": 100
        }
        ```
        just pick up suitable bootnode for bob.

## License
[Apache 2.0](LICENSE)

## Acknowledgements

This project is supported by a [Web3 Foundation grant](https://web3.foundation/grants/).

We would also like to thank the following teams for their support:

* [ChainX](https://www.chainx.org)
