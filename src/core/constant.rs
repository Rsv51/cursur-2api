// mod display_name;

use ::ahash::{HashMap, HashSet};
use ::core::{
    borrow::Borrow,
    sync::atomic::{AtomicBool, Ordering},
};
use ::parking_lot::RwLock;
use ::std::{
    sync::Arc,
    time::{Duration, Instant},
};

use super::model::Model;
use crate::{
    app::{constant::UNKNOWN, lazy::get_start_time, model::DateTime},
    leak::manually_init::ManuallyInit,
};

macro_rules! def_pri_const {
    ($($name:ident => $value:expr),+ $(,)?) => {
        $(
            const $name: &'static str = $value;
        )+
    };
}

macro_rules! def_pub_const {
    ($($name:ident => $value:expr),+ $(,)?) => {
        $(
            pub const $name: &'static str = $value;
        )+
    };
}

// 错误信息
def_pub_const!(
    ERR_UNSUPPORTED_GIF => "不支持动态 GIF",
    ERR_UNSUPPORTED_IMAGE_FORMAT => "不支持的图片格式，仅支持 PNG、JPEG、WEBP 和非动态 GIF",
    ERR_NODATA => "No data",
);

// AI 服务商
def_pri_const!(
    ANTHROPIC => "anthropic",
    CURSOR => "cursor",
    GOOGLE => "google",
    OPENAI => "openai",
    DEEPSEEK => "deepseek",
    XAI => "xai",
    MOONSHOTAI => "moonshotai",
    FIREWORKS => "fireworks",
);

macro_rules! def_const_models {
    ($($name:ident => $value:expr),+ $(,)?) => {
        // 定义常量
        $(
            const $name: &'static str = $value;
        )+

        // 生成 PHF map
        static MODEL_MAP: ::phf::Map<&'static str, &'static str> = ::phf::phf_map! {
            $(
                $value => $name,
            )+
        };

        /// 通过 PHF 快速查找模型 ID
        #[inline]
        pub fn get_model_const(id: &str) -> Option<&'static str> {
            MODEL_MAP.get(id).copied()
        }

        /// 获取静态字符串引用，如果不存在则 intern
        #[inline]
        pub fn get_static_id<S: Borrow<str>>(s: S) -> &'static str {
            let id = s.borrow();
            get_model_const(id)
                .unwrap_or_else(|| crate::leak::intern_static(id))
        }
    };
}

