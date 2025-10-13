use std::fmt::Display;

use colored::{
    Color,
    Colorize,
};

enum Message {
    Info,
    Success,
    Warning,
    Error,
}

fn log(msg_ty: Message, label: impl Display, msg: impl Display) {
    println!(
        "{} {} — {}",
        msg_ty.prefix(),
        label.to_string().white(),
        msg.to_string().bright_black()
    );
}

impl Message {
    fn prefix(&self) -> String {
        match self {
            Self::Info => "[INFO]".color(CustomColor::Info).to_string(),
            Self::Success => "[SUCCESS]".color(CustomColor::Highlight).to_string(),
            Self::Warning => "[WARNING]".color(CustomColor::Warning).to_string(),
            Self::Error => "[ERROR]".color(CustomColor::Error).to_string(),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
enum CustomColor {
    Highlight,
    Debug,
    Error,
    Warning,
    Header,
    Info,
    Gray,
    FadedGray,
}

#[rustfmt::skip]
mod unformatted {
    use super::*;

    pub fn log_info(label: impl Display, msg: impl Display) { log(Message::Info, label, msg) }
    pub fn log_success(label: impl Display, msg: impl Display) { log(Message::Success, label, msg) }
    pub fn log_warning(label: impl Display, msg: impl Display) { log(Message::Warning, label, msg) }
    pub fn log_error(label: impl Display, msg: impl Display) { log(Message::Error, label, msg) }

    impl From<CustomColor> for Color {
        fn from(value: CustomColor) -> Color {
            match value {
                CustomColor::Highlight  => Color::TrueColor { r: 255, g: 215, b: 87  },
                CustomColor::Debug      => Color::TrueColor { r: 135, g: 255, b: 135 },
                CustomColor::Error      => Color::TrueColor { r: 255, g: 0,   b: 95  },
                CustomColor::Warning    => Color::TrueColor { r: 215, g: 135, b: 0   },
                CustomColor::Header     => Color::TrueColor { r: 0,   g: 255, b: 0   },
                CustomColor::Info       => Color::TrueColor { r: 0,   g: 95,  b: 255 },
                CustomColor::Gray       => Color::TrueColor { r: 192, g: 192, b: 192 },
                CustomColor::FadedGray  => Color::TrueColor { r: 95,  g: 95,  b: 95  },
            }
        }
    }
}

pub use unformatted::*;
