pub mod chat_message;
pub mod meta_data;

#[derive(Debug, PartialEq, Eq)]
struct Emote<'a> {
    id: &'a str,
    indexes: Vec<(u32, u32)>,
}

#[derive(Debug, PartialEq, Eq)]
enum Badges {
    Admin,
    Bits,
    Broadcaster,
    GlobalMod,
    Moderator,
    Subscriber,
    Staff,
    Turbo,
    Premium,
    GlitchCon2020,
    SubGifter,
    Unimplemented,
}

impl From<&str> for Badges {
    fn from(i: &str) -> Self {
        match i.to_lowercase().as_str() {
            "admin" => Badges::Admin,
            "bits" => Badges::Bits,
            "broadcaster" => Badges::Broadcaster,
            "global_mod" => Badges::GlobalMod,
            "moderator" => Badges::Moderator,
            "subscriber" => Badges::Subscriber,
            "staff" => Badges::Staff,
            "turbo" => Badges::Turbo,
            "premium" => Badges::Premium,
            "glitchcon2020" => Badges::GlitchCon2020,
            "sub-gifter" => Badges::SubGifter,
            _ => Badges::Unimplemented,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum MessageType {
    PrivMsg,
}

impl From<&str> for MessageType {
    fn from(i: &str) -> Self {
        match i.to_lowercase().as_str() {
            "privmsg" => MessageType::PrivMsg,
            _ => unimplemented!("wuut?"),
        }
    }
}

#[cfg(test)]
mod test {

    use super::MessageType;

    #[test]
    fn should_give_correct_enum() {
        assert_eq!(MessageType::from("PRIVMSG"), MessageType::PrivMsg);
    }
}
