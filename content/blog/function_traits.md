+++
title = "Function Traits"
date = 2024-11-19
+++
[[blog]]
[[rust]]

The idea here is that you can have marker traits on functions. Each of these examples will have a follow-up blog post about what problems they solve, but to start, lets illustrate the core value of this concept: The ability to define constraints on a function, and not just data.

### Review: traits used for constraints

Traits are commonly used to constrain what is allowed to be used. For example:

```rust
pub struct Rc<T: ?Sized> {
    // snip
}

// Rc can never be used multi-thread, use Arc instead
impl<T> !Sync for Rc<T> {}
// ...same goes for moving Rc data between threads
impl<T> !Send for Rc<T> {}

let thing = Rc::new(3u32);
// std::thread::spawn(move || println!("use Arc instead: {}", *thing);
let cloned = Rc::clone(&thing);
// std::thread::spawn(|| println!("even if this was mutexed, it's a no-go on thread-sharing. use Arc instead"))
```

So let's have a look at what it might mean if we can apply similar checks at compile time but on functions.

#### Example Syntax

Lets make an illustrative example of what the syntax would look like. We'll keep it similar to the familiar syntax around traits.

To define a trait:
```rust
trait_fn Foo {}
```

and to attach a constraint to a function:
```rust
fn foo_constrained(...)<Foo> {...}
```

and to mark a function as satisfying a trait:

```rust
impl Foo for SomeStruct::foo_fn {}

// and when something is not marked as Foo
// impl Foo for OtherStruct::bar_fn {}
```

...and now, using `foo_constrained` means that you can't accidently add non-Foo functions to a `Foo` constrained call stack:

```rust
fn foo_constrained(good_struct: SomeStrict, bad_struct: OtherStruct)<Foo> {
	// tick emoji
	good_struct.foo_fn();
	// cross emoji!!!
	bad_struct.bar_fn();
	// ^^^ compiler error, 'cannot use non-Foo function in a <Foo> constrained context'
}
```


> NOTE
> I chose `for`, because it's what we are used to. The keyword `on` makes more sense to me, as a function is an action, not a thing. If anyone that knows english grammar rules can help me understand why `on` feels more appropriate than `for` in the context of marking a function instead of a struct, leave a comment, or email me.

So now that we have a basis for how a function can have traits attached to it, let's look at what we can do with it.


#### No panic

the standard `Vec` can panic when an underlying allocation isn't succesfull. this is unacceptable in some circumstances, such as the kernel. Other function properties serve as desirable to either restrict, or enforce, such as recursion depth, memory layout of the function on the stack, and so on.

Let's consider writing rust code that is intended for the kernel. We want a very strict constraint against panics, so we create a `StrictNoPanic` function trait, which causes a compiler error when you try to use the standard `Vec` allocation strategy:

```rust
fn do_thing()<StrictNoPanic> {
	let thing = alloc::collections::Vec::with_capacity(1 << 12);
}
```
^^^ This should compile fail: it's constrained at the function level to be a strictly no-panic scenario, but `Vec::with_capacity` violates this constraint


So how would this be set up? The `alloc_safe` crate provides a replacement for `with_capacity` that returns `Err(AllocError>` instead of triggering a panic:

```rust
use std::alloc::Vec;
use alloc_safe::{AllocError, VecExt};

// Auto-implemented where a panic in the call-stack is detected
fn_trait Panic {}
// Auto-implemented where things like unchecked allocation is detected (e.g. `Vec::with_capacity()`)
fn_trait HiddenPanic {}

fn_trait NoPanic: !Panic {}
fn_trait StrictNoPanic: !Panic + !HiddenPanic {}


fn some_rust_module()<StrictNoPanic> {
	// compiler error: use of function that violates the `StrictNoPanic` constraint
    // let might_panic = Vec::with_capacity(1 << 12);

    let cannot_panic = <Vec as VecExt>::with_capacity(1 << 12);
	let module_allocation = match module_allocation {
		Ok(a) => a,
		Err(alloc_err) => return handle_alloc_err(alloc_err),
	};
}
```

