use alloc::string::String;
use alloc::vec::Vec;
use nostd_html_parser::blocks::Block;

#[derive(Debug)]
pub struct Page {
    pub url: String,
    pub links: Vec<u32>,
    pub selection: i32,
    pub blocks: Vec<Block>,
}

