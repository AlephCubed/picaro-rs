# Picaro Rust

> [!caution]
> This implementation was made by a student for a university project.
> It should ***not*** be considered secure in any way.
>
> **Use at your own risk!**

> [!warning]
> This implementation is low-level, and is difficult to use securely.

This is an attempt at an implementation of the [Picaro](https://ia.cr/2012/358) cipher, for a university project.

```rust
use picaro_rs::Picaro;

let key = 1234;
let plaintext = 1024;

// The number of shares is provided via const generic, in this case 4.
let mut cipher = Picaro::<4 >::new_from_seed(key, 12345);
let ciphertext = cipher.encrypt(plaintext);

assert_ne!(plaintext, ciphertext);

let result = cipher.decrypt(ciphertext);

assert_eq!(plaintext, result);
```

## Current Status

The project includes a theoretically fully working Picaro implementation, with support for an arbitrary level of
masking. The masked level is provided via a constant generic which specifies the number of shares. So for `d` level
masking, `SHARE_COUNT` should be set to `d + 1`.

The project also includes a masked implementation of the AES S-box, and benchmarks used to compare the two.