#### Function memory layout constraints

Where the compiler can detect recursion count, this can be used:
```rust
fn_trait RecursionLimit {
	const DEPTH: u32;
}
fn call_deep()<RecursionLimit<const DEPTH: 12>> {
	// compiler can detect the number of recursive calls, decrementing `DEPTH` each time. the last time, DEPTH is 0, making a subsequent call a compiler error
}
```

Where the compiler can detect the memory layout of a function:
```rust
fn constrained_stack<const SIZE: usize>()<StackSizeLimit<2048>> {
	// uses up half the limit
	let array = [u8; 1024];
	// The compiler can detect stack-frame size-limit breaches
	let array2 = [u8; SIZE];
}
```

To illustrate a niche application where you want to optimise a function to be cache friendly for your hardware:
```rust
fn cache_optimised()<StackSizeExact<CACHE_LINE>>
```


#### Pure
A beautiful property of FP is function purity. Being able to strictly reason about side-effects and their nature has a plethora of benefits. Rust can offer approximations of expressing this with **immutable by default** and **const functions**, but there isn't a way to encapsulate the notion of _purity_ within the type system:
```rust
fn pure()<Pure> {
	// println! has side-effects, so cannot be used under the `Pure` constraint
	// println!(...)
}
```
^^^ would raise a warning: Pure functions with no input and/or output are effectively no-ops

In order to write side effects when constrained to be pure, you can define the purity, such that they can be deferred until later

```rust
fn print_shared_pure()<IOPure, SharedMutPur(val: u32)> {
	//println!("this is a side effect")

	// defines the defered evaluation for printing then incrementing
	let print_it = IOPure::do(|val| println!("io side effect: {val}"));

	// defines the defered evaluation for printing then incrementing
	return SharedMut::do({
		print_it(val);
		val += 1;
	});
}
```

#### For sync/async duality

It is a common headache for library writers/users to deal with having both an `async` and a `blocking` implementation. Having a mechanism to tell the compiler how to transform a regular `fn` into an `async fn` would solve these kinds of problems. 

> NOTE
> For the purpose of illustration and brevity, we are going to pretend that `sleep`, and a `Read` on a file are the only things that are relevant to async

Lets use task-sleeping as an e.g.:

```rust
// Both `Async` and non-`Async` implementations of the same function must be defined in order to meet the `MaybeAsync`
fn_trait MaybeAsync: Async, !Async {}
impl MaybeAsync for crate::sleep {

	// tokio::sleep already implements `Async`

	// the <!Async> version of tokio::sleep
	fn sleep(duration: Duration)<!Async> {
		std::thread::sleep(duration)
	}
}

impl MaybeAsync for crate::io::AsyncReadExt::read {
	// the <!Async> version of AsyncReadExt::read
	fn read(&mut self, buf: &mut [u8])<!Async> -> io::Result<usize> {
		std::io::Read::read(&mut self, &mut buf)
	}
}
```


now, if you want to write a library that works in blocking and non-blocking, you don't need this:
```rust
pub mod async_api {
	async fn do_mesage_wait(duration: Duration, msg: &str) {
		tokio::sleep(duration).await;
		
		println!("{}", msg);
	}
}
mod blocking_api {
	pub fn do_message_wait(duration: Duration, msg: &str) {
		std::thread::sleep(duration);
		println!("{}", msg);
	}
}
```
instead no longer having a duplicate api:
```rust
impl MaybeAsyinc for do_message_wait {
	async fn inner_sleep(duration: Duration) {
		tokio::sleep(duration)
	}
	fn inner_sleep(duration: Duration) {
		std::thread::sleep(duration)
	}
	pub fn do_message_wait(duration: Duration, msg: &str) {
		inner_sleep(duration).await
		println!("{}", msg);
}
```
