use chrono::{DateTime, TimeZone, Utc};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric0, alphanumeric1, digit1, multispace0, one_of},
    combinator::{opt, rest},
    error::{context, ErrorKind, VerboseError},
    multi::{many1, separated_list0},
    sequence::{preceded, separated_pair, tuple},
    AsChar, IResult, InputTakeAtPosition,
};

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

type Res<T, U> = IResult<T, U, VerboseError<T>>;

fn alphanumerichyphen1<T>(i: T) -> Res<T, T>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    i.split_at_position1_complete(
        |item| {
            let char_item = item.as_char();
            char_item != '-' && char_item != '_' && char_item != '.' && !char_item.is_alphanum()
        },
        ErrorKind::AlphaNumeric,
    )
}

fn alphanumerichyphenbackslash1<T>(i: T) -> Res<T, T>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    i.split_at_position1_complete(
        |item| {
            let char_item = item.as_char();
            char_item != '-' && char_item != '_' && char_item != '\\' && !char_item.is_alphanum()
        },
        ErrorKind::AlphaNumeric,
    )
}

fn alphanumerichyphencolon1<T>(i: T) -> Res<T, T>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    i.split_at_position1_complete(
        |item| {
            let char_item = item.as_char();
            char_item != '-' && char_item != ':' && char_item != '.' && !char_item.is_alphanum()
        },
        ErrorKind::AlphaNumeric,
    )
}

fn badge_info(input: &str) -> Res<&str, Vec<(Badges, &str)>> {
    context(
        "badge_info",
        preceded(
            opt(alt((tag("@"), tag(";")))),
            separated_pair(
                tag("badge-info"),
                tag("="),
                separated_list0(tag(","), badge_format),
            ),
        ),
    )(input)
    .map(|(next, (_, value))| (next, value))
}
fn badges(input: &str) -> Res<&str, Vec<(Badges, &str)>> {
    context(
        "badges",
        preceded(
            opt(alt((tag("@"), tag(";")))),
            separated_pair(
                tag("badges"),
                tag("="),
                separated_list0(tag(","), badge_format),
            ),
        ),
    )(input)
    .map(|(next, (_, value))| (next, value))
}

fn badge_format(input: &str) -> Res<&str, (Badges, &str)> {
    context(
        "badge_format",
        separated_pair(alphanumerichyphen1, tag("/"), alphanumerichyphenbackslash1),
    )(input)
    .map(|(next, (badge, version))| (next, (badge.into(), version)))
}

fn color(input: &str) -> Res<&str, Option<(u8, u8, u8)>> {
    context(
        "color",
        preceded(
            alt((tag("@"), tag(";"))),
            separated_pair(tag("color"), tag("="), opt(hex_to_rgb)),
        ),
    )(input)
    .map(|(next, (_, value))| (next, value))
}

fn hex_to_rgb(input: &str) -> Res<&str, (u8, u8, u8)> {
    context(
        "parse hex to rgb",
        preceded(tag("#"), many1(one_of("0123456789ABCDEF"))),
    )(input)
    .map(|(next, result)| {
        let r = format!("{}{}", result[0], result[1]);
        let g = format!("{}{}", result[2], result[3]);
        let b = format!("{}{}", result[4], result[5]);

        let r = u8::from_str_radix(r.as_str(), 16).unwrap();
        let g = u8::from_str_radix(g.as_str(), 16).unwrap();
        let b = u8::from_str_radix(b.as_str(), 16).unwrap();

        (next, (r, g, b))
    })
}

fn bits(input: &str) -> Res<&str, u32> {
    context(
        "bits",
        preceded(tag("bits"), separated_pair(tag("bits"), tag("="), digit1)),
    )(input)
    .map(|(next, (_, value))| (next, value.parse::<u32>().unwrap()))
}

fn display_name(input: &str) -> Res<&str, Option<&str>> {
    context(
        "display-name",
        preceded(
            tag(";"),
            separated_pair(tag("display-name"), tag("="), opt(alphanumerichyphen1)),
        ),
    )(input)
    .map(|(next, (_, value))| (next, value))
}

fn emote_indexes(input: &str) -> Res<&str, Vec<(u32, u32)>> {
    context(
        "emote indexes",
        separated_list0(tag(","), separated_pair(digit1, tag("-"), digit1)),
    )(input)
    .map(|(next, result)| {
        let result = result
            .iter()
            .map(|(v1, v2)| (v1.parse::<u32>().unwrap(), v2.parse::<u32>().unwrap()))
            .collect::<Vec<(u32, u32)>>();
        (next, result)
    })
}