// AI 模型
def_const_models!(
    // 默认模型
    DEFAULT => "default",

    // Anthropic 模型
    CLAUDE_4_OPUS_THINKING => "claude-4-opus-thinking",
    CLAUDE_4_OPUS => "claude-4-opus",
    CLAUDE_4_SONNET_THINKING => "claude-4-sonnet-thinking",
    CLAUDE_4_SONNET => "claude-4-sonnet",
    CLAUDE_3_5_SONNET => "claude-3.5-sonnet",
    CLAUDE_3_7_SONNET => "claude-3.7-sonnet",
    CLAUDE_3_7_SONNET_THINKING => "claude-3.7-sonnet-thinking",
    CLAUDE_3_5_HAIKU => "claude-3.5-haiku",

    // Cursor 模型
    CURSOR_SMALL => "cursor-small",

    // Google 模型
    GEMINI_2_5_PRO_PREVIEW_05_06 => "gemini-2.5-pro-preview-05-06",
    GEMINI_2_5_PRO => "gemini-2.5-pro",
    GEMINI_2_5_PRO_LATEST => "gemini-2.5-pro-latest",
    GEMINI_2_5_FLASH_PREVIEW_05_20 => "gemini-2.5-flash-preview-05-20",
    GEMINI_2_5_FLASH => "gemini-2.5-flash",
    GEMINI_2_5_FLASH_LATEST => "gemini-2.5-flash-latest",

    // OpenAI 模型
    O3 => "o3",
    GPT_5 => "gpt-5",
    GPT_5_HIGH => "gpt-5-high",
    GPT_5_LOW => "gpt-5-low",
    GPT_5_FAST => "gpt-5-fast",
    GPT_5_HIGH_FAST => "gpt-5-high-fast",
    GPT_5_LOW_FAST => "gpt-5-low-fast",
    GPT_4_1 => "gpt-4.1",
    GPT_4O => "gpt-4o",
    O4_MINI => "o4-mini",
    O3_PRO => "o3-pro",

    // Deepseek 模型
    DEEPSEEK_R1_0528 => "deepseek-r1-0528",
    DEEPSEEK_V3_1 => "deepseek-v3.1",

    // XAI 模型
    GROK_3_BETA => "grok-3-beta",
    GROK_3 => "grok-3",
    GROK_3_MINI => "grok-3-mini",
    GROK_4 => "grok-4",
    GROK_4_0709 => "grok-4-0709",

    // MoonshotAI 模型
    KIMI_K2_INSTRUCT => "kimi-k2-instruct",
    ACCOUNTS_FIREWORKS_MODELS_KIMI_K2_INSTRUCT => "accounts/fireworks/models/kimi-k2-instruct",

    // Anthropic 模型 (legacy)
    CLAUDE_3_OPUS => "claude-3-opus",

    // Cursor 模型 (legacy)
    CURSOR_FAST => "cursor-fast",

    // Google 模型 (legacy)
    GEMINI_2_5_PRO_EXP_03_25 => "gemini-2.5-pro-exp-03-25",
    GEMINI_2_5_FLASH_PREVIEW_04_17 => "gemini-2.5-flash-preview-04-17",
    GEMINI_2_5_PRO_MAX => "gemini-2.5-pro-max",
    GEMINI_2_0_FLASH_THINKING_EXP => "gemini-2.0-flash-thinking-exp",
    GEMINI_2_0_FLASH => "gemini-2.0-flash",

    // Deepseek 模型 (legacy)
    DEEPSEEK_R1 => "deepseek-r1",
    DEEPSEEK_V3 => "deepseek-v3",

    // OpenAI 模型 (legacy)
    GPT_4 => "gpt-4",
    GPT_4_5_PREVIEW => "gpt-4.5-preview",
    GPT_4_TURBO_2024_04_09 => "gpt-4-turbo-2024-04-09",
    GPT_4O_MINI => "gpt-4o-mini",
    O1_MINI => "o1-mini",
    O1_PREVIEW => "o1-preview",
    O1 => "o1",
    O3_MINI => "o3-mini",

    // XAI 模型 (legacy)
    GROK_2 => "grok-2",

    // 长对话模型 (legacy)
    GPT_4O_128K => "gpt-4o-128k",
    GEMINI_1_5_FLASH_500K => "gemini-1.5-flash-500k",
    CLAUDE_3_HAIKU_200K => "claude-3-haiku-200k",
    CLAUDE_3_5_SONNET_200K => "claude-3.5-sonnet-200k",
);

static INSTANCE: ManuallyInit<RwLock<Models>> = ManuallyInit::new();

macro_rules! create_models {
    ($($owner:ident => [$($model:expr,)+]),* $(,)?) => {
        #[deny(unused)]
        pub fn create_models() {
            // ModelIds 只在这个作用域内有效
            #[derive(Debug, Clone, Copy)]
            struct ModelIds {
                id: &'static str,
                client_id: &'static str,
                server_id: &'static str,
            }

            impl ModelIds {
                const fn new(id: &'static str) -> Self {
                    Self {
                        id,
                        client_id: id,
                        server_id: id,
                    }
                }

                const fn with_client_id(mut self, client_id: &'static str) -> Self {
                    self.client_id = client_id;
                    self
                }

                const fn with_server_id(mut self, server_id: &'static str) -> Self {
                    self.server_id = server_id;
                    self
                }
            }

            let models = vec![
                $($(
                    {
                        #[allow(non_upper_case_globals)]
                        const model_ids: ModelIds = $model;
                        Model {
                            id: model_ids.id,
                            server_id: model_ids.server_id,
                            client_id: model_ids.client_id,
                            owned_by: $owner,
                            is_thinking: SUPPORTED_THINKING_MODELS.contains(&model_ids.id),
                            is_image: SUPPORTED_IMAGE_MODELS.contains(&model_ids.id),
                            is_max: SUPPORTED_MAX_MODELS.contains(&model_ids.id)
                                || MAX_MODELS.contains(&model_ids.id),
                            is_non_max: !MAX_MODELS.contains(&model_ids.id),
                        }
                    },
                )+)*
            ];

            let mut cached_ids = Vec::new();
            for model in &models {
                push_ids(&mut cached_ids, model.id);

                if model.is_max && model.is_non_max {
                    push_ids(&mut cached_ids, crate::leak::intern_static(format!("{}-max", model.id)));
                }
            }
            let find_ids = HashMap::from_iter(models.iter().enumerate().map(|(i, m)| (m.id, i)));

            unsafe {
                INSTANCE.init(RwLock::new(Models {
                    models: Arc::new(models),
                    raw_models: None,
                    cached_ids: Arc::new(cached_ids),
                    find_ids,
                    last_update: Instant::now(),
                }))
            }
        }
    };
}

