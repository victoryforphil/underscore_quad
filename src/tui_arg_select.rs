use std::io::IsTerminal;

use anyhow::{bail, Result};

use crate::picker::{run_picker, PickerItem, PickerOutcome};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgSelectOption {
    pub label: String,
    pub value: String,
}

impl ArgSelectOption {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredArgSelectConfig {
    pub arg_name: String,
    pub title: String,
    pub cli_example: String,
    pub cancelled_message: String,
}

impl RequiredArgSelectConfig {
    pub fn new(
        arg_name: impl Into<String>,
        title: impl Into<String>,
        cli_example: impl Into<String>,
    ) -> Self {
        Self {
            arg_name: arg_name.into(),
            title: title.into(),
            cli_example: cli_example.into(),
            cancelled_message: "selection cancelled".to_owned(),
        }
    }

    pub fn with_cancelled_message(mut self, message: impl Into<String>) -> Self {
        self.cancelled_message = message.into();
        self
    }
}

pub fn select_required_arg(
    config: RequiredArgSelectConfig,
    options: Vec<ArgSelectOption>,
) -> Result<String> {
    if !stdio_is_tty() {
        bail!(
            "{} argument is required in non-interactive mode; pass it explicitly\nExample: {}",
            config.arg_name,
            config.cli_example
        );
    }

    if options.is_empty() {
        bail!("no available values to select for `{}`", config.arg_name);
    }

    let items: Vec<PickerItem> = options
        .into_iter()
        .map(|opt| PickerItem {
            label: opt.label,
            key: opt.value,
        })
        .collect();

    match run_picker(&config.title, items)? {
        PickerOutcome::Selected(value) => Ok(value),
        PickerOutcome::Cancelled => bail!(config.cancelled_message),
    }
}

fn stdio_is_tty() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}
