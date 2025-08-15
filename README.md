<h1>Mazzaroth</h1>

Mazzaroth is a decentralized, high-throughput, smart-contract-enabled, and Zero-knowledge (ZK)-friendly public blockchain platform based on BlockDag. Its core component is the Mazzaroth Virtual Machine (MVM), which is a Turing-complete virtual machine capable of executing smart contract code. By design, MVM is inherently ZK-friendly and embraces functional programming principles, deviating from the influence of the von Neumann architecture. Mazzaroth utilizes a cryptocurrency called Mth as the “fuel” for its internal transactions, which is used to pay for transaction fees and computational services.

## The Values of Mazzaroth
Mazzaroth is a system that empowers anyone to build a reliable and efficient decentralized consensus. To achieve this goal, we have broken it down into several principles:
1. **Mazzaroth is Decentralized**: No one can control Mazzaroth. Decentralization takes precedence over any functional support or user experience.
2. **Mazzaroth is Turing-Complete**: In theory, any consensus can be achieved on Mazzaroth.
3. **Mazzaroth is Efficient**: Common functions operate efficiently, and Mazzaroth can typically confirm a transaction within 10 seconds.
4. **Mazzaroth is Interoperable**: Mazzaroth actively integrates with, is compatible with, and supports any other blockchain systems, including but not limited to Ethereum, Uniswap, etc.

We always strive to ensure that any given design meets (or at least does not violate) these principles. However, these principles are not always orthogonal and can often be in tension with one another. In some cases, we may be forced to prioritize one principle over another when they conflict— in such situations, we tend to choose the principle with higher priority. The functionality of Mazzaroth is always designed amidst these multifaceted trade-offs and struggles (the best-case scenario, of course, is to find a “third way” to overcome these trade-offs), and this is the essence of Mazzaroth.

Storing data and rules on the MVM is encouraged. The essence of Mazzaroth is to achieve decentralized consensus and rules. The native token of Mazzaroth, Mth, is essentially a type of consensus data that satisfies the rules of monetary transfer. The data that can be stored includes, but is not limited to, the data you most want to expose, a law you like, or an ordinary movie. No one other than yourself can delete it from Mazzaroth. However, storing data requires a one-time fee and staking a sufficient amount of Mth to exchange for storage space. This is usually very expensive, as consensus itself is extremely valuable. Generally speaking, we only store the hash of important data on the VM to achieve consensus.

Compared to traditional blockchain VM designs, the state root of the MVM supports both forward and backward derivation of state roots. It is possible to derive the state roots of previous blocks based on the current block and the current VM. This means that when a miner wants to join the Mazzaroth network, they do not need to synchronize all blocks starting from the genesis node. Instead, they only need to synchronize the MVM at a specific point in time, as Mazzaroth allows for both forward and backward validation along the blocks. Mazzaroth guarantees to miners that the storage space occupied by the MVM will never exceed 2TB.

## Mazzaroth's Roadmap
white paper: https://mazzaroth.gitbook.io/mazzaroth-white-paper

1. **Genesis-Implement and Test Payment Functionality**: Achieve a target throughput of over 1,000 transactions per second (TPS) with confirmation times under 10 seconds.

2 **Ouroboros-Implement State Root to write blocks**

3. **Prometheus-Develop and Launch a ZK-Friendly Formal Virtual Machine**

4. **Midas-Formulate Economic Policies and Launch the Main Network**: Establish the economic framework.

5. **Heracles-Develop a Decentralized Social Network on Mazzaroth (MBlog)**: Migrate all future proposal discussions related to Mazzaroth to MBlog.

6. **Dionysus-Create a Decentralized Code Hosting Platform (Mhub)**: Move the maintenance of Mazzaroth's codebase to Mhub, completing the self-bootstrapping process and ensuring that no centralized entity can pose an absolute threat to Mazzaroth.

7. **Penelope-Support for Mazzaroth Rollup and Cross-Chain Bridges**: Implement and integrate these advanced functionalities to enhance the platform's capabilities.


## Running Mazzaroth Simulations
1. **Simulating Consensus Head Generation**
To simulate the generation of the consensus head in Mazzaroth, you can use the following command:
```
cargo run --bin simulation_consensus --release 
```
2. **Simulating MVM Stress Test**
To simulate stress tests on the Mazzaroth Virtual Machine (MVM), you can use the following command:
```
cargo run --bin simulation_mvm --release
```

## Theoretical Research and Further Reading
https://arxiv.org/abs/2506.01960

## Discord
https://discord.gg/J2svr2gQ