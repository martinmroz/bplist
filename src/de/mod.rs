//
// Copyright 2020 bplist Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

mod date;
mod parser;
mod uid;

use serde::Deserialize;
use serde::de::{
    self,
    DeserializeSeed,
    IntoDeserializer,
    MapAccess,
    SeqAccess,
};

use std::collections::BTreeSet;
use std::vec;

use crate::document::{
    HEADER_SIZE,
    HEADER_VERSION_00,
    TRAILER_SIZE,
    OffsetTable,
    ObjectFormat,
};
use crate::error::{Error, Result};

#[derive(Clone, Eq, PartialEq, Debug)]
struct Metadata {
    /// The table of offsets from the start of the input to the location of a given object.
    offset_table: OffsetTable,
    /// The byte length of an object reference.
    object_reference_size: usize,
    /// The index of the root object to decode.
    root_object: usize,
    /// The range of bytes of the input where objects may reside.
    object_table_range: std::ops::Range<usize>,
}

impl Metadata {
    /// Utilizes the offset table to compute the offset of the given object.
    fn offset_of(&self, object: usize) -> Result<usize> {
        if object >= self.offset_table.len() {
            Err(Error::InvalidObjectReference)
        } else {
            Ok(self.offset_table[object])
        }
    }
}

/// Provides access to objects within the object table.
#[derive(Debug)]
struct ObjectTable<'a> {
    input: &'a [u8],
    metadata: Metadata,
}

/// Defines a basic parser with serde-compatible error handling.
macro_rules! define_parser {
    ($name:ident, $parser:expr, $type:ty, $expected_error:path) => {
        fn $name(&self, object: usize) -> Result<$type> {
            let data = self.data_for(object)?;
            $parser(data)
                .map(|(_, value)| value)
                .map_err(|_| $expected_error)
        }
    };
}

impl<'a> ObjectTable<'a> {

    /// Returns the slice of the input corresponding to the object.
    fn data_for(&self, object: usize) -> Result<&[u8]> {
        let offset = self.metadata.offset_of(object)?;

        // Make sure the offset is to a point within the object table.
        if !self.metadata.object_table_range.contains(&offset) {
            return Err(Error::InvalidOffsetToObject);
        }

        Ok(&self.input[offset .. ])
    }

    /// Parses the marker byte for the specified object and returns the format.
    fn kind_of(&self, object: usize) -> Result<ObjectFormat> {
        let data = self.data_for(object)?;
        parser::object::any_marker(data)
            .map(|(_, (format, _))| format)
            .map_err(|_| Error::InvalidOrUnsupportedObjectFormat)
    }

    define_parser![
        parse_boolean,
        parser::object::boolean,
        bool,
        Error::ExpectedBool
    ];
    define_parser![
        parse_fill,
        parser::object::fill,
        (),
        Error::ExpectedFill
    ];
    define_parser![
        parse_uint8,
        parser::object::uint8,
        u8,
        Error::ExpectedUInt8
    ];
    define_parser![
        parse_uint16,
        parser::object::uint16,
        u16,
        Error::ExpectedUInt16
    ];
    define_parser![
        parse_uint32,
        parser::object::uint32,
        u32,
        Error::ExpectedUInt32
    ];
    define_parser![
        parse_sint64,
        parser::object::sint64,
        i64,
        Error::ExpectedSInt64
    ];
    define_parser![
        parse_float32,
        parser::object::float32,
        f32,
        Error::ExpectedFloat32
    ];
    define_parser![
        parse_float64,
        parser::object::float64,
        f64,  
        Error::ExpectedFloat64
    ];
    define_parser![
        parse_date, 
        parser::object::date, 
        f64,
        Error::ExpectedDate
    ];
    define_parser![
        parse_data, 
        parser::object::data,
        &[u8],
        Error::ExpectedData
    ];
    define_parser![
        parse_ascii_string,
        parser::object::ascii_string,
        &str,
        Error::ExpectedAsciiString
    ];
    define_parser![
        parse_utf16_string,
        parser::object::utf16_string,
        String,
        Error::ExpectedAsciiString
    ];
    define_parser![
        parse_uid,
        parser::object::uid,
        &[u8],
        Error::ExpectedUid
    ];

