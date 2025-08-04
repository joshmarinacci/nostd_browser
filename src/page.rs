use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use nostd_html_parser::blocks::{Block, BlockParser};
use nostd_html_parser::tags::TagParser;

#[derive(Debug)]
pub struct Page {
    pub url: String,
    pub links: Vec<u32>,
    pub selection: i32,
    pub blocks: Vec<Block>,
}

impl Page {
    pub fn from_bytes(bytes: &[u8], url:&str) -> Page {
        let tags = TagParser::new(bytes);
        let block_parser = BlockParser::new(tags);
        let blocks = block_parser.collect();
        Page {
            url: url.to_string(),
            links: vec![],
            selection: 0,
            blocks,
        }
    }
}

