use {
    derive_more::{From, Into},
    minicbor::{
        decode,
        encode::{Error, Write},
        Decode, Decoder, Encode, Encoder,
    },
};

#[derive(Clone, Debug, Eq, From, Into, PartialEq)]
pub struct Key(Vec<u8>);

#[derive(Clone, Debug, Eq, From, Into, PartialEq)]
pub struct Value(Vec<u8>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Proof {
    Child,
    KeyValueHash(Vec<u8>),
    KeyValuePair(Key, Value),
    NodeHash(Vec<u8>),
    Parent,
}

impl<C> Encode<C> for Proof {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, _: &mut C) -> Result<(), Error<W::Error>> {
        use Proof::{Child, KeyValueHash, KeyValuePair, NodeHash, Parent};
        match &self {
            Child => e.u8(0x11),
            KeyValueHash(hash) => e.array(2).and_then(|e| e.u8(1)).and_then(|e| e.bytes(hash)),
            KeyValuePair(Key(key), Value(value)) => e
                .array(3)
                .and_then(|e| e.u8(3))
                .and_then(|e| e.bytes(key))
                .and_then(|e| e.bytes(value)),
            NodeHash(hash) => e.array(2).and_then(|e| e.u8(2)).and_then(|e| e.bytes(hash)),
            Parent => e.u8(0x10),
        }
        .map(|_| ())
    }
}

impl<'d, C> Decode<'d, C> for Proof {
    fn decode(d: &mut Decoder<'d>, _: &mut C) -> Result<Self, decode::Error> {
        use {
            minicbor::data::Type::{
                Array, ArrayIndef, Bool, Break, Bytes, BytesIndef, Int, Map, MapIndef, Null,
                Simple, String, StringIndef, Tag, Undefined, Unknown, F16, F32, F64, I16, I32, I64,
                I8, U16, U32, U64, U8,
            },
            Proof::{Child, KeyValueHash, KeyValuePair, NodeHash, Parent},
        };
        d.datatype().and_then(|datatype| match datatype {
            Array | ArrayIndef => match d.array() {
                Err(_) | Ok(None) => Err(decode::Error::message(
                    "Error parsing array type into array",
                )),
                Ok(Some(array_length)) if array_length == 2 => {
                    d.u8().and_then(|value| match value {
                        1 => d.bytes().map(|hash| KeyValueHash(hash.to_vec())),
                        2 => d.bytes().map(|hash| NodeHash(hash.to_vec())),
                        variant => Err(decode::Error::unknown_variant(variant.into())),
                    })
                }
                Ok(Some(array_length)) if array_length == 3 => {
                    d.u8().and_then(|_| d.bytes()).and_then(|key| {
                        d.bytes()
                            .map(|value| KeyValuePair(key.to_vec().into(), value.to_vec().into()))
                    })
                }
                Ok(Some(array_length)) => Err(decode::Error::message(format!(
                    "Unexpected array size {array_length}"
                ))),
            },
            U8 => d.u8().and_then(|value| match value {
                0x10 => Ok(Parent),
                0x11 => Ok(Child),
                variant => Err(decode::Error::unknown_variant(variant.into())),
            }),
            Bool | Break | Bytes | BytesIndef | F16 | F32 | F64 | I8 | I16 | I32 | I64 | Int
            | Map | MapIndef | Null | Simple | String | StringIndef | Tag | U16 | U32 | U64
            | Undefined | Unknown(_) => Err(decode::Error::message("Expected either array or u8")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Proof;
    #[test]
    fn round_trip_parent() -> Result<(), ()> {
        assert_eq!(
            minicbor::decode::<Proof>(minicbor::to_vec(Proof::Parent).map_err(|_| ())?.as_slice())
                .map_err(|_| ())?,
            Proof::Parent
        );
        Ok(())
    }

    #[test]
    fn round_trip_child() -> Result<(), ()> {
        assert_eq!(
            minicbor::decode::<Proof>(minicbor::to_vec(Proof::Child).map_err(|_| ())?.as_slice())
                .map_err(|_| ())?,
            Proof::Child
        );
        Ok(())
    }

    #[test]
    fn round_trip_key_value_hash() -> Result<(), ()> {
        let key_value_hash = Proof::KeyValueHash(vec![1, 2, 3]);
        assert_eq!(
            minicbor::decode::<Proof>(
                minicbor::to_vec(key_value_hash.clone())
                    .map_err(|_| ())?
                    .as_slice()
            )
            .map_err(|_| ())?,
            key_value_hash
        );
        Ok(())
    }

    #[test]
    fn round_trip_node_hash() -> Result<(), ()> {
        let node_hash = Proof::KeyValueHash(vec![1, 2, 3]);
        assert_eq!(
            minicbor::decode::<Proof>(
                minicbor::to_vec(node_hash.clone())
                    .map_err(|_| ())?
                    .as_slice()
            )
            .map_err(|_| ())?,
            node_hash
        );
        Ok(())
    }

    #[test]
    fn round_trip_key_value_pair() -> Result<(), ()> {
        let key_value_pair = Proof::KeyValuePair(vec![1, 2, 3].into(), vec![4, 5, 6].into());
        assert_eq!(
            minicbor::decode::<Proof>(
                minicbor::to_vec(key_value_pair.clone())
                    .map_err(|_| ())?
                    .as_slice()
            )
            .map_err(|_| ())?,
            key_value_pair
        );
        Ok(())
    }
}