pub struct Models {
    models: Arc<Vec<Model>>,
    raw_models: Option<Arc<crate::core::aiserver::v1::AvailableModelsResponse>>,
    cached_ids: Arc<Vec<&'static str>>,

    find_ids: HashMap<&'static str, usize>,
    last_update: Instant,
}

impl Models {
    #[inline(always)]
    pub fn get() -> ::parking_lot::RwLockReadGuard<'static, Self> { INSTANCE.read() }

    pub fn to_arc() -> Arc<Vec<Model>> { Self::get().models.clone() }

    pub fn to_raw_arc() -> Option<Arc<crate::core::aiserver::v1::AvailableModelsResponse>> {
        Self::get().raw_models.clone()
    }

    // 克隆所有模型
    // pub fn cloned() -> Vec<Model> {
    //     Self::get().models.as_ref().clone()
    // }

    // 检查模型是否存在
    // pub fn exists(model_id: &str) -> bool {
    //     Self::get().models.iter().any(|m| m.id == model_id)
    // }

    // 查找模型并返回其 ID
    pub fn find_id(id: &str) -> Option<Model> {
        let guard = Self::get();
        guard
            .find_ids
            .get(id)
            .map(|&i| *unsafe { guard.models.get_unchecked(i) })
    }

    // 返回所有模型 ID 的列表
    pub fn ids() -> Arc<Vec<&'static str>> { Self::get().cached_ids.clone() }

    // 写入方法
    pub fn update(
        available_models: crate::core::aiserver::v1::AvailableModelsResponse,
    ) -> Result<(), &'static str> {
        #[allow(non_local_definitions)]
        impl crate::core::aiserver::v1::available_models_response::AvailableModel {
            #[inline(always)]
            fn extract_ids(&self) -> (&'static str, &'static str, &'static str) {
                let id = get_static_id(self.name.as_str());
                let client_id = if let Some(ref client_id) = self.client_display_name
                    && client_id != id
                {
                    get_static_id(client_id.as_str())
                } else {
                    id
                };
                let server_id = if let Some(ref server_id) = self.server_model_name
                    && server_id != id
                {
                    get_static_id(server_id.as_str())
                } else {
                    id
                };
                (id, client_id, server_id)
            }
        }

        if available_models.models.is_empty() {
            return Err("Models list cannot be empty");
        }

        let mut data = INSTANCE.write();
        data.raw_models = Some(Arc::new(available_models.clone()));

        // 检查时间间隔（30分钟）
        if data.last_update.elapsed() < Duration::from_secs(30 * 60) && {
            static FIRST_CHECK_PASSED: AtomicBool = AtomicBool::new(false);

            if FIRST_CHECK_PASSED.load(Ordering::Relaxed) {
                true
            } else {
                let result =
                    DateTime::naive_now() - *get_start_time() >= chrono::TimeDelta::minutes(30);
                if result {
                    FIRST_CHECK_PASSED.store(true, Ordering::Relaxed);
                }
                result
            }
        } {
            return Ok(());
        }

