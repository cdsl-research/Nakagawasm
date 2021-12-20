use std::env;
use std::error::Error;
use wasmtime::*;

fn main() -> Result<(), Box<dyn Error>> {
    // An engine stores and configures global compilation settings like
    // optimization level, enabled wasm features, etc.
    let engine = Engine::default();

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("args");
        return Ok(());
    }
    let fname = args[1].as_str();

    // We start off by creating a `Module` which represents a compiled form
    // of our input wasm module. In this case it'll be JIT-compiled after
    // we parse the text format.
    let module = Module::from_file(&engine, fname)?;

    // A `Store` is what will own instances, functions, globals, etc. All wasm
    // items are stored within a `Store`, and it's what we'll always be using to
    // interact with the wasm world. Custom data can be stored in stores but for
    // now we just use `()`.
    let mut store = Store::new(&engine, ());

    // With a compiled `Module` we can then instantiate it, creating
    // an `Instance` which we can actually poke at functions on.
    let instance = Instance::new(&mut store, &module, &[])?;

    // The `Instance` gives us access to various exported functions and items,
    // which we access here to pull out our `answer` exported function and
    // run it.
    let answer = instance
        .get_func(&mut store, "_start")
        .expect("`_start` was not an exported function");

    // There's a few ways we can call the `answer` `Func` value. The easiest
    // is to statically assert its signature with `typed` (in this case
    // asserting it takes no arguments and returns one i32) and then call it.
    answer.call(&mut store, &[], &mut []).unwrap();
    // let answer = answer.typed::<(), i32, _>(&store)?;

    // And finally we can call our function! Note that the error propagation
    // with `?` is done to handle the case where the wasm function traps.
    // let result = answer.call(&mut store, ())?;
    // println!("Answer: {:?}", result);
    Ok(())
}
