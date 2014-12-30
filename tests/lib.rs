#![feature(phase)]

extern crate xml;
#[phase(plugin, link)] extern crate fromxml;

use std::borrow::ToOwned;

use xml::reader::EventReader;

deriving_fromxml! {
    struct TestStruct {
        name:String,
        attr:Option<String>,
    }
}

// pub fn load_file(input:&str) -> Device {
//     let file = File::open(&Path::new(input)).unwrap();
//     let reader = BufferedReader::new(file);

//     let mut parser = EventReader::new(reader);
//     let mut iter = parser.events();
//     parse_root::<Device>(&mut iter).unwrap()
// }

#[test]
fn test_xml_1() {
    let mut parser = EventReader::new_from_string("<test><name>hi</name></test>".to_owned());
    let mut iter = parser.events();
    let test = ::fromxml::parse_root::<TestStruct, ::std::io::MemReader>(&mut iter).unwrap();
    println!("test {}", test);

    assert_eq!(test.name, "hi");
    assert_eq!(test.attr, None);
}

#[test]
fn test_xml_2() {
    let mut parser = EventReader::new_from_string("<test><name>hi</name><attr>hey girl</attr></test>".to_owned());
    let mut iter = parser.events();
    let test = ::fromxml::parse_root::<TestStruct, ::std::io::MemReader>(&mut iter).unwrap();
    println!("test {}", test);

    assert_eq!(test.name, "hi");
    assert_eq!(test.attr, Some("hey girl".to_owned()));
}

#[test]
#[should_fail]
fn test_xml_3() {
    let mut parser = EventReader::new_from_string("<test><attr>hey girl</attr></test>".to_owned());
    let mut iter = parser.events();
    let test = ::fromxml::parse_root::<TestStruct, ::std::io::MemReader>(&mut iter).unwrap();
    println!("test {}", test);
}

#[test]
fn test_attrs_1() {
    let mut parser = EventReader::new_from_string("<test name=\"hello\"/>".to_owned());
    let mut iter = parser.events();
    let test = ::fromxml::parse_root::<TestStruct, ::std::io::MemReader>(&mut iter).unwrap();
    println!("test {}", test);

    assert_eq!(test.name, "hello");
}
