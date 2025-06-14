# frege
A lightweight, async HTTP router for Rust with middleware support.

## Features
- Async handlers and middleware
- Flexible middleware system with tuple syntax: `.middlewares((m1, m2, m3))`
- CRUD resource registration
- Built on Hyper

## Installation
```toml
[dependencies]
frege = "0.1"