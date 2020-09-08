# spin_on

This crate contains what aims to be the simplest possible implementation of a valid executor.
Instead of nicely parking the thread and waiting for the future to wake it up, it continuously
polls the future until the future is ready. This will probably use a lot of CPU, so be careful
when you use it.

The advantages of this crate are:

- It is really simple
- It has no dependency on `std` or on an allocator
- It only has one dependency
