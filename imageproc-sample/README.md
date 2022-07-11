# imageproc sample

Wasmで画像にフィルタをかけるサンプル。

## build

```
cargo wasi build --release
```

## how it use

Wasmtimeから呼び出せる。

example:

```
wasmtime target/wasm32-wasi/release/imageproc-sample.wasm --dir . -- sample.img
```

## notes

2022年7月11日現在、wasmedgeからも呼び出しが可能だが、wasmtimeと比較して非常に遅い。
原因不明。
