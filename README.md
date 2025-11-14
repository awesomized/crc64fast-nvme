# ⚠️ DEPRECATED - No Longer Maintained

This project is no longer under active development and has been archived.

**We recommend using [crc-fast-rust](https://github.com/awesomized/crc-fast-rust) instead**, which offers better performance, more features, and active maintenance.

---

crc64fast-nvme
=========

[![Build status](https://github.com/awesomized/crc64fast-nvme/workflows/Rust/badge.svg)](https://github.com/awesomized/crc64fast-nvme/actions?query=workflow%3ARust)
[![Latest Version](https://img.shields.io/crates/v/crc64fast-nvme.svg)](https://crates.io/crates/crc64fast-nvme)
[![Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/crc64fast-nvme)

SIMD-accelerated carryless-multiplication [CRC-64/NVME](https://reveng.sourceforge.io/crc-catalogue/all.htm#crc.cat.crc-64-nvme) checksum computation
(similar to [crc32fast](https://crates.io/crates/crc32fast) and forked from [crc64fast](https://github.com/tikv/crc64fast) which calculates [CRC-64/XZ](https://reveng.sourceforge.io/crc-catalogue/all.htm#crc.cat.crc-64-xz) [a.k.a `CRC-64/GO-ECMA`]).

`CRC-64/NVME` comes from the [NVM Express® NVM Command Set Specification](https://nvmexpress.org/wp-content/uploads/NVM-Express-NVM-Command-Set-Specification-1.0d-2023.12.28-Ratified.pdf) (Revision 1.0d, December 2023) and has also been implemented in the [Linux kernel](https://github.com/torvalds/linux/blob/786c8248dbd33a5a7a07f7c6e55a7bfc68d2ca48/lib/crc64.c#L66-L73) (where it's called `CRC-64/Rocksoft`) and is [AWS S3's recommended checksum option](https://docs.aws.amazon.com/AmazonS3/latest/userguide/checking-object-integrity.html) as `CRC64-NVME`. (Note that the Check value in the spec uses incorrect endianness [Section 5.2.1.3.4, Figure 120, page 83]).

SIMD-accelerated carryless-multiplication is based on the Intel [Fast CRC Computation for Generic Polynomials Using PCLMULQDQ Instruction](https://web.archive.org/web/20131224125630/https://www.intel.com/content/dam/www/public/us/en/documents/white-papers/fast-crc-computation-generic-polynomials-pclmulqdq-paper.pdf) paper.

## Changes

See [CHANGELOG](CHANGELOG.md).

## Changes from [crc64fast](https://github.com/tikv/crc64fast)

Primarily changes the `CRC-64/XZ` (aka `CRC-64/GO-ECMA`) polynomial from [crc64fast](https://github.com/tikv/crc64fast) (which uses the `ECMA-182` polynomial [`0x42F0E1EBA9EA3693`]) to use the `NVME` polynomial (`0xAD93D23594C93659`), plus re-calculates the input parameters (tables, keys, mu, and reciprocal polynomial) for fast operations.

## Usage

### Rust

```rust
use crc64fast_nvme::Digest;

let mut c = Digest::new();
c.write(b"hello ");
c.write(b"world!");
let checksum = c.sum64();
assert_eq!(checksum, 0xd9160d1fa8e418e3);
```

### C-compatible shared library
`cargo build` will produce a shared library target (`.so` on Linux, `.dll` on Windows, `.dylib` on macOS, etc) and `crc64vnme.h` header file for use in non-Rust projects, such as through FFI.

There is a [crc-fast-php](https://github.com/awesomized/crc-fast-php) library using it with PHP, for example.

```php
/** \FFI $ffi */

$digest = $ffi->digest_new();
$ffi->digest_write($digest, 'hello world!', 12);
$checksum = $ffi->digest_sum64($digest); // 0xd9160d1fa8e418e3
```

## CLI example
A simple CLI implementation can be found in [crc_64_nvme_checksum.rs](src\bin\crc_64_nvme_checksum.rs), which will calculate the `CRC-64/NVME` checksum for a file on disk.

## Other CRC-64 implementations
Tooling to re-calculate input parameters for other `CRC-64` implementations/polynomials is supplied in [src\bin](src\bin).

## Performance

`crc64fast-nvme` provides two fast implementations, and the most performance one will
be chosen based on CPU feature at runtime.

* a fast, platform-agnostic table-based implementation, processing 16 bytes at a time.
* a SIMD-carryless-multiplication based implementation on modern processors:
    * using PCLMULQDQ + SSE 4.1 on x86/x86_64
    * using PMULL + NEON on AArch64 (64-bit ARM)

| Algorithm                   | Throughput (x86_64) | Throughput (aarch64) |
|:----------------------------|--------------------:|---------------------:|
| [crc 3.0.1]                 |           0.5 GiB/s |            0.3 GiB/s |
| crc64fast-nvme (table)      |           2.3 GiB/s |            1.8 GiB/s |
| crc64fast-nvme (SIMD)       |          28.2 GiB/s |           20.0 GiB/s |
| crc64fast-nvme (VPCLMULQDQ) |            52 GiB/s |                 n/a  |

[crc 3.0.1]: https://docs.rs/crc/3.0.1/crc/index.html

## Experimental "Vector Carry-Less Multiplication of Quadwords" (VPCLMULQDQ) support

Using Rust's support for [AVX512 intrinsics](https://github.com/rust-lang/rust/issues/111137), specifically [VPCLMULQDQ](https://doc.rust-lang.org/src/core/stdarch/crates/core_arch/src/x86/vpclmulqdq.rs.html), we can massively improve throughput for x86_64 processors which support them (Intel Ice Lake+ and AMD Zen4+).

Specifically, on an `m7i.8xlarge` EC2 instance (4th gen Xeon, aka Sapphire Rapids), throughput approximately _doubles_ from ~26GiB/s to ~52GiB/s.

Since these are currently marked as unstable features in Rust, you'll need to build with `nightly` and enable the `vpclmulqdq` feature:

``` 
rustup toolchain install nightly
cargo +nightly build --features="vpclmulqdq" -r
```

## References

* [crc32-fast](https://crates.io/crates/crc32fast) - Original `crc32` implementation in Rust.
* [crc64-fast](https://github.com/tikv/crc64fast) - Original `CRC-64/XZ` implementation in Rust (from which this project was forked).
* [Fast CRC Computation for Generic Polynomials Using PCLMULQDQ Instruction](https://web.archive.org/web/20131224125630/https://www.intel.com/content/dam/www/public/us/en/documents/white-papers/fast-crc-computation-generic-polynomials-pclmulqdq-paper.pdf) - Intel's paper.
* [NVM Express® NVM Command Set Specification](https://nvmexpress.org/wp-content/uploads/NVM-Express-NVM-Command-Set-Specification-1.0d-2023.12.28-Ratified.pdf) - The NVMe spec, including `CRC-64-NVME` (with incorrect endian Check value).
* [CRC-64/NVME](https://reveng.sourceforge.io/crc-catalogue/all.htm#crc.cat.crc-64-nvme) - The `CRC-64/NVME` quick definition.
* [Linux implementation](https://github.com/torvalds/linux/blob/786c8248dbd33a5a7a07f7c6e55a7bfc68d2ca48/lib/crc64.c) - Linux implementation of `CRC-64/NVME`.
* [C++ artifacts implementation](https://github.com/jeffareid/crc/blob/master/crc64r/crc64rg.cpp) - Inspiration C++ for the Rust code in [calculate_pclmulqdq_artifacts.rs](src\bin\calculate_pclmulqdq_artifacts.rs).
* [Intel isa-l GH issue #88](https://github.com/intel/isa-l/issues/88) - Additional insight into generating artifacts.
* [StackOverflow PCLMULQDQ CRC32 answer](https://stackoverflow.com/questions/71328336/fast-crc-with-pclmulqdq-not-reflected/71329114#71329114) - Insightful answer to implementation details for CRC32.
* [StackOverflow PCLMULQDQ CRC32 question](https://stackoverflow.com/questions/21171733/calculating-constants-for-crc32-using-pclmulqdq) - Insightful question & answer to CRC32 implementation details.
* [AWS S3 announcement about CRC64-NVME support](https://aws.amazon.com/blogs/aws/introducing-default-data-integrity-protections-for-new-objects-in-amazon-s3/)
* [AWS S3 docs on checking object integrity using CRC64-NVME](https://docs.aws.amazon.com/AmazonS3/latest/userguide/checking-object-integrity.html)
* [Vector Carry-Less Multiplication of Quadwords (VPCLMULQDQ) details](https://en.wikichip.org/wiki/x86/vpclmulqdq)

## License

`crc64fast-nvme` is dual-licensed under

* Apache 2.0 license ([LICENSE-Apache](./LICENSE-Apache) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](./LICENSE-MIT) or <https://opensource.org/licenses/MIT>)
