//! Cursor 版本信息管理模块
//!
//! 本模块使用 MaybeUninit 来存储版本信息，这种设计考虑了以下因素：
//! 1. 版本信息在程序生命周期内只需初始化一次
//! 2. 避免使用 lazy_static 或 once_cell 等额外依赖
//! 3. 性能考虑：避免每次访问时的同步开销
//!
//! # Safety
//!
//! 安全性保证：
//! - 初始化函数 `initialize_cursor_version` 必须在程序启动时的单线程环境中调用
//! - 必须且只能调用一次 `initialize_cursor_version`
//! - 初始化后，所有访问都是只读的（通过 clone() 返回副本）
//! - 虽然 `bytes::Bytes` 本身是线程安全的（使用原子操作的引用计数），
//!   但由于 Rust 2024 edition 的限制，我们仍需要 `#[allow(static_mut_refs)]`

use ::core::mem::MaybeUninit;

// 定义所有常量
crate::define_typed_constants! {
    &'static str => {
        /// 默认的客户端版本号
        DEFAULT_CLIENT_VERSION = "1.0.0",
        /// 环境变量名：Cursor 客户端版本
        ENV_CURSOR_CLIENT_VERSION = "CURSOR_CLIENT_VERSION",
        /// Chrome 版本信息
        CHROME_VERSION_INFO = " Chrome/132.0.6834.210 Electron/34.3.4 Safari/537.36",
        /// Windows User-Agent 前缀
        #[cfg(windows)]
        UA_PREFIX_WINDOWS = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/",
        /// Unix/macOS User-Agent 前缀
        #[cfg(unix)]
        UA_PREFIX_UNIX = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/",
        /// 默认的 Windows User-Agent
        #[cfg(windows)]
        DEFAULT_UA_WINDOWS = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/1.0.0 Chrome/132.0.6834.210 Electron/34.3.4 Safari/537.36",
        /// 默认的 Unix/macOS User-Agent
        #[cfg(unix)]
        DEFAULT_UA_UNIX = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/1.0.0 Chrome/132.0.6834.210 Electron/34.3.4 Safari/537.36",
    }

    usize => {
        /// 版本字符串最小长度
        VERSION_MIN_LENGTH = 5,
        /// 版本字符串最大长度
        VERSION_MAX_LENGTH = 32,
        /// StringBuilder 初始容量
        STRING_BUILDER_CAPACITY = 3,
    }

    u8 => {
        /// 版本号中每个部分的最大数字位数
        VERSION_PART_MAX_DIGITS = 4,
        /// 版本号中期望的点号数量
        VERSION_DOT_COUNT = 2,
    }
}

/// 客户端版本的 HeaderValue
static mut CLIENT_VERSION: MaybeUninit<http::header::HeaderValue> = MaybeUninit::uninit();

/// Cursor User-Agent 的 HeaderValue
static mut HEADER_VALUE_UA_CURSOR_LATEST: MaybeUninit<http::header::HeaderValue> =
    MaybeUninit::uninit();

/// 获取 Cursor 客户端版本的 HeaderValue
///
/// # Safety
///
/// 调用者必须确保 `initialize_cursor_version` 已经被调用。
/// `#[allow(static_mut_refs)]` 是必需的，因为 `HeaderValue` 内部使用 `bytes::Bytes`。
/// 尽管 `Bytes` 的 clone 操作是线程安全的（使用原子引用计数），
/// 但 Rust 的借用检查器无法验证这一点。
#[allow(static_mut_refs)]
#[inline(always)]
pub fn cursor_client_version() -> http::header::HeaderValue {
    unsafe { CLIENT_VERSION.assume_init_ref().clone() }
}

/// 获取 Cursor 用户代理的 HeaderValue
///
/// # Safety
///
/// 调用者必须确保 `initialize_cursor_version` 已经被调用。
/// `#[allow(static_mut_refs)]` 是必需的，因为 `HeaderValue` 内部使用 `bytes::Bytes`。
/// 尽管 `Bytes` 的 clone 操作是线程安全的（使用原子引用计数），
/// 但 Rust 的借用检查器无法验证这一点。
#[allow(static_mut_refs)]
#[inline(always)]
pub fn header_value_ua_cursor_latest() -> http::header::HeaderValue {
    unsafe { HEADER_VALUE_UA_CURSOR_LATEST.assume_init_ref().clone() }
}