        // 内联辅助函数：将服务器模型转换为内部模型表示
        #[inline]
        fn convert_model(
            model: crate::core::aiserver::v1::available_models_response::AvailableModel,
        ) -> Model {
            let (id, client_id, server_id) = model.extract_ids();
            let owned_by = {
                let mut chars = server_id.chars();
                match chars.next() {
                    Some('g') => match chars.next() {
                        Some('p') => OPENAI, // g + p → "gp" (gpt)
                        Some('e') => GOOGLE, // g + e → "ge" (gemini)
                        Some('r') => XAI,    // g + r → "gr" (grok)
                        _ => UNKNOWN,
                    },
                    Some('o') => match chars.next() {
                        // o 开头需要二次判断
                        Some('1') | Some('3') | Some('4') => OPENAI, // o1/o3/o4 系列
                        _ => UNKNOWN,
                    },
                    Some('c') => match chars.next() {
                        Some('l') => ANTHROPIC, // c + l → "cl" (claude)
                        Some('u') => CURSOR,    // c + u → "cu" (cursor)
                        _ => UNKNOWN,
                    },
                    Some('d') => match chars.next() {
                        Some('e') if chars.next() == Some('e') => DEEPSEEK, // d + e + e → "dee" (deepseek)
                        _ => UNKNOWN,
                    },
                    Some('a') =>
                        if server_id.len() > 26 {
                            FIREWORKS
                        } else {
                            UNKNOWN
                        },
                    // 其他情况
                    _ => UNKNOWN,
                }
            };
            let is_thinking = model.supports_thinking();
            let is_image = if server_id == DEFAULT {
                true
            } else {
                model.supports_images()
            };
            let is_max = model.supports_max_mode();
            let is_non_max = model.supports_non_max_mode();
            // let display_name = display_name::calculate_display_name_v4(id);;

            Model {
                id,
                client_id,
                owned_by,
                server_id,
                is_thinking,
                is_image,
                is_max,
                is_non_max,
            }
        }

        // 先获取当前模型列表的引用
        let current_models = Arc::clone(&data.models);

        // 根据不同的FetchMode来确定如何处理模型
        let new_models: Vec<_> = match crate::app::model::AppConfig::get_fetch_models() {
            crate::app::model::FetchMode::Truncate => {
                // 完全使用新获取的模型列表
                available_models
                    .models
                    .into_iter()
                    .map(convert_model)
                    .collect()
            }
            crate::app::model::FetchMode::AppendTruncate => {
                // 先收集所有在available_models中的模型ID
                let new_model_ids: HashSet<_> = available_models
                    .models
                    .iter()
                    .map(|model| get_static_id(model.name.as_str()))
                    .collect();

                // 保留current_models中不在new_model_ids中的模型
                let mut result: Vec<_> = current_models
                    .iter()
                    .filter(|model| !new_model_ids.contains(&model.id))
                    .cloned()
                    .collect();

                // 添加所有新模型
                result.extend(available_models.models.into_iter().map(convert_model));

                result
            }
            crate::app::model::FetchMode::Append => {
                // 只添加不存在的模型
                let existing_ids: HashSet<_> =
                    current_models.iter().map(|model| model.id).collect();

                // 复制现有模型
                let mut result = current_models.to_vec();

                // 仅添加ID不存在的新模型
                result.extend(
                    available_models
                        .models
                        .into_iter()
                        .filter(|model| !existing_ids.contains(&get_static_id(model.name.as_str())))
                        .map(convert_model),
                );

                result
            }
        };

        // 检查内容是否有变化
        if *data.models == new_models {
            return Ok(());
        }

        // 计算模型变化
        let old_ids: HashSet<_> = data.models.iter().map(|m| m.id).collect();
        let new_ids: HashSet<_> = new_models.iter().map(|m| m.id).collect();

        // 获取需要添加和移除的模型
        let to_add: Vec<_> = new_models
            .iter()
            .filter(|m| !old_ids.contains(&m.id))
            .collect();

        let to_remove: Vec<_> = data
            .models
            .iter()
            .filter(|m| !new_ids.contains(&m.id))
            .collect();

