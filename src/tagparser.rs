// #![cfg_attr(not(test), no_std)]


use alloc::format;
use alloc::string::String;
use core::fmt;
use core::fmt::{Formatter, Result};
use esp_println::println;
use Tag::{Close, Open};
use crate::tagparser::ParserState::{InsideText, StartCloseTag, StartTag};
use crate::tagparser::Tag::{Comment, OpenLink, Standalone, Text};

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Tag<'a> {
    Open(&'a [u8]),
    OpenLink(&'a [u8], &'a [u8]),
    Close(&'a [u8]),
    Text(&'a [u8]),
    Standalone(&'a [u8]),
    Comment(&'a [u8]),
}

impl<'a> Tag<'a> {
    pub fn to_json_string(&self) -> String {
        match *self {
            Open(txt) => format!("{{\"type\":\"open\" \"name\":\"{}\" }}",String::from_utf8_lossy(txt)),
            OpenLink(txt,linkk) => format!("{{\"type\":\"open\", \"name\":\"{}\" }}",String::from_utf8_lossy(txt)),
            Close(txt) => format!("{{\"type\":\"close\", \"name\":\"{}\" }}",String::from_utf8_lossy(txt)),
            Text(txt) => format!("{{\"type\":\"text\", \"text\":\"{}\" }}",String::from_utf8_lossy(txt).replace("\n", "\\n")),
            Standalone(txt) => format!("{{\"type\":\"standalone\" \"name\":\"{}\" }}",String::from_utf8_lossy(txt).replace("\n", "-")),
            Comment(txt) => format!("{{\"type\":\"comment\" }}"),
        }
    }
}

impl fmt::Display for Tag<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Open(bytes) => write!(f, "Open({})", String::from_utf8_lossy(bytes)),
            Text(bytes) => write!(f, "Text: \"{:?}\"", String::from_utf8_lossy(bytes)),
            _ => {
                write!(f, "{:?}", self)
            }
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
enum ParserState {
    StartStream,
    StartTag,
    StartCloseTag,
    InsideText,
}

pub struct TagParser<'a> {
    pos: usize,
    data: &'a [u8],
    state:ParserState,
    index: usize,
    debug: bool,
}

#[derive(Debug)]
#[derive(PartialEq)]
enum AttState {
    Whitespace,
    KeyName,
    KeyValue,
    Equals,
}

impl<'a> TagParser<'a> {
    pub(crate) fn find_href(&self, start: usize, end: usize) -> Option<&'a [u8]> {
        let mut state = AttState::Whitespace;
        let mut start_keyname = start;
        let mut end_keyname = start;
        let mut start_keyvalue = start;
        let mut end_keyvalue = start;
        for i in start .. end {
            // println!("{} {:?} {:?}", self.data[i] as char, self.data[i], state);
            let ch = self.data[i];
            match ch {
                b' ' => {
                    if AttState::Whitespace == state {
                        // println!("more whitespace");
                        start_keyname = i;
                    }
                },
                b'=' => {
                    // println!("equals");
                    state = AttState::Equals;
                    end_keyname = i;
                    let key = &self.data[start_keyname .. end_keyname];
                    // println!("finished a keyname {} to {} {:?} ", start_keyname, i, key);
                }
                b'"' | b'\'' => {
                    if state == AttState::KeyValue {
                        // println!("exiting value");
                        state = AttState::Whitespace;
                        end_keyvalue = i;
                        let value = &self.data[start_keyvalue .. end_keyvalue];
                        // println!("finished a key value {:?}", value);
                        let key = &self.data[start_keyname .. end_keyname];
                        // println!("key name is {:?}", key);
                        if key.eq(b"href") {
                            // println!("href found ");
                            return Some(value);
                        }
                    }
                    if state == AttState::Equals {
                        // println!("entering value");
                        start_keyvalue = i+1;
                        state = AttState::KeyValue;
                    }
                }
                _ => {
                    if state == AttState::Whitespace {
                        // println!("leaving whitespace");
                        start_keyname = i;
                        state = AttState::KeyName;
                    }
                    if state == AttState::Equals {
                        // println!("after equals without a quote. start keyvalue");
                        start_keyvalue = i;
                        state = AttState::KeyValue;
                    }
                    // println!("other {:?} -> {:?}", ch, self.data[i]);
                }
            }
            if state == AttState::Whitespace  && ch == b'h'{
                state = AttState::KeyName;
            }
            // if self.data[i..].starts_with(b"href=\"") {
            //     println!("found href");
            // }
        }
        return None
    }
}

impl<'a> TagParser<'a> {
    pub(crate) fn chomp_to(&mut self, target: &[u8; 3]) {
        loop {
            // println!("chomped {} {}", self.data[self.pos] as char, self.data[self.pos]);
            if self.data[self.pos..].starts_with(target) {
                // println!("matched");
                return;
            }
            self.pos += 1;
        }
    }
}

impl TagParser<'_> {
    pub fn new(input: &str) -> TagParser {
        TagParser::new(input)
    }
    pub fn with_debug(input: &[u8], debug:bool) -> TagParser {
        TagParser {
            data:input,
            pos: 0,
            state:ParserState::StartStream,
            index: 0,
            debug:debug,
        }
    }
}


impl<'a> Iterator for TagParser<'a> {
    type Item = Tag<'a>;

    /*
        < => start tag
        <! => start directive
        <!-- => start comment
        <!-- * --!> => comment
        <! * >      => directive
        <A+ * >     => open tag
        <A+ * />    => void tag
        </A+ * >    => close tag
     */
    fn next(&mut self) -> Option<Self::Item> {
        if self.debug {
            println!("start loop {:?} {:?} {:?} {}", self.state, self.pos, self.index, self.data.len());
        }
        loop {
            // if self.pos == self.data.len() -1 {
            //     println!("end loop");
            //     return None;
            // }
            if self.pos == self.data.len() {
                let slice = &self.data[self.index .. self.pos];
                self.pos += 1;
                return Some(Text(slice))
            }
            if self.pos > self.data.len() {
                // println!("end loop");
                return None;
            }
            let ch = self.data[self.pos];
            if self.debug { println!("{} {} {:?}", String::from(ch as char), ch, self.state); }
            self.pos += 1;
            match ch {
                b'<' => {
                    // println!("opening a tag");
                    let text = &self.data[self.index .. self.pos-1];
                    self.index = self.pos;
                    self.state = StartTag;
                    // println!("text length {}", text.len());
                    if text.len() > 0 {
                        // println!("prev text is{:?}", text);
                        return Some(Text(text));
                    }
                }
                b'!' => {
                    if self.state == StartTag {
                        let next = self.data[self.pos+1];
                        if next == b'-' {
                            // println!("doing a comment");
                            self.chomp_to(b"-->");
                            let slice = &self.data[self.index + 3..self.pos];
                            self.state = InsideText;
                            self.index = self.pos;
                            return Some(Comment(slice));
                        }
                    }
                }
                b'/' => {
                    if self.state == StartTag {
                        let prev = self.data[self.pos-2];
                        if prev == b'<' {
                            // println!("switch to close tag");
                            self.state = StartCloseTag;
                            self.index = self.pos;
                        }
                    }
                }
                b'>' => {
                    // println!("ending a tag");
                    // get the part from the beginning to the first whitespace, if any
                    let mut name_end = self.index;
                    while self.data[name_end] != b' ' && name_end < self.pos-1 {
                        name_end += 1;
                    }
                    // if name_end = / then back up one
                    if self.data[name_end-1] == b'/' {
                        // println!("too far by one");
                        name_end = name_end -1;
                    }
                    let slice = &self.data[self.index .. name_end];
                    // println!("slice  {:?}", slice);

                    let mut tag = Some(Open(slice));
                    if self.state == StartCloseTag {
                        tag = Some(Close(slice));
                    }
                    let maybe_href = self.find_href(name_end, self.pos-1);
                    if let Some(href) = maybe_href {
                        // println!("found an href  {:?}", href);
                        tag = Some(OpenLink(slice,href))
                    }
                    self.state = InsideText;
                    self.index = self.pos;
                    return tag;
                }
                _ => {}
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn plain_text() {
    //     let html = r#"some text"#;
    //     let mut parser = TagParser::new(&html);
    //     assert_eq!(parser.next().unwrap(), Text(b"some text"));
    // }
    // #[test]
    // fn open_tag() {
    //     let html = r#"<html>"#;
    //     let mut parser = TagParser::new(&html);
    //     assert_eq!(parser.next().unwrap(), Open(b"html"));
    // }
    //
    // #[test]
    // fn close_tag() {
    //     let html = r#"</html>"#;
    //     let mut parser = TagParser::new(&html);
    //     assert_eq!(parser.next().unwrap(), Close(b"html"));
    // }
    //
    // #[test]
    // fn nested_tags() {
    //     let html = r#"<html><body></body></html>"#;
    //     let mut parser = TagParser::new(&html);
    //     assert_eq!(parser.next().unwrap(), Open(b"html"));
    //     assert_eq!(parser.next().unwrap(), Open(b"body"));
    //     assert_eq!(parser.next().unwrap(), Close(b"body"));
    //     assert_eq!(parser.next().unwrap(), Close(b"html"));
    // }
    //
    // #[test]
    // fn more_nested_tags() {
    //     let html = r#"<body><p>some text</p><p>more text</p></body>"#;
    //     let mut parser = TagParser::new(&html);
    //     assert_eq!(parser.next().unwrap(), Open(b"body"));
    //     assert_eq!(parser.next().unwrap(), Open(b"p"));
    //     assert_eq!(parser.next().unwrap(), Text(b"some text"));
    //     assert_eq!(parser.next().unwrap(), Close(b"p"));
    //     assert_eq!(parser.next().unwrap(), Open(b"p"));
    //     assert_eq!(parser.next().unwrap(), Text(b"more text"));
    //     assert_eq!(parser.next().unwrap(), Close(b"p"));
    //     assert_eq!(parser.next().unwrap(), Close(b"body"));
    // }
    //
    // #[test]
    // fn strip_whitespace_inside_tags() {
    //     let html = r#"<html   >"#;
    //     let mut parser = TagParser::new(&html);
    //     assert_eq!(parser.next().unwrap(), Open(b"html"));
    // }
    //
    // #[test]
    // fn skip_attributes() {
    //     let mut parser = TagParser::new(r#"<html foo="bar" bar="foo" >"#);
    //     assert_eq!(parser.next().unwrap(), Open(b"html"));
    // }
    //
    // #[test]
    // fn parse_href() {
    //     let mut parser = TagParser::new(r#"<a href="foo">"#);
    //     assert_eq!(parser.next().unwrap(), OpenLink(b"a",b"foo"));
    // }
    //
    // #[test]
    // fn handle_single_quote_attributes() {
    //     let mut parser = TagParser::new(r#"<a href='foo'>"#);
    //     assert_eq!(parser.next().unwrap(), OpenLink(b"a",b"foo"));
    // }
    //
    // // #[test]
    // // fn handle_missing_attribute_quotes() {
    // //     let mut parser = TagParser::new(r#"<a href=foo>"#);
    // //     assert_eq!(parser.next().unwrap(), OpenLink(b"a",b"foo"));
    // // }
    //
    // #[test]
    // fn parse_comments() {
    //     let mut parser = TagParser::new("<!-- hi there -->");
    //     assert_eq!(parser.next().unwrap(), Comment(b" hi there "));
    // }
    // //
    // // #[test]
    // // fn parse_meta_as_standalone() {
    // //     let it: TagParser = TagParser::new("<meta key=\"value\">");
    // //     let tags: Vec<Tag> = it.into_iter().collect();
    // //     let meta = Tag {
    // //         kind: Standalone,
    // //         name: "meta".into(),
    // //         text: "".into(),
    // //         attributes: HashMap::from([
    // //             ("key".into(), "value".into())
    // //         ]),
    // //     };
    // //     assert_eq!(tags, vec![meta]);
    // // }
    // //
    //
    // #[test]
    // fn parse_doctype() {
    //     let mut parser: TagParser = TagParser::new("<!DOCTYPE html>");
    //     assert_eq!(parser.next().unwrap(), Open(b"!DOCTYPE"));
    // }
    //
    //
    // #[test]
    // fn self_closing_img() {
    //     let mut parser = TagParser::new("<img href='foo'/>");
    //     assert_eq!(parser.next().unwrap(), OpenLink(b"img",b"foo"));
    // }
    //
    // #[test]
    // fn dont_parse_slash_in_short_selfclosing_tags() {
    //     let mut parser = TagParser::new("<br/>");
    //     assert_eq!(parser.next().unwrap(), Open(b"br"));
    // }
    //
    // #[test]
    // fn slash_in_text() {
    //     let mut it: TagParser = TagParser::new("<p> foo / bar </p>");
    //     assert_eq!(it.next().unwrap(), Open(b"p"));
    //     assert_eq!(it.next().unwrap(), Text(b" foo / bar "));
    //     assert_eq!(it.next().unwrap(), Close(b"p"));
    // }

    //
    // #[test]
    // fn entity_in_text() {
    //     let it: TagParser = TagParser::new("<p>&gt;</p>");
    //     let tags: Vec<Tag> = it.into_iter().collect();
    //     assert_eq!(
    //         tags,
    //         vec![
    //             Tag::new(Open, "p"),
    //             Tag::new(Text, ">"),
    //             Tag::new(Close, "p"),
    //         ]
    //     );
    // }
    //
    // #[test]
    // fn attributes_in_parent_and_child() {
    //     let it: TagParser = TagParser::new(r#"<div class='foo'><a href="bar">link</a></div>"#);
    //     let tags: Vec<Tag> = it.into_iter().collect();
    //     assert_eq!(
    //         tags,
    //         vec![
    //             Tag::open_element_with_attributes(
    //                 "div",
    //                 &HashMap::from([("class".into(), "foo".into())]),
    //             ),
    //             Tag::open_element_with_attributes(
    //                 "a",
    //                 &HashMap::from([("href".to_string(), "bar".to_string())])
    //             ),
    //             // Tag::new(Open,"a"),d
    //             Tag::new(Text, "link"),
    //             Tag::new(Close, "a"),
    //             Tag::new(Close, "div"),
    //         ]
    //     );
    // }
    //
    // #[test]
    // fn collapse_adjacent_whitespace() {
    //     let it: TagParser = TagParser::new(r#"<div>before
    //         after</div>"#);
    //     let tags: Vec<Tag> = it.into_iter().collect();
    //     assert_eq!(
    //         tags,
    //         vec![
    //             Tag::open("div"),
    //             Tag::text("before after"),
    //             Tag::close("div"),
    //         ]
    //     )
    // }
    //
    // #[test]
    // fn chomp_away_script_tags() {
    //     let it: TagParser = TagParser::new(r#"<script>some inner
    //     text with a greater than > < </script><div>more</div>"#);
    //     let tags: Vec<Tag> = it.into_iter().collect();
    //     let script = Tag {
    //         kind:Stripped,
    //         name: "script".into(),
    //         text: "".into(),
    //         attributes:HashMap::new(),
    //     };
    //     assert_eq!(tags[0], script);
    //     assert_eq!(tags[1], Tag::open("div"));
    // }

}


