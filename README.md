<h1>Mazzaroth</h1>

Mazzaroth是一个去中心化、具备高吞吐量和具备智能合约功能并且ZK（Zero-knowledge）友好的公共区块链平台，基于BlockDag。其核心组件是Mazzaroth Virtual Machine，MVM，它是一个图灵完备的虚拟机，能够执行智能合约代码，在设计上先天对Zero-knowledg友好并且践行函数式思想，脱离冯诺伊曼架构影响的形式化虚拟机。Mazzaroth使用名为Mth的加密货币作为其内部交易的“燃料”，用于支付交易费用和计算服务。

在MVM上存储数据和规则是被鼓励的，Mazzaroth的本质就是达成去中心化的共识和规则，Mazzaroth的代币Mth本质也只是一种满足货币转移规则的共识数据而已。可以存储的数据包括但不限于你最想要曝光的数据、你喜欢的一部律法、一部普通的电影，没有除你以外的任何人可以从Mazzaroth上删除它。当然存储数据需要支付一次性的费用，以及质押足够的Mth用于换取存储空间。这通常会非常昂贵，因为共识本身就弥足珍贵的。通常来讲我们只存储重要数据的hash到VM上以达成共识。

在相较于传统的区块链vm设计，MVM的状态根支持向前和向后推导状态根，可以通过当前区块和当前vm，推导之前的区块的状态根。这意味着当矿工需要加入Mazzaroth时，他不需要从创始节点开始同步所有区块，只需要同步某个时间节点的MVM。因为Mazzaroth可以沿着区块向前和向后验证的。Mazzaroth向矿工保证，MVM占用的存储空间永远不会超过2TB。

## Mazzaroth的价值观
Mazzaroth是一个赋权任何人构建可靠且高效去中心化共识的系统。为了达到这个目标，我们将这个目标拆解为几个原则
1. Mazzaroth是去中心化的：没有人可以控制Mazzaroth。任何功能支持和用户体验都不能凌驾于去中心化之上
2. Mazzaroth是图灵完全的：理论上你可以在Mazzaroth上达成任何共识
3. Mazzaroth是高效的：惯用功能高效运行，Mazzaroth通常能在10s内确认一笔交易
4. Mazzaroth是多种支持的：Mazzaroth会积极的接入、兼容、支持任何其他的区块链系统，包括但不限于eth，uniswap等

我们总是希望，给定任一设计都能满足（或者不违背）这里的原则。但通常这些原则并不正交，甚至相互掣肘(in tension)。于是些时候我们可能就会在选择其中一个原则时，被迫违背了另一个原则——这种情况下，倾向于选择优先级更高的原则。Mazzaroth的功能总是在这样多方权衡与斗争的情况下设计出来的（最好的情况当然是找到克服这些权衡的“第三条路”），这便是Mazzaroth的灵魂所在。

## Mazzaroth的计划
1. 实现支付功能并进行测试，预期TPS>1000，确认时间小于10S
2. 实现zk友好的形式化虚拟机并上线测试网络
3. 制定经济策略并上线正式网络
4. 实现基于Mazzaroth的去中心化社交网络，MBlog，后续对Mazzaroth的所有提案讨论将会迁移到MBlog。
5. 实现基于Mazzaroth的去中心化代码托管平台Mhub。Mazzaroth代码维护将会迁移至Mhub，至此Mazzaroth完全自举，再也没有任何中心化机构可以对mazzaroth构成绝对威胁
6. 实现Mazzaroth Rollup、跨链桥等众多功能的支持


## 运行Mazzaroth测试
```
# 模拟共识头的生成
cargo run --bin simulation_consensus --release 
```
```
#模拟mvm压力测试
cargo run --bin simulation_mvm --release
```

## 理论研究
https://arxiv.org/abs/2506.01960