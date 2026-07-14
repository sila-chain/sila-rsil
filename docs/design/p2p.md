# P2P

> Explanation of the [Rsil P2P Stack](../../crates/net/p2p) design process

* Our initial design exploration started in [#64](https://github.com/sila-chain/sila-rsil/issues/64), which focused on layering dependent subprotocols as generic async streams, then using those streams to construct higher level network abstractions.
* Following the above design, we then implemented `P2PStream` and `SilStream`, corresponding to the `p2p` and `sil` subprotocol respectively.
* The wire protocol used to decode messages in `SilStream` came from ethp2p, making it easy to get the full stack to work.