fn single_emote(input: &str) -> Res<&str, Emote> {
    context(
        "emote",
        separated_pair(alphanumerichyphen1, tag(":"), emote_indexes),
    )(input)
    .map(|(next, (id, indexes))| (next, Emote { id, indexes }))
}

fn emotes(input: &str) -> Res<&str, Vec<Emote>> {
    context(
        "emotes",
        preceded(
            tag(";"),
            separated_pair(
                tag("emotes"),
                tag("="),
                separated_list0(tag("/"), single_emote),
            ),
        ),
    )(input)
    .map(|(next, (_, result))| (next, result))
}

fn id(input: &str) -> Res<&str, &str> {
    context(
        "id",
        preceded(
            tag(";"),
            separated_pair(tag("id"), tag("="), alphanumerichyphen1),
        ),
    )(input)
    .map(|(next, (_, result))| (next, result))
}

fn moderator(input: &str) -> Res<&str, bool> {
    context(
        "mod",
        preceded(tag(";"), separated_pair(tag("mod"), tag("="), digit1)),
    )(input)
    .map(|(next, (_, value))| {
        let value = value.parse::<u32>().unwrap();
        let value = value != 0;
        (next, value)
    })
}

fn subscriber(input: &str) -> Res<&str, bool> {
    context(
        "subscriber",
        preceded(
            tag(";"),
            separated_pair(tag("subscriber"), tag("="), digit1),
        ),
    )(input)
    .map(|(next, (_, value))| {
        let value = value.parse::<u32>().unwrap();
        let value = value != 0;
        (next, value)
    })
}

fn turbo(input: &str) -> Res<&str, bool> {
    context(
        "turbo",
        preceded(tag(";"), separated_pair(tag("turbo"), tag("="), digit1)),
    )(input)
    .map(|(next, (_, value))| {
        let value = value.parse::<u32>().unwrap();
        let value = value != 0;
        (next, value)
    })
}

fn first_msg(input: &str) -> Res<&str, bool> {
    context(
        "first-msg",
        preceded(tag(";"), separated_pair(tag("first-msg"), tag("="), digit1)),
    )(input)
    .map(|(next, (_, value))| {
        let value = value.parse::<u32>().unwrap();
        let value = value != 0;
        (next, value)
    })
}

fn emote_only(input: &str) -> Res<&str, bool> {
    context(
        "emote_only",
        preceded(
            tag(";"),
            separated_pair(tag("emote-only"), tag("="), digit1),
        ),
    )(input)
    .map(|(next, (_, value))| {
        let value = value.parse::<u32>().unwrap();
        let value = value != 0;
        (next, value)
    })
}

fn flags(input: &str) -> Res<&str, Option<&str>> {
    context(
        "flags",
        preceded(
            tag(";"),
            separated_pair(tag("flags"), tag("="), opt(alphanumerichyphencolon1)),
        ),
    )(input)
    .map(|(next, (_, value))| (next, value))
}

fn client_nonce(input: &str) -> Res<&str, &str> {
    context(
        "client-nonce",
        preceded(
            tag(";"),
            separated_pair(tag("client-nonce"), tag("="), alphanumeric1),
        ),
    )(input)
    .map(|(next, (_, result))| (next, result))
}

fn room_id(input: &str) -> Res<&str, u32> {
    context(
        "room-id",
        preceded(tag(";"), separated_pair(tag("room-id"), tag("="), digit1)),
    )(input)
    .map(|(next, (_, result))| (next, result.parse::<u32>().unwrap()))
}

fn user_id(input: &str) -> Res<&str, u32> {
    context(
        "user-id",
        preceded(tag(";"), separated_pair(tag("user-id"), tag("="), digit1)),
    )(input)
    .map(|(next, (_, result))| (next, result.parse::<u32>().unwrap()))
}

fn tmi_sent_ts(input: &str) -> Res<&str, DateTime<Utc>> {
    context(
        "tmi-sent-ts",
        preceded(
            tag(";"),
            separated_pair(tag("tmi-sent-ts"), tag("="), digit1),
        ),
    )(input)
    .map(|(next, (_, result))| {
        (
            next,
            Utc.timestamp(result[0..10].parse::<i64>().unwrap(), 0),
        )
    })
}