        // 从缓存中移除不再需要的ID
        let mut cached_ids: Vec<_> = data
            .cached_ids
            .iter()
            .filter(|&&id| {
                !to_remove.iter().any(|m| {
                    // 基本ID匹配
                    if id == m.id {
                        return true;
                    }

                    // 处理带有"-online"后缀的情况
                    if let Some(base) = id.strip_suffix("-online") {
                        if base == m.id {
                            return true;
                        }
                        // 处理同时有"-max"和"-online"后缀的情况（即"-max-online"）
                        if let Some(base_without_max) = base.strip_suffix("-max")
                            && base_without_max == m.id
                        {
                            return true;
                        }
                        false
                    }
                    // 处理仅带有"-max"后缀的情况
                    else if let Some(base) = id.strip_suffix("-max") {
                        base == m.id
                    } else {
                        false
                    }
                })
            })
            .copied()
            .collect();

        // 只为新增的模型创建ID组合
        for model in to_add {
            push_ids(&mut cached_ids, model.id);

            if model.is_max && model.is_non_max {
                push_ids(&mut cached_ids, crate::leak::intern_static(format!("{}-max", model.id)));
            }
        }

        // 更新数据和时间戳
        data.find_ids = HashMap::from_iter(new_models.iter().enumerate().map(|(i, m)| (m.id, i)));
        data.models = Arc::new(new_models);
        data.cached_ids = Arc::new(cached_ids);
        data.last_update = Instant::now();

        Ok(())
    }
}

#[inline]
fn push_ids(ids: &mut Vec<&'static str>, id: &'static str) {
    ids.push(id);
    ids.push(crate::leak::intern_static(format!("{id}-online")));
}

create_models! {
    DEFAULT => [
        ModelIds::new("default"),
    ],

    ANTHROPIC => [
        ModelIds::new("claude-4-opus-thinking"),
        ModelIds::new("claude-4-opus"),
        ModelIds::new("claude-4-sonnet-thinking"),
        ModelIds::new("claude-4-sonnet"),
        ModelIds::new("claude-3.5-sonnet"),
        ModelIds::new("claude-3.7-sonnet"),
        ModelIds::new("claude-3.7-sonnet-thinking"),
        ModelIds::new("claude-3.5-haiku"),
        ModelIds::new("claude-3-opus"),
        ModelIds::new("claude-3-haiku-200k"),
        ModelIds::new("claude-3.5-sonnet-200k"),
    ],

    CURSOR => [
        ModelIds::new("cursor-small"),
        ModelIds::new("cursor-fast"),
    ],

    GOOGLE => [
        ModelIds::new("gemini-2.5-pro-preview-05-06")
            .with_client_id("gemini-2.5-pro")
            .with_server_id("gemini-2.5-pro-latest"),
        ModelIds::new("gemini-2.5-pro"),
        ModelIds::new("gemini-2.5-pro-latest"),
        ModelIds::new("gemini-2.5-flash-preview-05-20")
            .with_client_id("gemini-2.5-flash")
            .with_server_id("gemini-2.5-flash-latest"),
        ModelIds::new("gemini-2.5-flash"),
        ModelIds::new("gemini-2.5-flash-latest"),
        ModelIds::new("gemini-2.5-pro-exp-03-25"),
        ModelIds::new("gemini-2.5-flash-preview-04-17"),
        ModelIds::new("gemini-2.5-pro-max"),
        ModelIds::new("gemini-2.0-flash-thinking-exp"),
        ModelIds::new("gemini-2.0-flash"),
        ModelIds::new("gemini-1.5-flash-500k"),
    ],

    OPENAI => [
        ModelIds::new("o3"),
        ModelIds::new("gpt-5"),
        ModelIds::new("gpt-5-high"),
        ModelIds::new("gpt-5-low"),
        ModelIds::new("gpt-5-fast"),
        ModelIds::new("gpt-5-high-fast"),
        ModelIds::new("gpt-5-low-fast"),
        ModelIds::new("gpt-4.1"),
        ModelIds::new("gpt-4o"),
        ModelIds::new("o4-mini"),
        ModelIds::new("o3-pro"),
        ModelIds::new("gpt-4"),
        ModelIds::new("gpt-4.5-preview"),
        ModelIds::new("gpt-4-turbo-2024-04-09"),
        ModelIds::new("gpt-4o-mini"),
        ModelIds::new("o1-mini"),
        ModelIds::new("o1-preview"),
        ModelIds::new("o1"),
        ModelIds::new("o3-mini"),
        ModelIds::new("gpt-4o-128k"),
    ],

    DEEPSEEK => [
        ModelIds::new("deepseek-r1-0528"),
        ModelIds::new("deepseek-v3.1"),
        ModelIds::new("deepseek-r1"),
        ModelIds::new("deepseek-v3"),
    ],

    XAI => [
        ModelIds::new("grok-3-beta")
            .with_client_id("grok-3"),
        ModelIds::new("grok-3"),
        ModelIds::new("grok-3-mini"),
        ModelIds::new("grok-4")
            .with_server_id("grok-4-0709"),
        ModelIds::new("grok-2"),
    ],

    MOONSHOTAI => [
        ModelIds::new("kimi-k2-instruct")
            .with_server_id("accounts/fireworks/models/kimi-k2-instruct"),
    ],
}

