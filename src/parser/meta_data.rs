use super::tags;
use chrono::{DateTime, TimeZone, Utc};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{alphanumeric0, alphanumeric1, digit1, one_of},
    combinator::opt,
    error::{context, ErrorKind, ParseError, VerboseError},
    multi::{many1, separated_list0},
    sequence::{preceded, separated_pair, tuple},
    AsChar, IResult, InputTakeAtPosition, Parser,
};

use super::{Badges, Emote};

#[derive(Debug, PartialEq, Eq)]
pub struct MetaData<'a> {
    pub badge_info: Vec<(Badges, &'a str)>,
    pub badges: Vec<(Badges, &'a str)>,
    pub client_nonce: Option<&'a str>,
    pub bits: Option<u32>,
    pub color: Option<Option<(u8, u8, u8)>>,
    pub display_name: Option<&'a str>,
    pub emote_only: Option<bool>,
    pub emotes: Vec<Emote<'a>>,
    pub first_msg: bool,
    pub flags: Option<Option<&'a str>>,
    pub id: &'a str,
    pub moderator: bool,
    pub reply: Option<Reply<'a>>,
    pub room_id: u32,
    pub subscriber: bool,
    pub tmi_sent_ts: DateTime<Utc>,
    pub turbo: bool,
    pub user_id: u32,
    pub user_type: Option<&'a str>,
}

impl<'a> MetaData<'a> {
    pub fn new(input: &'a str) -> Res<&str, MetaData> {
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
                tuple((
                    opt(sep_pair(tags::REPLY_PARENT_DISPLAY_NAME, username)),
                    opt(sep_pair(tags::REPLY_PARENT_MSG_BODY, parse_msg_body)),
                    opt(sep_pair(tags::REPLY_PARENT_MSG_ID, alphanumerichyphen1)),
                    opt(sep_pair(tags::REPLY_PARENT_USER_ID, digit1).map(|v| v.parse().unwrap())),
                    opt(sep_pair(tags::REPLY_PARENT_USER_LOGIN, username)),
                )),
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
                    (
                        reply_parent_display_name,
                        reply_parent_msg_body,
                        reply_parent_msg_id,
                        reply_parent_user_id,
                        reply_parent_user_login,
                    ),
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
                        reply: Some(Reply {
                            display_name: reply_parent_display_name,
                            msg_body: reply_parent_msg_body,
                            msg_id: reply_parent_msg_id,
                            user_id: reply_parent_user_id,
                            user_login: reply_parent_user_login,
                        }),
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
}

#[derive(Debug, Eq, PartialEq)]
pub struct Reply<'a> {
    display_name: Option<&'a str>,
    msg_body: Option<&'a str>,
    msg_id: Option<&'a str>,
    user_id: Option<u32>,
    user_login: Option<&'a str>,
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

fn username<T>(i: T) -> Res<T, T>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    i.split_at_position1_complete(
        |item| {
            let char_item = item.as_char();
            char_item != '_' && !char_item.is_alphanum()
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

fn badge_info(input: &str) -> Res<&str, Vec<(Badges, &str)>> {
    context(
        tags::BADGE_INFO,
        preceded(
            opt(alt((tag("@"), tag(";")))),
            separated_pair(
                tag(tags::BADGE_INFO),
                tag("="),
                separated_list0(tag(","), badge_format),
            ),
        ),
    )(input)
    .map(|(next, (_, value))| (next, value))
}

fn badges(input: &str) -> Res<&str, Vec<(Badges, &str)>> {
    context(
        tags::BADGES,
        preceded(
            opt(alt((tag("@"), tag(";")))),
            separated_pair(
                tag(tags::BADGES),
                tag("="),
                separated_list0(tag(","), badge_format),
            ),
        ),
    )(input)
    .map(|(next, (_, value))| (next, value))
}

fn badge_format(input: &str) -> Res<&str, (Badges, &str)> {
    context(
        tags::BADGE_FORMAT,
        separated_pair(alphanumerichyphen1, tag("/"), alphanumerichyphenbackslash1),
    )(input)
    .map(|(next, (badge, version))| (next, (badge.into(), version)))
}

fn color(input: &str) -> Res<&str, Option<(u8, u8, u8)>> {
    context(
        tags::COLOR,
        preceded(
            alt((tag("@"), tag(";"))),
            separated_pair(tag(tags::COLOR), tag("="), opt(hex_to_rgb)),
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
    sep_pair(tags::BITS, digit1)
        .parse(input)
        .map(|(next, value)| (next, value.parse::<u32>().unwrap()))
}

fn display_name(input: &str) -> Res<&str, Option<&str>> {
    context(
        tags::DISPLAY_NAME,
        preceded(
            tag(";"),
            separated_pair(tag(tags::DISPLAY_NAME), tag("="), opt(username)),
        ),
    )(input)
    .map(|(next, (_, value))| (next, value))
}

fn emote_only(input: &str) -> Res<&str, bool> {
    sep_pair_to_bool(tags::EMOTE_ONLY, digit1).parse(input)
}

fn client_nonce(input: &str) -> Res<&str, &str> {
    sep_pair(tags::CLIENT_NONCE, alphanumeric1)
        .parse(input)
        .map(|(next, result)| (next, result))
}

fn room_id(input: &str) -> Res<&str, u32> {
    context(
        tags::ROOM_ID,
        preceded(
            tag(";"),
            separated_pair(tag(tags::ROOM_ID), tag("="), digit1),
        ),
    )(input)
    .map(|(next, (_, result))| (next, result.parse::<u32>().unwrap()))
}

fn user_id(input: &str) -> Res<&str, u32> {
    context(
        tags::USER_ID,
        preceded(
            tag(";"),
            separated_pair(tag(tags::USER_ID), tag("="), digit1),
        ),
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
    sep_pair(tags::USER_TYPE, alphanumeric0).parse(input)
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

fn flags(input: &str) -> Res<&str, Option<&str>> {
    context(
        tags::FLAGS,
        preceded(
            tag(";"),
            separated_pair(tag(tags::FLAGS), tag("="), opt(alphanumerichyphencolon1)),
        ),
    )(input)
    .map(|(next, (_, value))| (next, value))
}

fn moderator(input: &str) -> Res<&str, bool> {
    sep_pair_to_bool(tags::MODERATOR, digit1).parse(input)
}

fn subscriber(input: &str) -> Res<&str, bool> {
    sep_pair_to_bool(tags::SUBSCRIBER, digit1).parse(input)
}

fn turbo(input: &str) -> Res<&str, bool> {
    sep_pair_to_bool(tags::TURBO, digit1).parse(input)
}

fn first_msg(input: &str) -> Res<&str, bool> {
    sep_pair_to_bool(tags::FIRST_MSG, digit1).parse(input)
}

fn emotes(input: &str) -> Res<&str, Vec<Emote>> {
    context(
        tags::EMOTES,
        preceded(
            tag(";"),
            separated_pair(
                tag(tags::EMOTES),
                tag("="),
                separated_list0(tag("/"), single_emote),
            ),
        ),
    )(input)
    .map(|(next, (_, result))| (next, result))
}

fn single_emote(input: &str) -> Res<&str, Emote> {
    context(
        tags::EMOTE,
        separated_pair(alphanumerichyphen1, tag(":"), emote_indexes),
    )(input)
    .map(|(next, (id, indexes))| (next, Emote { id, indexes }))
}

fn emote_indexes(input: &str) -> Res<&str, Vec<(u32, u32)>> {
    context(
        tags::EMOTE_INDEXES,
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

fn id(input: &str) -> Res<&str, &str> {
    sep_pair(tags::ID, alphanumerichyphen1).parse(input)
}

fn parse_msg_body<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    take_till(|c| c == ';')(i)
}

fn sep_pair<'a, T>(
    param_string: &'static str,
    p: impl Parser<&'a str, T, VerboseError<&'a str>> + Copy,
) -> impl Parser<&'a str, T, VerboseError<&'a str>> {
    move |input| {
        context(
            param_string,
            preceded(tag(";"), separated_pair(tag(param_string), tag("="), p)),
        )(input)
        .map(|(next, (_, result))| (next, result))
    }
}

fn sep_pair_to_bool<'a>(
    param_string: &'static str,
    p: impl Parser<&'a str, &'a str, VerboseError<&'a str>> + Copy,
) -> impl Parser<&'a str, bool, VerboseError<&'a str>> {
    move |input| {
        context(
            param_string,
            preceded(tag(";"), separated_pair(tag(param_string), tag("="), p)),
        )(input)
        .map(|(next, (_, value))| {
            let value = value.parse::<u32>().unwrap();
            let value = value != 0;
            (next, value)
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use nom::error::VerboseErrorKind;
    use nom::Err as NomErr;

    // @badge-info=;
    // badges=;
    // client-nonce=fd58b3d4628840dc10378c532c344ff8;
    // color=;
    // display-name=leprajon;
    // emotes=;
    // first-msg=0;
    // flags=;
    // id=d238119b-ebe3-41a7-b177-55d145402d0b;
    // mod=0;
    // reply-parent-display-name=Toerktumlare;
    // reply-parent-msg-body=hello\schat!;
    // reply-parent-msg-id=9bf7210b-e249-4d32-a240-fc0a6bb762a8;
    // reply-parent-user-id=47496925;
    // reply-parent-user-login=toerktumlare;
    // room-id=47496925;
    // subscriber=0;
    // tmi-sent-ts=1646864986812;
    // turbo=0;
    // user-id=149182416;
    // user-type=
    // :leprajon!leprajon@leprajon.tmi.twitch.tv PRIVMSG #toerktumlare :@Toerktumlare asd
    //
    //
    //
    // badge_info: Vec<(Badges, &'a str)>,
    // badges: Vec<(Badges, &'a str)>,
    // client_nonce: Option<&'a str>,
    // bits: Option<u32>,
    // pub color: Option<Option<(u8, u8, u8)>>,
    // pub display_name: Option<&'a str>,
    // emote_only: Option<bool>,
    // emotes: Vec<Emote<'a>>,
    // first_msg: bool,
    // flags: Option<Option<&'a str>>,
    // id: &'a str,
    // moderator: bool,
    // reply_parent_display_name: Option<&'a str>,
    // reply_parent_msg_body: Option<&'a str>,
    // reply_parent_msg_id: Option<&'a str>,
    // reply_parent_user_id: Option<u32>,
    // reply_parent_user_login: Option<&'a str>,
    // room_id: u32,
    // subscriber: bool,
    // pub tmi_sent_ts: DateTime<Utc>,
    // turbo: bool,
    // user_id: u32,
    // user_type: Option<&'a str>,

    #[test]
    fn parse_reply_message_meta_data() {
        let meta_data_string = "@badge-info=;badges=;client-nonce=abc123;color=#FFFFFF;display-name=kirglow;emotes=;first-msg=0;flags=;id=2f-7e;mod=0;reply-parent-display-name=Toerktumlare;reply-parent-msg-body=take\\s2;reply-parent-msg-id=87-f3;reply-parent-user-id=4749;reply-parent-user-login=toerktumlare;room-id=4749;subscriber=0;tmi-sent-ts=1500000000;turbo=0;user-id=60;user-type=";
        let meta_data = MetaData {
            badge_info: vec![],
            badges: vec![],
            client_nonce: Some("abc123"),
            bits: None,
            color: Some(Some((255, 255, 255))),
            display_name: Some("kirglow"),
            emote_only: None,
            emotes: vec![],
            first_msg: false,
            flags: Some(None),
            id: "2f-7e",
            moderator: false,
            reply: Some(Reply {
                display_name: Some("Toerktumlare"),
                msg_body: Some("take\\s2"),
                msg_id: Some("87-f3"),
                user_id: Some(4749),
                user_login: Some("toerktumlare"),
            }),
            room_id: 4749,
            subscriber: false,
            tmi_sent_ts: Utc.timestamp(1500000000, 0),
            turbo: false,
            user_id: 60,
            user_type: Some(""),
        };

        assert_eq!(MetaData::new(meta_data_string), Ok(("", meta_data)));
    }

    #[test]
    fn test_reply_parent_display_name() {
        assert_eq!(
            sep_pair(tags::REPLY_PARENT_DISPLAY_NAME, username)
                .parse(";reply-parent-display-name=Toerktumlare"),
            Ok(("", "Toerktumlare"))
        );
    }

    #[test]
    fn test_reply_parent_msg_body() {
        assert_eq!(
            sep_pair(tags::REPLY_PARENT_MSG_BODY, parse_msg_body)
                .parse(";reply-parent-msg-body=hello\\schat!"),
            Ok(("", "hello\\schat!"))
        );
    }

    #[test]
    fn test_reply_parent_msg_id() {
        assert_eq!(
            sep_pair(tags::REPLY_PARENT_MSG_ID, alphanumerichyphen1)
                .parse(";reply-parent-msg-id=9bf7210b-e249-4d32-a240-fc0a6bb762a8h"),
            Ok(("", "9bf7210b-e249-4d32-a240-fc0a6bb762a8h"))
        );
    }

    #[test]
    fn test_reply_parent_user_id() {
        assert_eq!(
            sep_pair(tags::REPLY_PARENT_USER_ID, digit1).parse(";reply-parent-user-id=47496925"),
            Ok(("", "47496925"))
        );
    }

    #[test]
    fn test_reply_parent_user_login() {
        assert_eq!(
            sep_pair(tags::REPLY_PARENT_USER_LOGIN, username)
                .parse(";reply-parent-user-login=toerktumlare"),
            Ok(("", "toerktumlare"))
        );
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
                    ("admin#1", VerboseErrorKind::Context("badge-format"))
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
    fn should_format_date() {
        assert_eq!(
            Utc.timestamp(1431648000, 0).to_string(),
            "2015-05-15 00:00:00 UTC"
        );
    }
}
