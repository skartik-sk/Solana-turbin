
#![feature(test)]

extern crate test;
use std::{ error::Error, fmt::Debug as db, marker::PhantomData};

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use wincode::{SchemaRead, SchemaWrite, config::DefaultConfig};

pub trait Serializer<T: db> {
    fn name(&self) -> &'static str;
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn Error>>;
    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn Error>>;
}

struct Borsh;
struct Wincode;
struct Json;

impl<T: db + BorshDeserialize + BorshSerialize> Serializer<T> for Borsh {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn Error>> {
        borsh::to_vec(value).map_err(|e| e.into())
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn Error>> {
        borsh::from_slice(bytes).map_err(|e| e.into())
    }

    fn name(&self) -> &'static str {
        "Borsh"
    }
}

impl<T: db + Serialize + DeserializeOwned> Serializer<T> for Json {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn Error>> {
        serde_json::to_vec(value).map_err(|e| e.into())
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn Error>> {
        serde_json::from_slice(bytes).map_err(|e| e.into())
    }

    fn name(&self) -> &'static str {
        "Serde_Json"
    }
}

impl<T: db + SchemaWrite<DefaultConfig, Src = T> + for<'de> SchemaRead<'de, DefaultConfig, Dst = T>>
    Serializer<T> for Wincode
{
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, Box<dyn Error>> {
        wincode::serialize(value).map_err(|e| e.into())
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn Error>> {
        wincode::deserialize(bytes).map_err(|e| e.into())
    }

    fn name(&self) -> &'static str {
        "WinCode"
    }
}

pub struct Storage<T, S> {
    data: Option<Vec<u8>>,
    serializer: S,
    _type: PhantomData<T>,
}

impl<T: db, S: Serializer<T>> Storage<T, S> {
    pub fn new(serializer: S) -> Self {
        Storage {
            data: None,
            serializer,
            _type: PhantomData,
        }
    }
    pub fn save(&mut self, value: &T) -> Result<(), Box<dyn Error>> {
        let bytes = self.serializer.to_bytes(value)?;
        println!("to_bytes {:?} : \n  {:?}", self.serializer.name(), bytes);
        self.data = Some(bytes);
        Ok(())
    }
    pub fn load(&self) -> Result<T, Box<dyn Error>> {
        match &self.data {
            Some(data) => {
                let obj = self.serializer.from_bytes(data);
                println!("From_bytes {:?} : \n  {:?}", self.serializer.name(), obj);
                obj
            }
            None => Err("no data".into()),
        }

        //self.serializer.from_bytes(self.data)?;
    }
    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }
}

//5 pending
#[derive(
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    SchemaWrite,
    SchemaRead,
)]
struct Person {
    pub color_hex: String,
    pub fav_num: u64,
}

//\/\/\/\/\/\/\/\Test/\/\/\/\/\/\/\

#[test]
pub fn for_borsh() {
    let per = Person {
        color_hex: "ffffff/000000".to_string(),
        fav_num: 6,
    };
    let mut st = Storage::new(Borsh);
    assert_eq!(st.has_data(), false);
    st.save(&per).unwrap();
    assert_eq!(st.load().unwrap(), per)
}

#[test]
pub fn for_serde() {
    let per = Person {
        color_hex: "ffffff/000000".to_string(),
        fav_num: 6,
    };
    let mut st = Storage::new(Json);
    assert_eq!(st.has_data(), false);
    st.save(&per).unwrap();
    assert_eq!(st.load().unwrap(), per)
}

#[test]
pub fn for_wincode() {
    let per = Person {
        color_hex: "ffffff/000000".to_string(),
        fav_num: 6,
    };
    let mut st = Storage::new(Wincode);
    assert_eq!(st.has_data(), false);
    st.save(&per).unwrap();
    assert_eq!(st.load().unwrap(), per)
}


//\/\/\/\//\\/\\\/\\/ bench \/\/\/\/\/\/\

#[bench]
fn bench_borsh_serialize(b: &mut test::Bencher) {
    let per = Person {
        color_hex: "ffffff".to_string(),
        fav_num: 702496809348,
    };
    
    b.iter(|| {            
        let mut storage = Storage::new(Borsh);
        storage.save(&per).unwrap();

    });
}



#[bench]
fn bench_json_serialize(b: &mut test::Bencher) {
    let per = Person {
        color_hex: "ffffff".to_string(),
        fav_num: 702496809348,
    };
    
    b.iter(|| {

        let mut storage = Storage::new(Json);
        storage.save(&per).unwrap();

    });
    
            
       
}

#[bench]
fn bench_wincode_serialize(b: &mut test::Bencher) {
    let per = Person {
        color_hex: "ffffff".to_string(),
        fav_num: 702496809348,
    };
    
    b.iter(|| {
            
        let mut storage = Storage::new(Wincode);
        storage.save(&per).unwrap();

    });
    
            
       
}


#[bench]
fn bench_borsh_deserialize(b: &mut test::Bencher) {
    let per = Person {
        color_hex: "ffffff".to_string(),
        fav_num: 702496809348,
    };
    
    let mut storage = Storage::new(Borsh);
    storage.save(&per).unwrap();
    
    b.iter(|| {

        storage.load().unwrap();
    })
}

#[bench]
fn bench_json_deserialize(b: &mut test::Bencher) {
    let per = Person {
        color_hex: "ffffff".to_string(),
        fav_num: 702496809348,
    };
    
    let mut storage = Storage::new(Json);
    storage.save(&per).unwrap();
    
    b.iter(|| {

        storage.load().unwrap();
    })
}

#[bench]
fn bench_wincode_deserialize(b: &mut test::Bencher) {
    let per = Person {
        color_hex: "ffffff".to_string(),
        fav_num: 702496809348,
    };
    
    let mut storage = Storage::new(Wincode);
    storage.save(&per).unwrap();

    

    b.iter(|| {            
        storage.load().unwrap();

    })
}

/*
*


Bonus Challenges (Optional)
If you want to extend the challenge:
1 Add a method to convert between different serializers

*/

fn main() {
    println!("Serialize and deserialize!");
    // for_borsh();
    // for_serde();
    // for_wincode();
}
