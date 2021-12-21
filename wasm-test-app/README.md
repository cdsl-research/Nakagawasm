# wasm test app

実験に用いるWebAssemblyアプリケーション群．
様々なメモリ使用量の傾向を再現することが目的．

## Details

<dl>
    <dt>a.rs</dt>
    <dd>メモリ使用量が増加し続ける．</dd>
    <dt>b.rs</dt>
    <dd>メモリリークによってメモリ使用量が増加し続ける．a.rsと見かけ上はまったく同一になるはずである．</dd>
</dl>
