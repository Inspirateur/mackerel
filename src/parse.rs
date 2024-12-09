use nom::{bytes::complete::tag, character::complete::{digit1, line_ending, multispace0, multispace1, space0, space1}, combinator::{eof, map, map_res, opt}, error::{Error, ParseError}, multi::{many1, separated_list1}, sequence::{delimited, tuple}, IResult};
use rdev::{Button, EventType, Key};
use crate::{script::{Action, Trigger}, Macro};

pub fn parse_file(input: &str) -> IResult<&str, Vec<Macro>> {
    let (input, res) = many1(ws(parse_macro))(input)?;
    Ok((input, res))
}

fn parse_macro(input: &str) -> IResult<&str, Macro> {
    let (input, trigger) = parse_trigger(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("{")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, actions) = many1(ws(parse_action_line))(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("}")(input)?;
    Ok((input, Macro { trigger, actions: actions.into_iter().flatten().collect() }))
}

fn parse_trigger(input: &str) -> IResult<&str, Trigger> {
    let (input, actions) = separated_list1(ws(tag("+")), parse_trigger_press)(input)?;
    let actions = actions.into_iter().map(|action| match action {
        Action::Event(event_type) => event_type,
        _ => unreachable!(),
    }).collect::<Vec<_>>();
    Ok((input, if actions.len() == 1 { Trigger::Single(actions[0]) } else { Trigger::Combo(actions) } ))
}

fn parse_trigger_press(input: &str) -> IResult<&str, Action> {
    let (input, action) = map(parse_button, button_release)(input)?;
    Ok((input, action))
}

fn parse_action_line(input: &str) -> IResult<&str, Vec<Action>> {
    let (input, action) = parse_action(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = statement_end(input)?;
    Ok((input, action))
}

fn parse_action(input: &str) -> IResult<&str, Vec<Action>> {
    let (input, action) = parse_press(input)
        .or(parse_hold(input))
        .or(parse_release(input))
        .or(parse_move(input))
        .or(parse_wait(input))?;
    Ok((input, action))
}

fn parse_wait(input: &str) -> IResult<&str, Vec<Action>> {
    let (input, _) = tag("wait")(input)?;
    let (input, _) = space1(input)?;
    let (input, ms) = parse_seconds(input)?;
    Ok((input, vec![Action::Wait(ms)]))
}

fn parse_press(input: &str) -> IResult<&str, Vec<Action>> {
    let (input, _) = tag("press")(input)?;
    let (input, _) = space1(input)?;
    let (input, action) = map(parse_button, button_press_release)(input)
        .or_else(|_: nom::Err<Error<&str>>| map(parse_key, key_press_release)(input))?;
    Ok((input, action))
}

fn parse_hold(input: &str) -> IResult<&str, Vec<Action>> {
    let (input, _) = tag("hold")(input)?;
    let (input, _) = space1(input)?;
    let (input, action) = map(parse_button, button_press)(input)
        .or_else(|_: nom::Err<Error<&str>>| map(parse_key, key_press)(input))?;
    Ok((input, vec![action]))
}

fn parse_release(input: &str) -> IResult<&str, Vec<Action>> {
    let (input, _) = tag("release")(input)?;
    let (input, _) = space1(input)?;
    let (input, action) = map(parse_button, button_release)(input)
        .or_else(|_: nom::Err<Error<&str>>| map(parse_key, key_release)(input))?;
    Ok((input, vec![action]))
}

fn parse_button(input: &str) -> IResult<&str, Button> {
    let (input, _) = tag("Mouse")(input)?;
    map(tag("Left"), |_| Button::Left)(input)
        .or_else(|_: nom::Err<Error<&str>>| map(tag("Right"), |_| Button::Right)(input))
        .or_else(|_: nom::Err<Error<&str>>| map(tag("Middle"), |_| Button::Middle)(input))
        .or_else(|_: nom::Err<Error<&str>>| map(digit1, |n: &str| Button::Unknown(n.parse().unwrap()))(input))
}

fn parse_key(input: &str) -> IResult<&str, Key> {
    todo!()
}

fn parse_move(input: &str) -> IResult<&str, Vec<Action>> {
    let (input, _) = tag("move to")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, action) = map(tag("start"), |_| Action::MoveToStart)(input)
        .or_else(|_: nom::Err<Error<&str>>| map(parse_loc, mouse_move)(input))?;
    Ok((input, vec![action]))
}

fn parse_loc(input: &str) -> IResult<&str, (i32, i32)> {
    let (input, x) = map_res(digit1, str::parse)(input)?;
    let (input, _) = ws(tag(","))(input)?;
    let (input, y) = map_res(digit1, str::parse)(input)?;
    Ok((input, (x, y)))
}

/// Result in ms
fn parse_seconds(input: &str) -> IResult<&str, u32> {
    let (input, i) = digit1(input)?;
    let (input, opt_dec) = opt(tuple((tag("."), digit1)))(input)?;
    let dec = if let Some((_, d)) = opt_dec {
        d[0..3.min(d.len())].parse::<u32>().unwrap()
    } else {
        0
    };
    Ok((input, i.parse::<u32>().unwrap()*1000 + dec*100))
}

fn statement_end(input: &str) -> IResult<&str, ()> {
    line_ending(input).map(|(input, _)| (input, ())).or_else(|_: nom::Err<Error<&str>>| eof(input).map(|(input, _)| (input, ())))
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and 
/// trailing whitespace, returning the output of `inner`.
fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
  where
  F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(
        multispace0,
        inner,
        multispace0
    )
}

fn mouse_move((x, y): (i32, i32)) -> Action {
    Action::Event(EventType::MouseMove { x: x as f64, y: y as f64 })
}

fn key_press(key: Key) -> Action {
    Action::Event(EventType::KeyPress(key))
}

fn button_press(button: Button) -> Action {
    Action::Event(EventType::ButtonPress(button))
}

fn key_release(key: Key) -> Action {
    Action::Event(EventType::KeyRelease(key))
}

fn button_release(button: Button) -> Action {
    Action::Event(EventType::ButtonRelease(button))
}

fn key_press_release(key: Key) -> Vec<Action> {
    vec![key_press(key), key_release(key)]
}

fn button_press_release(button: Button) -> Vec<Action> {
    vec![button_press(button), button_release(button)]
}

#[cfg(test)]
mod tests {
    use crate::parse::*;

    #[test]
    fn test_parse_seconds() {
        let secdef1 = "0.1234";
        let secdef2 = "0.123";
        let (_, ms1) = parse_seconds(secdef1).unwrap();
        let (_, ms2) = parse_seconds(secdef2).unwrap();
        assert_eq!(123, ms1);
        assert_eq!(123, ms2);
    }

    #[test]
    fn test_parse_action() {
        let actiondef = "press Mouse1";
        let (input, action) = parse_action(actiondef).unwrap();
        assert_eq!(input, "");
        println!("{action:?}");
    }

    #[test]
    fn test_parse_macro() {
        let macrodef = r#"Mouse1 {
            move to 1110, 600
            press MouseLeft
            move to start
        }"#;
        let (input, script) = parse_macro(macrodef).unwrap();
        assert_eq!(input, "");
        println!("{script:?}");
    }

    #[test]
    fn test_parse_file() {
        let file = r#"
        Mouse1 {
            move to 1110, 600
            press MouseLeft
            move to start
        }"#;
        let (input, macros) = parse_file(file).unwrap();
        assert_eq!(input, "");
        println!("{macros:?}");
    }
}