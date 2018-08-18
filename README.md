readwrite
---


Given two things, one of which implements `std::io::Read` and other implements `std::io::Write`, make a single socket-like object which implmenets `Read + Write`. Note that you can't write to it while waiting for data to come from read part.

Example: generate a virtual socketpair.

```rust
fn main() {
    extern crate pipe;
    extern crate readwrite;

    let (r1,w1) = pipe::pipe();
    let (r2,w2) = pipe::pipe();
    let (s1,s2) = (ReadWrite::new(r1,w2), ReadWrite::new(r2,w1));
}
```
