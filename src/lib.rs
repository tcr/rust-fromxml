#![feature(globs, macro_rules)]
#![macro_escape]

extern crate xml;

use std::io::{File, BufferedReader};
use std::num::{FromStrRadix};

use xml::reader::Events;
use xml::reader::events::XmlEvent::*;

#[macro_export]
macro_rules! deriving_fromxml {
    (
        $($(#[$attr:meta])* struct $Id:ident {
            $($(#[$Flag_field:meta])* $Flag:ident:$T:ty),+,
        })+
    ) => {
        $($(#[$attr])*
        #[deriving(Default, Show, Clone)]
        #[allow(non_snake_case)]
        pub struct $Id {
            $($(#[$Flag_field])* pub $Flag:$T,)+
        }

        impl ::fromxml::Placeholder for $Id {
            fn hold() -> Option<$Id> {
                return None
            }
            fn assign(field:&mut $Id, value:$Id) {
                *field = value;
            }
        }

        impl ::fromxml::FromXml for $Id {
            fn from_xml<'a>(iter:&'a mut ::xml::reader::Events<::std::io::BufferedReader<::std::io::File>>) -> Option<$Id> {
                #[allow(non_snake_case)]
                struct TempStruct {
                    $($(#[$Flag_field])* $Flag:Option<$T>,)+
                };

                let stage = TempStruct {
                    $($(#[$Flag_field])* $Flag: ::fromxml::Placeholder::hold(),)+
                };

                fn inner<'a> (iter:&'a mut ::xml::reader::Events<::std::io::BufferedReader<::std::io::File>>, arg:&mut TempStruct, name:&str) {
                    match name {
                        $($(#[$Flag_field])* stringify!($Flag) => ::fromxml::Placeholder::assign(&mut arg.$Flag, ::fromxml::FromXml::from_xml(iter)),)+
                        _ => ::fromxml::skip_node(iter),
                    };
                }

                let collected = ::fromxml::collect(iter, stage, inner);
                match collected {
                    None => return None,
                    _ => {},
                }
                let stage = collected.unwrap();

                let mut obj = $Id { ..Default::default() };

                $($(#[$Flag_field])* 
                match stage.$Flag {
                    None => panic!("Expected xml field \"{}\" for struct \"{}\"", stringify!($Flag), stringify!($Id)),
                    Some(arg) => obj.$Flag = arg,
                }
                )+

                Some(obj)
            }
        }
        )+
    };
}

pub trait Placeholder {
    fn hold() -> Option<Self>;
    fn assign(field:&mut Self, value:Self) {
        *field = value;
    }
}

impl<T:Placeholder + Clone> Placeholder for Option<T> {
    fn hold() -> Option<Option<T>> {
        return Some(None)
    }
    fn assign(field:&mut Option<T>, value:Option<T>) {
        let mut result;
        match field {
            &Some(ref ptr) => {
                if let Some(val) = value {
                    let mut left:T = (*ptr).clone();
                    Placeholder::assign(&mut left, val);
                    result = Some(left);
                } else {
                    result = value;
                }
            },
            _ => {
                result = value;
            },
        }
        *field = result;
    }
}

impl<T:Clone> Placeholder for Vec<T> {
    fn hold() -> Option<Vec<T>> {
        return Some(vec![])
    }
    fn assign(field:&mut Vec<T>, value:Vec<T>) {
        field.push_all(value.as_slice());
    }
}

impl Placeholder for u32 {
    fn hold() -> Option<u32> {
        return None
    }
    fn assign(field:&mut u32, value:u32) {
        *field = value;
    }
}

impl Placeholder for uint {
    fn hold() -> Option<uint> {
        return None
    }
    fn assign(field:&mut uint, value:uint) {
        *field = value;
    }
}

impl Placeholder for String {
    fn hold() -> Option<String> {
        return None
    }
    fn assign(field:&mut String, value:String) {
        *field = value;
    }
}

pub type XmlIter<'a> = Events<'a, BufferedReader<File>>;

pub fn collect<'a, T>(iter:&'a mut XmlIter, mut arg:T, back:for<'b> fn(&'b mut XmlIter, &mut T, &str)) -> Option<T> {
    loop {
        match iter.next() {
            Some(StartElement { name, attributes: _, namespace: _ }) => {
                back(iter, &mut arg, name.local_name.as_slice());
            }
            Some(EndElement { name: _ }) => break,
            Some(Error(e)) => {
                println!("Error: {}", e);
                return None;
            }
            Some(_) => {}
            None => return None
        }
    }

    return Some(arg);
}

pub fn skip_node<'a>(iter:&'a mut XmlIter) {
    let mut depth:uint = 1;
    loop {
        match iter.next() {
            Some(StartElement { name: _, attributes: _, namespace: _ }) => {
                depth = depth + 1;
            }
            Some(EndElement { name: _ }) => {
                depth = depth - 1;
                if depth == 0 {
                    return;
                }
            }
            Some(Error(e)) => {
                println!("Error: {}", e);
                return;
            }
            Some(..) => {}
            None => return

        }
    }
}

pub trait FromXml {
    fn from_xml<'a>(iter:&'a mut XmlIter) -> Option<Self>;
}

impl<T:FromXml> FromXml for Vec<T> {
    fn from_xml<'a>(iter:&'a mut XmlIter) -> Option<Vec<T>> {
        let mut ret:Vec<T> = vec![];
        ret.push(FromXml::from_xml(iter).unwrap());
        Some(ret)
    }
}

impl<T:FromXml> FromXml for Option<T> {
    fn from_xml<'a>(iter:&'a mut XmlIter) -> Option<Option<T>> {
        Some(FromXml::from_xml(iter))
    }
}

impl FromXml for uint {
    fn from_xml<'a>(iter:&'a mut XmlIter) -> Option<uint> {
        FromXml::from_xml(iter).and_then(|s: String| {
            if s.contains_char('x') {
                FromStrRadix::from_str_radix(&*s.slice_from(2), 16)
            } else {
                from_str(&*s)
            }
        })
    }
}

impl FromXml for u32 {
    fn from_xml<'a>(iter:&'a mut XmlIter) -> Option<u32> {
        FromXml::from_xml(iter).and_then(|s: String| {
            if s.contains_char('x') {
                FromStrRadix::from_str_radix(&*s.slice_from(2), 16)
            } else {
                from_str(&*s)
            }
        })
    }
}

impl FromXml for String {
    fn from_xml<'a>(iter:&'a mut XmlIter) -> Option<String> {
        let mut str = "".to_string();
        loop {
            match iter.next() {
                Some(Characters(text)) => str.push_str(text.as_slice()),
                Some(_) => break,
                None => return None,
            }
        }
        Some(str)
    }
}

pub fn parse_root<'a, T:FromXml>(iter:&'a mut XmlIter) -> Option<T> {
    loop {
        match iter.next() {
            Some(StartElement { name: _, attributes: _, namespace: _ }) => {
                return FromXml::from_xml(iter);
            }
            Some(Error(e)) => {
                println!("Error: {}", e);
                return None;
            }
            Some(..) => {}
            None => return None
        }
    }
}
