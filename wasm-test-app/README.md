# wasm test app

実験に用いるWebAssemblyアプリケーション群．
様々なメモリ使用量の傾向を再現することが目的．

## How to Build

予め下記コマンドで`wasm32-wasi` がターゲットのツールチェーンを追加しておく必要がある．

```shell
rustup target add wasm32-wasi
```

下記コマンドの実行により，`wasm32-wasi` バイナリが生成される．
生成先は，`target/wasm32-wasi/debug/` である．

```shell
cargo build --target=wasm32-wasi
```

## Details

<dl>
    <dt>a.rs</dt>
    <dd>メモリ使用量が増加し続ける．</dd>
    <dt>b.rs</dt>
    <dd>メモリリークによってメモリ使用量が増加し続ける．a.rsと見かけ上はまったく同一になるはずである．</dd>
</dl>
