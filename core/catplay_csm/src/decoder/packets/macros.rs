#[macro_export]
#[cfg(feature = "inventory")]
macro_rules! register_packet_type {
    ($id:expr, $ty:ty) => {
        inventory::submit! {
            $crate::decoder::CsmPacketRegistry::as_registration::<$ty>($id)
        }
    };
}

#[macro_export]
macro_rules! packet_type {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident : $id:tt {
            $($field:tt)*
        }
    ) => {

        $crate::csm_struct! {
            $(#[$meta])*
            $vis struct $name { $($field)* }
        }

        impl $crate::decoder::CsmPacketId for $name {
            const PACKET_ID: u16 = $id;
        }

        #[cfg(feature = "inventory")]
        $crate::register_packet_type!($id, $name);
    };
}

#[macro_export]
macro_rules! group_type {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($field:tt)*
        }
    ) => {
        $crate::csm_struct! {
            $(#[$meta])*
            $vis struct $name { $($field)* }
        }
    };
}

#[macro_export]
macro_rules! enum_type {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $first_key:ident = $first_val:expr,
            $($key:ident = $val:expr),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[repr(u8)]
        #[derive(Debug, PartialEq, Clone)]
        $vis enum $name {
            $first_key = $first_val,
            $($key = $val),*
        }

        impl Default for $name {
            fn default() -> Self {
                Self::$first_key
            }
        }

        impl core::convert::TryFrom<u8> for $name {
            type Error = ();

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $first_val => Ok(Self::$first_key),
                    $($val => Ok(Self::$key),)*
                    _ => Err(()),
                }
            }
        }

        impl From<$name> for u8 {
            fn from(e: $name) -> u8 {
                e as u8
            }
        }

        impl $crate::decoder::CsmDecode for $name {
            fn decode_from_bytes(data: &[u8]) -> Self {
                let byte = data.get(0).copied().unwrap_or(0);
                let default = Self::default();
                let tried = Self::try_from(byte);
                match tried {
                    Err(_) => default,
                    Ok(val) => val,
                }
            }
        }

        impl $crate::decoder::CsmEncode for $name {
            fn encode_param(&self, id: u16, out: &mut $crate::decoder::CsmWriter) {
                let val: u8 = self.clone().into();
                out.write_tlv(id, &val.to_be_bytes());
            }
        }
    };
}

#[macro_export]
macro_rules! csm_struct {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                #[csm_id($id:expr)]
                $( #[$attrs:meta] )*
                $field_vis:vis $field:ident : $ty:ty
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, PartialEq, Default, Clone)]
        $(#[$meta])*
        $vis struct $name {
            $(
                $( #[$attrs] )*
                $field_vis $field : $ty,
            )*
        }

        impl $crate::decoder::CsmParamDecode for $name {
            #[allow(unused)]
            fn feed_param(&mut self, param: &$crate::decoder::CsmParam) {
                #[allow(unused_imports)]
                use $crate::decoder::CsmAccum;

                match param.id {
                    $(
                        $id => {
                            self.$field.add_param(&param);
                        }
                    )*
                    _ => {}
                }
            }

            #[allow(unused)]
            fn prealloc(&mut self, reader: &$crate::decoder::CsmScanner) {
                $(
                    let size = reader.count_repeating($id);
                    $crate::decoder::CsmAccum::prealloc(&mut self.$field, size);
                )*
            }
        }

        impl $crate::decoder::CsmParamEncode for $name {
            #[allow(unused)]
            fn encode_to_params(&self, out: &mut $crate::decoder::CsmWriter) {
                #[allow(unused_imports)]
                use $crate::decoder::CsmEncode;

                $(
                    self.$field.encode_param($id, out);
                )*
            }
        }

        impl AsRef<dyn $crate::decoder::CsmPacket> for $name {
            fn as_ref(&self) -> &dyn $crate::decoder::CsmPacket {
                self
            }
        }
    };
}
