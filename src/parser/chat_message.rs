use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, multispace0},
    combinator::rest,
    error::{context, ErrorKind, VerboseError},
    sequence::preceded,
    AsChar, IResult, InputTakeAtPosition,
};

use super::{
    meta_data::{new_meta_data, MetaData},
    MessageType,
};

type Res<T, U> = IResult<T, U, VerboseError<T>>;

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
        let (next, meta_data) = new_meta_data(input)?;
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

    #[test]
    fn test_prefix() {
        assert_eq!(
            prefix(" :toerktumlare!toerktumlare@toerktumlare.tmi.twitch.tv"),
            Ok(("", "toerktumlare!toerktumlare@toerktumlare.tmi.twitch.tv"))
        )
    }
}