/// 初始化 Cursor 的版本信息
///
/// # Safety
///
/// 此函数必须满足以下条件：
/// 1. 在程序启动时的单线程环境中调用
/// 2. 在整个程序生命周期中只能调用一次
/// 3. 必须在调用 `cursor_client_version` 或 `header_value_ua_cursor_latest` 之前调用
pub fn initialize_cursor_version() {
    use ::core::ops::Deref as _;

    let version = crate::common::utils::parse_from_env(
        ENV_CURSOR_CLIENT_VERSION,
        DEFAULT_CLIENT_VERSION,
    );

    // 验证版本格式
    validate_version_string(&version);

    let version_header = match http::header::HeaderValue::from_str(&version) {
        Ok(header) => header,
        Err(_) => {
            __cold_path!();
            __eprintln!("Error: Invalid version string for HTTP header");
            // 使用默认版本
            http::header::HeaderValue::from_static(DEFAULT_CLIENT_VERSION)
        }
    };

    use crate::common::utils::string_builder::StringBuilder;

    // 构建 User-Agent 字符串
    #[cfg(windows)]
    let (ua_string, default_ua) = {
        let ua = StringBuilder::with_capacity(STRING_BUILDER_CAPACITY)
            .append(UA_PREFIX_WINDOWS)
            .append(version.deref())
            .append(CHROME_VERSION_INFO)
            .build();
        (ua, DEFAULT_UA_WINDOWS)
    };

    #[cfg(unix)]
    let (ua_string, default_ua) = {
        let ua = StringBuilder::with_capacity(STRING_BUILDER_CAPACITY)
            .append(UA_PREFIX_UNIX)
            .append(version.deref())
            .append(CHROME_VERSION_INFO)
            .build();
        (ua, DEFAULT_UA_UNIX)
    };

    let ua_header = match http::header::HeaderValue::from_str(&ua_string) {
        Ok(header) => header,
        Err(_) => {
            __cold_path!();
            __eprintln!("Error: Invalid user agent string for HTTP header");
            // 使用默认 UA
            http::header::HeaderValue::from_static(default_ua)
        }
    };

    #[allow(static_mut_refs)]
    unsafe {
        CLIENT_VERSION.write(version_header);
        HEADER_VALUE_UA_CURSOR_LATEST.write(ua_header);
    }
}

/// 检查版本字符串是否符合 VSCode/Cursor 的版本格式
///
/// 期望的格式：`major.minor.patch`
/// 例如：`1.0.0`、`1.95.3`
///
/// # Returns
///
/// 如果版本格式有效返回 `true`，否则返回 `false`
#[inline]
pub const fn is_valid_version_format(version: &str) -> bool {
    // 快速路径：检查基本长度要求
    if version.len() < VERSION_MIN_LENGTH || version.len() > VERSION_MAX_LENGTH {
        return false;
    }

    let bytes = version.as_bytes();
    let mut dot_count = 0u8;
    let mut digit_count = 0u8;
    let mut i = 0;

    // 解析 major.minor.patch 部分
    while i < bytes.len() {
        match bytes[i] {
            b'0'..=b'9' => {
                digit_count += 1;
                // 防止数字部分过长
                if digit_count > VERSION_PART_MAX_DIGITS {
                    return false;
                }
            }
            b'.' => {
                // 点号前必须有数字
                if digit_count == 0 {
                    return false;
                }
                dot_count += 1;
                if dot_count > VERSION_DOT_COUNT {
                    return false;
                }
                digit_count = 0;
            }
            _ => return false,
        }
        i += 1;
    }

    // 必须正好有两个点号，且最后一部分有数字
    dot_count == VERSION_DOT_COUNT && digit_count > 0
}

/// 验证并警告无效的版本字符串
///
/// 如果版本字符串不符合格式，打印警告信息但不终止程序
#[inline]
pub fn validate_version_string(version: &str) {
    if !is_valid_version_format(version) {
        __cold_path!();
        use crate::common::utils::string_builder::StringBuilder;
        let warning = StringBuilder::with_capacity(STRING_BUILDER_CAPACITY)
            .append("Warning: Invalid version format '")
            .append(version)
            .append("'. Expected format: major.minor.patch (e.g., 1.0.0)")
            .build();
        __eprintln!(&warning);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_version_formats() {
        assert!(is_valid_version_format("1.0.0"));
        assert!(is_valid_version_format("1.95.3"));
        assert!(is_valid_version_format("10.20.30"));
        assert!(is_valid_version_format("1234.5678.9012"));
    }

    #[test]
    fn test_invalid_version_formats() {
        assert!(!is_valid_version_format("1.0"));
        assert!(!is_valid_version_format("1.0.0.0"));
        assert!(!is_valid_version_format("v1.0.0"));
        assert!(!is_valid_version_format(".1.0.0"));
        assert!(!is_valid_version_format("1..0"));
        assert!(!is_valid_version_format(""));
        assert!(!is_valid_version_format("1.0."));
        assert!(!is_valid_version_format("10000.0.0")); // 超过4位数字
        assert!(!is_valid_version_format("1")); // 太短
        assert!(!is_valid_version_format(&"1.0.".repeat(20))); // 太长
    }
}
