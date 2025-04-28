[] get_part_sort应该是一个可以校验区块是否正确的函数，如果是正确的，则计算出part sort。然后由外层函数将数据入库。而不是现在这样用于计算局部排序。check_block_and_gen_part_sort
[] 应该有一个函数专门用作矿工生成区块，形状跟get_part_sort一样，gen_block_data
[] 如果在计算lca的时候，不计算所有tips只计算well connect，那么矿工对与网络情况变差就会变得难以感觉
[] 如果在计算lca的时候，计算所有tips，那么机会出现恶意攻击的机会，恶意矿工就会连接创世节点，是的lca变得非常大，