//! Simple in-memory file system for static assets

use phf::phf_map;

pub static ASSETS: phf::Map<&'static str, &'static [u8]> = phf_map! {
    "static/fartscroll.js" => include_bytes!("../static/fartscroll.js"),
    "static/robots.txt" => include_bytes!("../static/robots.txt")
};
