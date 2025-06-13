If you run into borrow checker issues in Dioxus, try the following. If the variable is a component prop, you can wrap it in a non-mutable `use_signal()` which will resolve the borrow checker issue. Prefer to shadow the original variable if possible:

```rust
#[component]
fn PatientView(id: String) -> Element {
    let id = use_signal(|| id);
    ...
}
```

If the variable is created inside the `rsx!{}` block for example in a `for` loop, the issue often arises due to use in an event handler. In this case you can rewrite the event handler as a block that first clones the variable and then passes it into a closure:

```rust
onchange: {
    let value = value.clone();
    move |_| {
        // Use `value` here
    }
}
```
