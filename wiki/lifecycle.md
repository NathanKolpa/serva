# Life cycle

A typical serva program looks like this:

```rust
fn main() {
    // setup    
    while let Some(request) = Request::next() {
        handle_request(request);
    }
    // tear down
}
```