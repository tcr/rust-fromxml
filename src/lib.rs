#![feature(globs, macro_rules)]
#![macro_escape]

extern crate xml;

use std::io::Buffer;
use std::num::FromStrRadix;

use xml::reader::Events;
use xml::reader::events::XmlEvent::*;
use xml::attribute::OwnedAttribute as Attribute;

#[macro_export]
macro_rules! derive_fromxml {
    (
        $($(#[$attr:meta])* struct $Id:ident {
            $($(#[$Flag_field:meta])* $Flag:ident:$T:ty),+,
        })+
    ) => {
        $($(#[$attr])*
        #[derive(Default, Show, Clone)]
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
            fn from_xml<'a, B: ::std::io::Buffer>(iter:&'a mut ::xml::reader::Events<B>, attributes:Vec<::xml::attribute::OwnedAttribute>) -> Option<$Id> {
                #[allow(non_snake_case)]
                struct TempStruct {
                    $($(#[$Flag_field])* $Flag:Option<$T>,)+
                };

                let mut stage = TempStruct {
                    $($(#[$Flag_field])* $Flag: ::fromxml::Placeholder::hold(),)+
                };

                for attr in attributes.iter() {
                    match attr.name.local_name.as_slice() {
                        $($(#[$Flag_field])* stringify!($Flag) => stage.$Flag = ::fromxml::Placeholder::assign_attr(attr.value.clone()),)+
                        _ => {},
                    };
                }

                fn inner<'a, B: ::std::io::Buffer> (iter:&'a mut ::xml::reader::Events<B>, arg:&mut TempStruct, name:&str, attributes:Vec<::xml::attribute::OwnedAttribute>) {
                    match name {
                        $($(#[$Flag_field])* stringify!($Flag) => ::fromxml::Placeholder::assign(&mut arg.$Flag, ::fromxml::FromXml::from_xml(iter, attributes)),)+
                        _ => ::fromxml::skip_node(iter),
                    };
                }

                let collected = ::fromxml::collect(iter, stage, inner);
                match collected {
                    None => return None,
                    _ => {},
                }
                let stage = collected.unwrap();

                let mut obj = $Id { ..::std::default::Default::default() };

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

pub trait Placeholder : Sized {
    fn hold() -> Option<Self> {
        None
    }
    fn assign(field:&mut Self, value:Self) {
        *field = value;
    }
    fn assign_attr(_:String) -> Option<Self> {
        None
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

    fn assign_attr(value:String) -> Option<Option<T>> {
        Some(Placeholder::assign_attr(value))
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
}

impl Placeholder for uint {
}

impl Placeholder for String {
    fn assign_attr(value:String) -> Option<String> {
        Some(value)
    }
}

// E0122: pub type XmlIter<'a, B:Buffer> = Events<'a, B>;
pub type XmlIter<'a, B> = Events<'a, B>;

pub fn collect<'a, T, B:Buffer>(iter:&'a mut XmlIter<B>, mut arg:T, back:for<'b> fn(&'b mut XmlIter<B>, &mut T, &str, Vec<Attribute>)) -> Option<T> {
    loop {
        match iter.next() {
            Some(StartElement { name, attributes, namespace: _ }) => {
                back(iter, &mut arg, name.local_name.as_slice(), attributes);
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

pub fn skip_node<'a, B:Buffer>(iter:&'a mut XmlIter<B>) {
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
    fn from_xml<'a, B:Buffer>(iter:&'a mut XmlIter<B>, attributes:Vec<Attribute>) -> Option<Self>;
}

impl<T:FromXml> FromXml for Vec<T> {
    fn from_xml<'a, B:Buffer>(iter:&'a mut XmlIter<B>, attributes:Vec<Attribute>) -> Option<Vec<T>> {
        let mut ret:Vec<T> = vec![];
        ret.push(FromXml::from_xml(iter, attributes).unwrap());
        Some(ret)
    }
}

impl<T:FromXml> FromXml for Option<T> {
    fn from_xml<'a, B:Buffer>(iter:&'a mut XmlIter<B>, attributes:Vec<Attribute>) -> Option<Option<T>> {
        Some(FromXml::from_xml(iter, attributes))
    }
}

impl FromXml for uint {
    fn from_xml<'a, B:Buffer>(iter:&'a mut XmlIter<B>, attributes:Vec<Attribute>) -> Option<uint> {
        FromXml::from_xml(iter, attributes).and_then(|s: String| {
            if s.contains_char('x') {
                FromStrRadix::from_str_radix(&*s.slice_from(2), 16)
            } else {
                s.parse()
            }
        })
    }
}

impl FromXml for u32 {
    fn from_xml<'a, B:Buffer>(iter:&'a mut XmlIter<B>, attributes:Vec<Attribute>) -> Option<u32> {
        FromXml::from_xml(iter, attributes).and_then(|s: String| {
            if s.contains_char('x') {
                FromStrRadix::from_str_radix(&*s.slice_from(2), 16)
            } else {
                s.parse()
            }
        })
    }
}

impl FromXml for String {
    fn from_xml<'a, B:Buffer>(iter:&'a mut XmlIter<B>, _:Vec<Attribute>  ) -> Option<String> {
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

pub fn parse_root<'a, T:FromXml, B:Buffer>(iter:&'a mut XmlIter<B>) -> Option<T> {
    loop {
        match iter.next() {
            Some(StartElement { name: _, attributes, namespace: _ }) => {
                return FromXml::from_xml(iter, attributes);
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