    /// Parses an array of objects whose reference size is determined in metadata.
    fn parse_array(&self, object: usize) -> Result<Vec<usize>> {
        let data = self.data_for(object)?;
        parser::object::array(self.metadata.object_reference_size)(data)
            .map(|(_, objects)| objects)
            .map_err(|_| Error::ExpectedArray)
    }

    /// Parses an array of objects whose reference size is determined in metadata.
    fn parse_dictionary(&self, object: usize) -> Result<Vec<(usize, usize)>> {
        let data = self.data_for(object)?;
        parser::object::dictionary(self.metadata.object_reference_size)(data)
            .map(|(_, pairs)| pairs)
            .map_err(|_| Error::ExpectedDictionary)
    }

}

#[derive(Debug)]
pub struct Deserializer<'de> {
    /// The bytes which represent the totality of the input document.
    input: &'de [u8],
}

impl<'de> Deserializer<'de> {
    /// Designated initializer for a binary property list object deserializer.
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer { input }
    }
}

/// Support for deserializing any supported type from a binary property list document.
pub fn from_bytes<'a, T>(b: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(b);
    T::deserialize(&mut deserializer)
}

impl<'de> Deserializer<'de> {

    /// Parses the metadata necessary to interpret the contents of the document.
    /// 
    /// # Notes
    /// 
    /// The smallest possible document consists of:
    ///   1. A fixed-sized header.
    ///   2. An object table with a single one-byte entry such as a zero-length array.
    ///   3. An offset table with a single one-byte entry for the root object.
    ///   4. A fixed-sized trailer.
    fn parse_metadata(&self) -> Result<Metadata> {
        if self.input.len() < HEADER_SIZE + 2 + TRAILER_SIZE {
            return Err(Error::Eof);
        }

        // Parse the header and verify both the magic number and the version marker.
        let header_slice = &self.input[0 .. HEADER_SIZE];
        let (_, header) = parser::document::header(header_slice).map_err(|_| {
            Error::MissingOrInvalidHeader
        })?;
        if header.version != HEADER_VERSION_00 {
            return Err(Error::UnsupportedVersion);
        }

        // Parse the trailer from the end of the input and sanity check the fields.
        let trailer_slice = &self.input[self.input.len() - TRAILER_SIZE .. ];
        let (_, trailer) = parser::document::trailer(trailer_slice).map_err(|_| {
            Error::MissingOrInvalidTrailer
        })?;
        if trailer.root_object >= trailer.number_of_objects {
            return Err(Error::InvalidRootObject);
        }

        // Compute the location and length of the offset table.
        let offset_table_start = trailer.offset_table_offset;
        let offset_table_length = trailer.number_of_objects * trailer.offset_table_entry_size;
    
        // The offset table should not be defined as overlapping with the trailer.
        if (offset_table_start + offset_table_length) > (self.input.len() - TRAILER_SIZE) {
            return Err(Error::MissingOrInvalidOffsetTable);
        }

        // Parse the offset table.
        let offset_table_slice = &self.input[offset_table_start .. offset_table_start + offset_table_length];
        let (_, offset_table) = parser::document::offset_table(
            trailer.number_of_objects, 
            trailer.offset_table_entry_size
        )(offset_table_slice).map_err(|_| {
            Error::MissingOrInvalidOffsetTable
        })?;

        Ok(Metadata {
            offset_table,
            object_reference_size: trailer.object_reference_size,
            root_object: trailer.root_object,
            object_table_range: (HEADER_SIZE .. offset_table_start)
        })
    }

}

impl<'de, 'b> de::Deserializer<'de> for &'b mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // Parse the metadata and use it to create an object table.
        let metadata = self.parse_metadata()?;
        let object_table = ObjectTable {
            metadata,
            input: self.input,
        };

        // Verify the root object is an Array or Dictionary.
        let root_object = object_table.metadata.root_object;
        let root_object_type = object_table.kind_of(root_object)?;

        // Create a deserializer for the root object and forward the call.
        let mut deserializer = ObjectDeserializer::new(object_table, root_object);
        if root_object_type == ObjectFormat::Array {
            deserializer.deserialize_seq(visitor)
        } else if root_object_type == ObjectFormat::Dictionary {
            deserializer.deserialize_map(visitor)
        } else {
            Err(Error::RootObjectNotArrayOrDictionary)
        }
    }

    serde::forward_to_deserialize_any! {
        bool 
        u8 u16 u32 u64 u128
        i8 i16 i32 i64 i128
        f32 f64 
        char str string 
        seq map
        bytes byte_buf
        enum
        struct
        unit unit_struct
        tuple tuple_struct
        newtype_struct 
        ignored_any 
        option
        identifier
    }

}