fn user_type(input: &str) -> Res<&str, &str> {
    context(
        "user-type",
        preceded(
            tag(";"),
            separated_pair(tag("user-type"), tag("="), alphanumeric0),
        ),
    )(input)
    .map(|(next, (_, result))| (next, result))
}

fn prefix(input: &str) -> Res<&str, &str> {
    context(
        "prefix",
        preceded(multispace0, preceded(tag(":"), prefix_chars)),
    )(input)
}

fn prefix_chars<T>(i: T) -> Res<T, T>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    i.split_at_position1_complete(
        |item| {
            let char_item = item.as_char();
            char_item != '_'
                && char_item != '!'
                && char_item != '@'
                && char_item != '.'
                && !char_item.is_alphanum()
        },
        ErrorKind::AlphaNumeric,
    )
}

fn message_type(input: &str) -> Res<&str, MessageType> {
    context("message-type", preceded(multispace0, alpha1))(input)
        .map(|(next, value)| (next, value.into()))
}

fn destination(input: &str) -> Res<&str, &str> {
    context(
        "destination",
        preceded(multispace0, preceded(tag("#"), alpha1)),
    )(input)
}

fn message(input: &str) -> Res<&str, &str> {
    context("message", preceded(multispace0, preceded(tag(":"), rest)))(input)
}

fn meta_data(input: &str) -> Res<&str, MetaData> {
    context(
        "MetaData",
        tuple((
            badge_info,
            badges,
            opt(client_nonce),
            opt(bits),
            opt(color),
            display_name,
            opt(emote_only),
            emotes,
            first_msg,
            opt(flags),
            id,
            moderator,
            room_id,
            subscriber,
            tmi_sent_ts,
            turbo,
            user_id,
            opt(user_type),
        )),
    )(input)
    .map(
        |(
            next,
            (
                badge_info,
                badges,
                client_nonce,
                bits,
                color,
                display_name,
                emote_only,
                emotes,
                first_msg,
                flags,
                id,
                moderator,
                room_id,
                subscriber,
                tmi_sent_ts,
                turbo,
                user_id,
                user_type,
            ),
        )| {
            (
                next,
                MetaData {
                    badge_info,
                    badges,
                    client_nonce,
                    bits,
                    color,
                    display_name,
                    emote_only,
                    emotes,
                    first_msg,
                    flags,
                    id,
                    moderator,
                    room_id,
                    subscriber,
                    tmi_sent_ts,
                    turbo,
                    user_id,
                    user_type,
                },
            )
        },
    )
}

