use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchHistoryOptions {
    #[serde(default)]
    pub from_time: Option<u64>,

    #[serde(default = "default_forward")]
    pub forward: i64,

    #[serde(default = "default_backward")]
    pub backward: i64,

    #[serde(default)]
    pub backward_time: i64,

    #[serde(default)]
    pub forward_time: i64,

    #[serde(default = "default_get_chat")]
    pub get_chat: bool,

    #[serde(default = "default_item_type")]
    pub item_type: ItemType,

    #[serde(default = "default_get_messages")]
    pub get_messages: bool,

    #[serde(default)]
    pub interactive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ItemType {
    Regular,
    Delayed,
}

fn default_forward() -> i64 { 0 }
fn default_backward() -> i64 { 40 }
fn default_get_chat() -> bool { false }
fn default_get_messages() -> bool { true }
fn default_item_type() -> ItemType { ItemType::Regular }

impl Default for FetchHistoryOptions {
    fn default() -> Self {
        Self {
            from_time: None,
            forward: default_forward(),
            backward: default_backward(),
            backward_time: 0,
            forward_time: 0,
            get_chat: default_get_chat(),
            item_type: default_item_type(),
            get_messages: default_get_messages(),
            interactive: false,
        }
    }
}

impl FetchHistoryOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_time(mut self, v: u64) -> Self {
        self.from_time = Some(v);
        self
    }

    pub fn forward(mut self, v: i64) -> Self {
        self.forward = v;
        self
    }

    pub fn backward(mut self, v: i64) -> Self {
        self.backward = v;
        self
    }

    pub fn interactive(mut self, v: bool) -> Self {
        self.interactive = v;
        self
    }

    pub fn item_type(mut self, v: ItemType) -> Self {
        self.item_type = v;
        self
    }
}