#[derive(Debug)]
pub struct ObjectDeserializer<'de> {
    /// The bytes which represent the totality of the input document.
    object_table: ObjectTable<'de>,
    /// The index of the next object to process.
    next_object: usize,
    /// Ordered set of the collections being processed to detect cycles.
    collection_stack: BTreeSet<usize>,
}

impl<'de> ObjectDeserializer<'de> {

    /// Returns a new instance of the receiver for the specified object table and object.
    fn new(object_table: ObjectTable<'de>, next_object: usize) -> Self {
        ObjectDeserializer { 
            object_table,
            next_object,
            collection_stack: BTreeSet::new(),
        }
    }

    /// Sets the next object to process.
    fn set_next_object(&mut self, object: usize) {
        self.next_object = object
    }

    /// Pushes an object onto the collection stack to ensure no cycles can occur.
    #[must_use = "the result must be checked to avoid creating a cycle"]
    fn enter_collection(&mut self, object: usize) -> Result<()> {
        let already_visited = self.collection_stack.insert(object) == false;
        if already_visited {
            Err(Error::CycleDetected)
        } else {
            Ok(())
        }
    }

    /// Pops the most recently entered collection from the stack.
    fn exit_collection(&mut self) {
        assert!(self.collection_stack.is_empty() == false, "unbalanced calls in object stack tracking");
        let value = self.collection_stack
            .iter()
            .cloned()
            .next_back()
            .unwrap();
        self.collection_stack.remove(&value);
    }

}

impl<'de, 'b> de::Deserializer<'de> for &'b mut ObjectDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // Determine the format of the next object.
        let object = self.next_object;
        let format = self.object_table.kind_of(object)?;

        match format {

            // Parse basic object types.
            ObjectFormat::Boolean =>
                visitor.visit_bool(self.object_table.parse_boolean(object)?),
            ObjectFormat::UInt8 =>
                visitor.visit_u8(self.object_table.parse_uint8(object)?),
            ObjectFormat::UInt16 =>
                visitor.visit_u16(self.object_table.parse_uint16(object)?),
            ObjectFormat::UInt32 =>
                visitor.visit_u32(self.object_table.parse_uint32(object)?),
            ObjectFormat::SInt64 =>
                visitor.visit_i64(self.object_table.parse_sint64(object)?),
            ObjectFormat::Float32 =>
                visitor.visit_f32(self.object_table.parse_float32(object)?),
            ObjectFormat::Float64 =>
                visitor.visit_f64(self.object_table.parse_float64(object)?),
            ObjectFormat::Data =>
                visitor.visit_bytes(self.object_table.parse_data(object)?),
            ObjectFormat::AsciiString =>
                visitor.visit_str(self.object_table.parse_ascii_string(object)?),
            ObjectFormat::Utf16String =>
                visitor.visit_string(self.object_table.parse_utf16_string(object)?),

            // Fill bytes are interpreted as unit values.
            ObjectFormat::Fill => {
                let _ = self.object_table.parse_fill(object)?;
                visitor.visit_unit()
            }

            // A date object is deserialized as a Date type via map access object.
            ObjectFormat::Date => {
                let absolute_time = self.object_table.parse_date(object)?;
                let deserializer = DateDeserializer::new(absolute_time);
                visitor.visit_map(deserializer)
            }

            // A UID object is deserialized as a Uid type via map access object.
            ObjectFormat::Uid => {
                let bytes = self.object_table.parse_uid(object)?;
                let deserializer = UidDeserializer::new(Vec::from(bytes));
                visitor.visit_map(deserializer)
            }

            // Arrays are processed through a sequence access object.
            ObjectFormat::Array => {
                let objects = self.object_table.parse_array(object)?;

                // Track entering the array to detect reference cycles.
                self.enter_collection(object)?;
                let sequence = ArraySequence::new(&mut self, objects);
                let result = visitor.visit_seq(sequence);
                self.exit_collection();
                result
            }

            // Dictionaries are processed through a map access object.
            ObjectFormat::Dictionary => {
                let pairs = self.object_table.parse_dictionary(object)?;

                // Track the entering the dictionary to detect reference cycles.
                self.enter_collection(object)?;
                let map = DictionarySequence::new(&mut self, pairs);
                let result = visitor.visit_map(map);
                self.exit_collection();
                result
            }
        }
    }

    serde::forward_to_deserialize_any! {
        bool 
        u8 u16 u32 u64 u128
        i8 i16 i32 i64 i128
        f32 f64 
        char str string 
        seq map
        bytes byte_buf
        enum
        struct
        unit unit_struct
        tuple tuple_struct
        newtype_struct 
        ignored_any 
        option
        identifier
    }

}

