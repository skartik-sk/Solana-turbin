use std::{error::Error, marker::PhantomData};

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use wincode::{SchemaRead, SchemaWrite, config::DefaultConfig};

trait Serializer<T> {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn Error>>;
    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn Error>>;
}

struct Borsh;
struct Wincode;
struct Json;

impl<T: BorshDeserialize + BorshSerialize> Serializer<T> for Borsh {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn Error>> {
        borsh::to_vec(value).map_err(|e| e.into())
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn Error>> {
        borsh::from_slice(bytes).map_err(|e| e.into())
    }
}

impl<T: Serialize + DeserializeOwned> Serializer<T> for Json {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn Error>> {
        serde_json::to_vec(value).map_err(|e| e.into())
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn Error>> {
        serde_json::from_slice(bytes).map_err(|e| e.into())
    }
}

impl<T: SchemaWrite<DefaultConfig, Src = T> + for<'de> SchemaRead<'de, DefaultConfig, Dst = T>>
    Serializer<T> for Wincode
{
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn Error>> {
        wincode::serialize(value).map_err(|e| e.into())
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn Error>> {
        wincode::deserialize(bytes).map_err(|e| e.into())
    }
}

pub struct Storage<T, S> {
    data: Option<Vec<u8>>,
    serializer: S,
    _type: PhantomData<T>,
}

impl<T, S: Serializer<T>> Storage<T, S> {
    pub fn new(serializer: S) -> Self {
        Storage {
            data: None,
            serializer,
            _type: PhantomData,
        }
    }
    pub fn save(&mut self, value: &T) -> Result<(), Box<dyn Error>> {
        let bytes = self.serializer.to_bytes(value)?;
        
        self.data = Some(bytes);
        Ok(())
    }
    pub fn load(&self) -> Result<T, Box<dyn Error>> {
        match &self.data {
            Some(data) => self.serializer.from_bytes(data),
            None => Err("no data".into()),
        }

        //self.serializer.from_bytes(self.data)?;
    }
    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }
}

//5 pending
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Debug,SchemaWrite ,  SchemaRead)]
struct Person {
    pub color_hex: String,
    pub fav_num: u64,
}

//\/\/\/\/\/\/\/\Test/\/\/\/\/\/\/\

#[test]
fn for_borsh() {
    let per = Person {
        color_hex: "ffffff".to_string(),
        fav_num: 7,
    };
    let mut st = Storage::new(Borsh);
    assert_eq!(st.has_data(), false);
    st.save(&per).unwrap();
    assert_eq!(st.load().unwrap(), per)
}

#[test]
fn for_serde() {
    let per = Person {
        color_hex: "ffffff".to_string(),
        fav_num: 7,
    };
    let mut st = Storage::new(Json);
    assert_eq!(st.has_data(), false);
    st.save(&per).unwrap();
    assert_eq!(st.load().unwrap(), per)
}

#[test]
fn for_wincode() {
    let per = Person {
        color_hex: "ffffff".to_string(),
        fav_num: 7,
    };
    let mut st = Storage::new(Wincode);
    assert_eq!(st.has_data(), false);
    st.save(&per).unwrap();
    assert_eq!(st.load().unwrap(), per)
}
/*
*



6. Write Tests
Create unit tests that verify:
• Data can be saved and loaded with Borsh
• Data can be saved and loaded with Wincode
• Data can be saved and loaded with JSON
• Loaded data matches the original data
Learning Goals
By completing this challenge, you should understand:
• How to design and implement generic traits
• How to use PhantomData for zero-cost type tracking
• How trait bounds work with multiple serialization libraries
• The differences between various serialization formats
• Error handling with Result types
• How to write generic code that works with different implementations
Bonus Challenges (Optional)
If you want to extend the challenge:
1 Add a method to convert between different serializers
2 Add benchmarks to compare serialization performance
Expected Output
Your program should be able to run something like:

rust
let person = Person { name: "André".to_string(), age: 30 };
let mut borsh_storage = Storage::new(Borsh);
borsh_storage.save(&person).unwrap();
let loaded = borsh_storage.load().unwrap();
println!("Loaded: {:?}", loaded);
And successfully save/load data using all three serialization formats.
*/

fn main() {
    println!("Hello, world!");
}