#[derive(Debug, PartialEq, Eq)]
struct Emote<'a> {
    id: &'a str,
    indexes: Vec<(u32, u32)>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MetaData<'a> {
    badge_info: Vec<(Badges, &'a str)>,
    badges: Vec<(Badges, &'a str)>,
    client_nonce: Option<&'a str>,
    bits: Option<u32>,
    pub color: Option<Option<(u8, u8, u8)>>,
    pub display_name: Option<&'a str>,
    emote_only: Option<bool>,
    emotes: Vec<Emote<'a>>,
    first_msg: bool,
    flags: Option<Option<&'a str>>,
    id: &'a str,
    moderator: bool,
    room_id: u32,
    subscriber: bool,
    pub tmi_sent_ts: DateTime<Utc>,
    turbo: bool,
    user_id: u32,
    user_type: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ChatMessage<'a> {
    pub meta_data: MetaData<'a>,
    prefix: &'a str,
    message_type: MessageType,
    destination: &'a str,
    pub message: &'a str,
}

impl<'a> ChatMessage<'a> {
    pub fn parse(input: &str) -> Result<ChatMessage, nom::Err<VerboseError<&str>>> {
        let (next, meta_data) = meta_data(input)?;
        let (next, prefix) = prefix(next)?;
        let (next, message_type) = message_type(next)?;
        let (next, destination) = destination(next)?;
        let (_, message) = message(next)?;
        Ok(ChatMessage {
            message_type,
            meta_data,
            prefix,
            destination,
            message,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::error::VerboseErrorKind;
    use nom::Err as NomErr;

    #[test]
    fn test_prefix() {
        assert_eq!(
            prefix(" :toerktumlare!toerktumlare@toerktumlare.tmi.twitch.tv"),
            Ok(("", "toerktumlare!toerktumlare@toerktumlare.tmi.twitch.tv"))
        )
    }

    #[test]
    fn test_badge_info() {
        assert_eq!(badge_info("badge-info="), Ok(("", vec![])));
        assert_eq!(
            badge_info("@badge-info=global_mod/1"),
            Ok(("", vec![(Badges::GlobalMod, "1")]))
        );
        assert_eq!(
            badge_info(";badge-info=admin/1,staff/1"),
            Ok(("", vec![(Badges::Admin, "1"), (Badges::Staff, "1")]))
        );
        assert_eq!(badge_info(";badge-info="), Ok(("", vec![])));
    }

    #[test]
    fn test_badge_format() {
        assert_eq!(badge_format("admin/1"), Ok(("", (Badges::Admin, "1"))));
        assert_eq!(badge_format("admin/A"), Ok(("", (Badges::Admin, "A"))));
        assert_eq!(
            badge_format("admin#1"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("#1", VerboseErrorKind::Nom(ErrorKind::Tag)),
                    ("admin#1", VerboseErrorKind::Context("badge_format"))
                ]
            }))
        );
        assert_eq!(
            badge_format("foobar/1"),
            Ok(("", (Badges::Unimplemented, "1")))
        );
    }

    #[test]
    fn test_badges() {
        assert_eq!(
            badges("badges=admin/1"),
            Ok(("", vec![(Badges::Admin, "1")]))
        );
        assert_eq!(
            badges(";badges=admin/1,subscriber/2"),
            Ok(("", vec![(Badges::Admin, "1"), (Badges::Subscriber, "2")]))
        );
        assert_eq!(
            badges("badges=admin/1"),
            Ok(("", vec![(Badges::Admin, "1")]))
        );
        assert_eq!(badges("badges="), Ok(("", vec![])));
    }

    #[test]
    fn test_hex_to_rgb() {
        assert_eq!(hex_to_rgb("#FFFFFF"), Ok(("", (255, 255, 255))));
        assert_eq!(
            hex_to_rgb("!FFFFFF"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("!FFFFFF", VerboseErrorKind::Nom(ErrorKind::Tag)),
                    ("!FFFFFF", VerboseErrorKind::Context("parse hex to rgb"))
                ]
            }))
        );
        assert_eq!(
            hex_to_rgb("#HFFFFF"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("HFFFFF", VerboseErrorKind::Nom(ErrorKind::OneOf)),
                    ("HFFFFF", VerboseErrorKind::Nom(ErrorKind::Many1)),
                    ("#HFFFFF", VerboseErrorKind::Context("parse hex to rgb"))
                ]
            }))
        );
    }

    #[test]
    fn test_color() {
        assert_eq!(color(";color=#FFFFFF"), Ok(("", Some((255, 255, 255)))));
        assert_eq!(color(";color="), Ok(("", None)));
        assert_eq!(
            color(";color#"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("#", VerboseErrorKind::Nom(ErrorKind::Tag)),
                    (";color#", VerboseErrorKind::Context("color"))
                ]
            }))
        );
    }

    #[test]
    fn test_display_name() {
        assert_eq!(
            display_name(";display-name=toerktumlare"),
            Ok(("", Some("toerktumlare")))
        );
        assert_eq!(
            display_name(";display-name#toerktumlare"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("#toerktumlare", VerboseErrorKind::Nom(ErrorKind::Tag)),
                    (
                        ";display-name#toerktumlare",
                        VerboseErrorKind::Context("display-name")
                    ),
                ]
            }))
        );
    }

    #[test]
    fn test_emote_indexes() {
        assert_eq!(emote_indexes("0-4,12-16"), Ok(("", vec![(0, 4), (12, 16)])));
        assert_eq!(emote_indexes("0-4"), Ok(("", vec![(0, 4)])));
        assert_eq!(emote_indexes(""), Ok(("", vec![])));
    }

    #[test]
    fn test_emote() {
        assert_eq!(
            single_emote("25:0-4"),
            Ok((
                "",
                Emote {
                    id: "25",
                    indexes: vec![(0, 4)],
                }
            ))
        );
        assert_eq!(
            single_emote("25:0-4,12-16"),
            Ok((
                "",
                Emote {
                    id: "25",
                    indexes: vec![(0, 4), (12, 16)],
                }
            ))
        );
        assert_eq!(
            single_emote(":0-4"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    (":0-4", VerboseErrorKind::Nom(ErrorKind::AlphaNumeric)),
                    (":0-4", VerboseErrorKind::Context("emote"))
                ]
            }))
        );
    }

    #[test]
    fn test_emotes() {
        assert_eq!(
            emotes(";emotes=25:0-4,12-16/1902:6-10"),
            Ok((
                "",
                vec![
                    Emote {
                        id: "25",
                        indexes: vec![(0, 4), (12, 16)]
                    },
                    Emote {
                        id: "1902",
                        indexes: vec![(6, 10)]
                    }
                ]
            ))
        );
        assert_eq!(
            emotes("=25:0-4,12-16/1902:6-10"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    (
                        "=25:0-4,12-16/1902:6-10",
                        VerboseErrorKind::Nom(ErrorKind::Tag)
                    ),
                    (
                        "=25:0-4,12-16/1902:6-10",
                        VerboseErrorKind::Context("emotes")
                    ),
                ]
            }))
        )
    }

    #[test]
    fn test_id() {
        assert_eq!(
            id(";id=b34ccfc7-4977-403a-8a94-33c6bac34fb8"),
            Ok(("", "b34ccfc7-4977-403a-8a94-33c6bac34fb8"))
        );
        assert_eq!(
            id("=b34ccfc7-4977-403a-8a94-33c6bac34fb8"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    (
                        "=b34ccfc7-4977-403a-8a94-33c6bac34fb8",
                        VerboseErrorKind::Nom(ErrorKind::Tag)
                    ),
                    (
                        "=b34ccfc7-4977-403a-8a94-33c6bac34fb8",
                        VerboseErrorKind::Context("id")
                    ),
                ]
            }))
        )
    }

    #[test]
    fn test_mod() {
        assert_eq!(moderator(";mod=0"), Ok(("", false)));
        assert_eq!(moderator(";mod=1"), Ok(("", true)));
        assert_eq!(
            moderator("=0"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("=0", VerboseErrorKind::Nom(ErrorKind::Tag)),
                    ("=0", VerboseErrorKind::Context("mod")),
                ]
            }))
        )
    }

    #[test]
    fn test_subscriber() {
        assert_eq!(subscriber(";subscriber=0"), Ok(("", false)));
        assert_eq!(subscriber(";subscriber=1"), Ok(("", true)));
        assert_eq!(
            subscriber("=0"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("=0", VerboseErrorKind::Nom(ErrorKind::Tag)),
                    ("=0", VerboseErrorKind::Context("subscriber")),
                ]
            }))
        )
    }

    #[test]
    fn test_turbo() {
        assert_eq!(turbo(";turbo=0"), Ok(("", false)));
        assert_eq!(turbo(";turbo=1"), Ok(("", true)));
        assert_eq!(
            turbo("=0"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("=0", VerboseErrorKind::Nom(ErrorKind::Tag)),
                    ("=0", VerboseErrorKind::Context("turbo")),
                ]
            }))
        )
    }

    #[test]
    fn test_client_nonce() {
        assert_eq!(
            client_nonce(";client-nonce=076d1c4a051be09a506e0ad9c26e6ea6"),
            Ok(("", "076d1c4a051be09a506e0ad9c26e6ea6"))
        );
        assert_eq!(
            client_nonce("=076"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("=076", VerboseErrorKind::Nom(ErrorKind::Tag)),
                    ("=076", VerboseErrorKind::Context("client-nonce")),
                ]
            }))
        );
    }

    #[test]
    fn test_tmi_sent_ts() {
        assert_eq!(
            tmi_sent_ts(";tmi-sent-ts=1500000000"),
            Ok(("", Utc.timestamp(1500000000, 0)))
        )
    }

    #[test]
    fn should_give_correct_enum() {
        assert_eq!(MessageType::from("PRIVMSG"), MessageType::PrivMsg);
    }

    #[test]
    fn should_parse_privmsg_into_enum() {
        assert_eq!(
            parse_message_type("PRIVMSG #toerktumlare: foobar"),
            Ok((" #toerktumlare: foobar", MessageType::PrivMsg))
        );
    }

    #[test]
    fn should_format_date() {
        assert_eq!(
            Utc.timestamp(1431648000, 0).to_string(),
            "2015-05-15 00:00:00 UTC"
        );
        assert_eq!(
            Utc.timestamp(1643578014, 567).to_string(),
            "2015-05-15 00:00:00 UTC"
        );
    }
}