/// Access object to process the elements in an Array.
struct ArraySequence<'a, 'de: 'a> {
    de: &'a mut ObjectDeserializer<'de>,
    objects: vec::IntoIter<usize>,
}

impl<'a, 'de> ArraySequence<'a, 'de> {
    fn new(de: &'a mut ObjectDeserializer<'de>, object_list: Vec<usize>) -> Self {
        ArraySequence {
            de,
            objects: object_list.into_iter()
        }
    }
}

impl<'de, 'a> SeqAccess<'de> for ArraySequence<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(object) = self.objects.next() {
            self.de.set_next_object(object);
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }
}

/// Access object used to process the elements in a Dictionary.
struct DictionarySequence<'a, 'de: 'a> {
    de: &'a mut ObjectDeserializer<'de>,
    key_value_pairs: vec::IntoIter<(usize, usize)>,
    current_pair: Option<(usize, usize)>,
}

impl<'a, 'de> DictionarySequence<'a, 'de> {
    fn new(de: &'a mut ObjectDeserializer<'de>, list: Vec<(usize, usize)>) -> Self {
        DictionarySequence {
            de,
            key_value_pairs: list.into_iter(),
            current_pair: None,
        }
    }
}

impl<'de, 'a> MapAccess<'de> for DictionarySequence<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        // Advance the iterator to obtain the next key-value pair.
        self.current_pair = self.key_value_pairs.next();

        // Point the deserializer at the key and deserialize it.
        if let Some((key, _)) = self.current_pair {
             self.de.set_next_object(key);
             seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // Point the deserializer at the value and deserialize it.
        let (_, value) = self.current_pair.unwrap();
        self.de.set_next_object(value);
        seed.deserialize(&mut *self.de)
    }
}

/// Access object to provide a Map around a Date-type pseudo-structure.
struct DateDeserializer {
    visited: bool,
    absolute_time: f64,
}

impl DateDeserializer {
    fn new(absolute_time: f64) -> Self {
        DateDeserializer {
            absolute_time,
            visited: false,
        }
    }
}

impl<'de> de::MapAccess<'de> for DateDeserializer {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de> {
        if self.visited {
            Ok(None)
        } else {
            self.visited = true;
            seed.deserialize(date::STRUCT_FIELD.into_deserializer()).map(Some)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de> {
        seed.deserialize(self.absolute_time.into_deserializer())
    }
}

/// Access object to provide a Map around a UID-type pseudo-structure.
struct UidDeserializer {
    visited: bool,
    data: Vec<u8>,
}

impl UidDeserializer {
    fn new(data: Vec<u8>) -> Self {
        UidDeserializer {
            data,
            visited: false,
        }
    }
}

impl<'de> de::MapAccess<'de> for UidDeserializer {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de> {
        if self.visited {
            Ok(None)
        } else {
            self.visited = true;
            seed.deserialize(uid::STRUCT_FIELD.into_deserializer()).map(Some)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de> {
        let data_to_yield = std::mem::replace(&mut self.data, vec![]);
        seed.deserialize(data_to_yield.into_deserializer())
    }
}