pub(super) const FREE_MODELS: [&str; 6] = [
    GPT_4O_MINI,
    CURSOR_FAST,
    CURSOR_SMALL,
    DEEPSEEK_V3,
    DEEPSEEK_V3_1,
    GROK_3_MINI,
];

pub(super) const LONG_CONTEXT_MODELS: [&str; 4] = [
    GPT_4O_128K,
    GEMINI_1_5_FLASH_500K,
    CLAUDE_3_HAIKU_200K,
    CLAUDE_3_5_SONNET_200K,
];

// 支持思考的模型
const SUPPORTED_THINKING_MODELS: [&str; 15] = [
    CLAUDE_4_OPUS_THINKING,
    CLAUDE_4_SONNET_THINKING,
    O3,
    GPT_5,
    GPT_5_HIGH,
    GPT_5_LOW,
    GPT_5_FAST,
    GPT_5_HIGH_FAST,
    GPT_5_LOW_FAST,
    GEMINI_2_5_PRO_PREVIEW_05_06,
    GEMINI_2_5_FLASH_PREVIEW_05_20,
    CLAUDE_3_7_SONNET_THINKING,
    O4_MINI,
    DEEPSEEK_R1_0528,
    GROK_4,
    O3_PRO,
];

// 支持图像的模型（DEFAULT 始终支持）
const SUPPORTED_IMAGE_MODELS: [&str; 24] = [
    DEFAULT,
    CLAUDE_4_OPUS_THINKING,
    CLAUDE_4_OPUS,
    CLAUDE_4_SONNET_THINKING,
    CLAUDE_4_SONNET,
    CLAUDE_3_5_SONNET,
    O3,
    GPT_5,
    GPT_5_HIGH,
    GPT_5_LOW,
    GPT_5_FAST,
    GPT_5_HIGH_FAST,
    GPT_5_LOW_FAST,
    GEMINI_2_5_PRO_PREVIEW_05_06,
    GEMINI_2_5_FLASH_PREVIEW_05_20,
    GPT_4_1,
    CLAUDE_3_7_SONNET,
    CLAUDE_3_7_SONNET_THINKING,
    CLAUDE_3_5_HAIKU,
    GEMINI_2_5_PRO_EXP_03_25,
    GPT_4O,
    O4_MINI,
    GROK_4,
    O3_PRO,
];

// 支持Max与非Max的模型
const SUPPORTED_MAX_MODELS: [&str; 19] = [
    CLAUDE_4_SONNET_THINKING,
    CLAUDE_4_SONNET,
    CLAUDE_3_5_SONNET,
    O3,
    GPT_5,
    GPT_5_HIGH,
    GPT_5_LOW,
    GPT_5_FAST,
    GPT_5_HIGH_FAST,
    GPT_5_LOW_FAST,
    GEMINI_2_5_PRO_PREVIEW_05_06,
    GEMINI_2_5_FLASH_PREVIEW_05_20,
    GPT_4_1,
    CLAUDE_3_7_SONNET,
    CLAUDE_3_7_SONNET_THINKING,
    GEMINI_2_5_PRO_EXP_03_25,
    O4_MINI,
    GROK_3_BETA,
    GROK_4,
];

// 只支持Max的模型
const MAX_MODELS: [&str; 3] = [CLAUDE_4_OPUS_THINKING, CLAUDE_4_OPUS, O3_PRO];
