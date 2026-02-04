---
trigger: always_on
---

# Code Style Guide

Language: Rust (Edition 2024)

## Documentation Comments (`///`)
*   **Language**: Dual-language enabled.
*   **Preference**: English first, followed by Chinese.
*   **Format**:
    ```rust
    /// Calculates the edit distance between two strings.
    ///
    /// 计算两个字符串之间的编辑距离。
    pub fn levenshtein_distance(a: &str, b: &str) -> usize { ... }
    ```
*   **Target**: All public modules, structs, traits, and functions.

## Inline Comments (`//`)
*   **Language**: English only.
*   **Purpose**: Explain logic flow, algorithms, or complex implementation details.

## Code Conventions
*   **Formatting**: Use standard `rustfmt`.
*   **Module Structure**: Prohibit the use of `mod.rs`. Use the "folder_name.rs" style (e.g., `src/module.rs` and `src/module/` instead of `src/module/mod.rs`).
*   **Error Handling**: specific typed errors (enum `Error`) rather than strings.
*   **Idioms**: Prefer iterators and functional patterns where readable. Use `Result` and `Option` extensively.