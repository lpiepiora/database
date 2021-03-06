// Copyright 2020 Alex Dukhno
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use protocol::sql_types::PostgreSqlType;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum SqlType {
    Bool,
    Char(u64),
    VarChar(u64),
    Decimal,
    SmallInt,
    Integer,
    BigInt,
    Real,
    DoublePrecision,
    Time,
    TimeWithTimeZone,
    Timestamp,
    TimestampWithTimeZone,
    Date,
    Interval,
}

impl SqlType {
    pub fn constraint(&self) -> Box<dyn Constraint> {
        match *self {
            Self::Char(length) => Box::new(CharSqlTypeConstraint { length }),
            Self::VarChar(length) => Box::new(VarCharSqlTypeConstraint { length }),
            Self::SmallInt => Box::new(SmallIntTypeConstraint),
            Self::Integer => Box::new(IntegerSqlTypeConstraint),
            Self::BigInt => Box::new(BigIntTypeConstraint),
            sql_type => unimplemented!("Type constraint for {:?} is not currently implemented", sql_type),
        }
    }

    pub fn serializer(&self) -> Box<dyn Serializer> {
        match *self {
            Self::Char(_length) => Box::new(CharSqlTypeSerializer),
            Self::VarChar(_length) => Box::new(VarCharSqlTypeSerializer),
            Self::SmallInt => Box::new(SmallIntTypeSerializer),
            Self::Integer => Box::new(IntegerSqlTypeSerializer),
            Self::BigInt => Box::new(BigIntTypeSerializer),
            sql_type => unimplemented!("Type Serializer for {:?} is not currently implemented", sql_type),
        }
    }

    pub fn to_pg_types(&self) -> PostgreSqlType {
        match *self {
            Self::Bool => PostgreSqlType::Bool,
            Self::Char(_) => PostgreSqlType::Char,
            Self::VarChar(_) => PostgreSqlType::VarChar,
            Self::Decimal => PostgreSqlType::Decimal,
            Self::SmallInt => PostgreSqlType::SmallInt,
            Self::Integer => PostgreSqlType::Integer,
            Self::BigInt => PostgreSqlType::BigInt,
            Self::Real => PostgreSqlType::Real,
            Self::DoublePrecision => PostgreSqlType::DoublePrecision,
            Self::Time => PostgreSqlType::Time,
            Self::TimeWithTimeZone => PostgreSqlType::TimeWithTimeZone,
            Self::Timestamp => PostgreSqlType::Timestamp,
            Self::TimestampWithTimeZone => PostgreSqlType::TimestampWithTimeZone,
            Self::Date => PostgreSqlType::Date,
            Self::Interval => PostgreSqlType::Interval,
        }
    }
}

pub trait Constraint {
    fn validate(&self, in_value: &str) -> Result<(), ConstraintError>;
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ConstraintError {
    OutOfRange,
    NotAnInt,
    ValueTooLong,
}

pub trait Serializer {
    fn ser(&self, in_value: &str) -> Vec<u8>;

    fn des(&self, out_value: &[u8]) -> String;
}

struct SmallIntTypeConstraint;

impl Constraint for SmallIntTypeConstraint {
    fn validate(&self, in_value: &str) -> Result<(), ConstraintError> {
        match lexical::parse::<i16, _>(in_value) {
            Ok(_) => Ok(()),
            Err(e) if e.code == lexical::ErrorCode::InvalidDigit => Err(ConstraintError::NotAnInt),
            Err(_) => Err(ConstraintError::OutOfRange),
        }
    }
}

struct SmallIntTypeSerializer;

impl Serializer for SmallIntTypeSerializer {
    #[allow(clippy::match_wild_err_arm)]
    fn ser(&self, in_value: &str) -> Vec<u8> {
        match lexical::parse::<i16, _>(in_value) {
            Ok(parsed) => parsed.to_be_bytes().to_vec(),
            Err(_) => unimplemented!(),
        }
    }

    fn des(&self, out_value: &[u8]) -> String {
        i16::from_be_bytes(out_value[0..2].try_into().unwrap()).to_string()
    }
}

struct IntegerSqlTypeConstraint;

impl Constraint for IntegerSqlTypeConstraint {
    fn validate(&self, in_value: &str) -> Result<(), ConstraintError> {
        match lexical::parse::<i32, _>(in_value) {
            Ok(_) => Ok(()),
            Err(e) if e.code == lexical::ErrorCode::InvalidDigit => Err(ConstraintError::NotAnInt),
            Err(_) => Err(ConstraintError::OutOfRange),
        }
    }
}

struct IntegerSqlTypeSerializer;

impl Serializer for IntegerSqlTypeSerializer {
    #[allow(clippy::match_wild_err_arm)]
    fn ser(&self, in_value: &str) -> Vec<u8> {
        match lexical::parse::<i32, _>(in_value) {
            Ok(parsed) => parsed.to_be_bytes().to_vec(),
            Err(_) => unimplemented!(),
        }
    }

    fn des(&self, out_value: &[u8]) -> String {
        i32::from_be_bytes(out_value[0..4].try_into().unwrap()).to_string()
    }
}

struct BigIntTypeConstraint;

impl Constraint for BigIntTypeConstraint {
    fn validate(&self, in_value: &str) -> Result<(), ConstraintError> {
        match lexical::parse::<i64, _>(in_value) {
            Ok(_) => Ok(()),
            Err(e) if e.code == lexical::ErrorCode::InvalidDigit => Err(ConstraintError::NotAnInt),
            Err(_) => Err(ConstraintError::OutOfRange),
        }
    }
}

struct BigIntTypeSerializer;

impl Serializer for BigIntTypeSerializer {
    #[allow(clippy::match_wild_err_arm)]
    fn ser(&self, in_value: &str) -> Vec<u8> {
        match lexical::parse::<i64, _>(in_value) {
            Ok(parsed) => parsed.to_be_bytes().to_vec(),
            Err(_) => unimplemented!(),
        }
    }

    fn des(&self, out_value: &[u8]) -> String {
        i64::from_be_bytes(out_value[0..8].try_into().unwrap()).to_string()
    }
}

struct CharSqlTypeConstraint {
    length: u64,
}

impl Constraint for CharSqlTypeConstraint {
    fn validate(&self, in_value: &str) -> Result<(), ConstraintError> {
        let trimmed = in_value.trim_end();
        if trimmed.len() > self.length as usize {
            Err(ConstraintError::ValueTooLong)
        } else {
            Ok(())
        }
    }
}

struct CharSqlTypeSerializer;

impl Serializer for CharSqlTypeSerializer {
    fn ser(&self, in_value: &str) -> Vec<u8> {
        in_value.trim_end().as_bytes().to_vec()
    }

    fn des(&self, out_value: &[u8]) -> String {
        String::from_utf8(out_value.to_vec()).unwrap()
    }
}

struct VarCharSqlTypeConstraint {
    length: u64,
}

impl Constraint for VarCharSqlTypeConstraint {
    fn validate(&self, in_value: &str) -> Result<(), ConstraintError> {
        let trimmed = in_value.trim_end();
        if trimmed.len() > self.length as usize {
            Err(ConstraintError::ValueTooLong)
        } else {
            Ok(())
        }
    }
}

struct VarCharSqlTypeSerializer;

impl Serializer for VarCharSqlTypeSerializer {
    fn ser(&self, in_value: &str) -> Vec<u8> {
        in_value.trim_end().as_bytes().to_vec()
    }

    fn des(&self, out_value: &[u8]) -> String {
        String::from_utf8(out_value.to_vec()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod to_postgresql_type_conversion {
        use crate::SqlType;
        use protocol::sql_types::PostgreSqlType;

        #[test]
        fn boolean() {
            assert_eq!(SqlType::Bool.to_pg_types(), PostgreSqlType::Bool);
        }

        #[test]
        fn small_int() {
            assert_eq!(SqlType::SmallInt.to_pg_types(), PostgreSqlType::SmallInt);
        }

        #[test]
        fn integer() {
            assert_eq!(SqlType::Integer.to_pg_types(), PostgreSqlType::Integer);
        }

        #[test]
        fn big_int() {
            assert_eq!(SqlType::BigInt.to_pg_types(), PostgreSqlType::BigInt);
        }

        #[test]
        fn char() {
            assert_eq!(SqlType::Char(0).to_pg_types(), PostgreSqlType::Char);
            assert_eq!(SqlType::Char(10).to_pg_types(), PostgreSqlType::Char);
            assert_eq!(SqlType::Char(100).to_pg_types(), PostgreSqlType::Char);
        }

        #[test]
        fn var_char() {
            assert_eq!(SqlType::VarChar(0).to_pg_types(), PostgreSqlType::VarChar);
            assert_eq!(SqlType::VarChar(10).to_pg_types(), PostgreSqlType::VarChar);
            assert_eq!(SqlType::VarChar(100).to_pg_types(), PostgreSqlType::VarChar);
        }

        #[test]
        fn decimal() {
            assert_eq!(SqlType::Decimal.to_pg_types(), PostgreSqlType::Decimal);
        }

        #[test]
        fn real() {
            assert_eq!(SqlType::Real.to_pg_types(), PostgreSqlType::Real);
        }

        #[test]
        fn double_precision() {
            assert_eq!(SqlType::DoublePrecision.to_pg_types(), PostgreSqlType::DoublePrecision);
        }

        #[test]
        fn time() {
            assert_eq!(SqlType::Time.to_pg_types(), PostgreSqlType::Time);
        }

        #[test]
        fn time_with_time_zone() {
            assert_eq!(
                SqlType::TimeWithTimeZone.to_pg_types(),
                PostgreSqlType::TimeWithTimeZone
            );
        }

        #[test]
        fn timestamp() {
            assert_eq!(SqlType::Timestamp.to_pg_types(), PostgreSqlType::Timestamp);
        }

        #[test]
        fn timestamp_with_timezone() {
            assert_eq!(
                SqlType::TimestampWithTimeZone.to_pg_types(),
                PostgreSqlType::TimestampWithTimeZone
            );
        }

        #[test]
        fn date() {
            assert_eq!(SqlType::Date.to_pg_types(), PostgreSqlType::Date);
        }

        #[test]
        fn interval() {
            assert_eq!(SqlType::Interval.to_pg_types(), PostgreSqlType::Interval);
        }
    }

    #[cfg(test)]
    mod ints {
        use super::*;

        #[cfg(test)]
        mod small {
            use super::*;

            #[cfg(test)]
            mod serialization {
                use super::*;

                #[rstest::fixture]
                fn serializer() -> Box<dyn Serializer> {
                    SqlType::SmallInt.serializer()
                }

                #[rstest::rstest]
                fn serialize(serializer: Box<dyn Serializer>) {
                    assert_eq!(serializer.ser("1"), vec![0, 1])
                }

                #[rstest::rstest]
                fn deserialize(serializer: Box<dyn Serializer>) {
                    assert_eq!(serializer.des(&[0, 1]), "1".to_owned())
                }
            }

            #[cfg(test)]
            mod validation {
                use super::*;

                #[rstest::fixture]
                fn constraint() -> Box<dyn Constraint> {
                    SqlType::SmallInt.constraint()
                }

                #[rstest::rstest]
                fn in_range(constraint: Box<dyn Constraint>) {
                    assert_eq!(constraint.validate("1"), Ok(()));
                    assert_eq!(constraint.validate("32767"), Ok(()));
                    assert_eq!(constraint.validate("-32768"), Ok(()));
                }

                #[rstest::rstest]
                fn greater_than_max(constraint: Box<dyn Constraint>) {
                    assert_eq!(constraint.validate("32769"), Err(ConstraintError::OutOfRange))
                }

                #[rstest::rstest]
                fn less_than_min(constraint: Box<dyn Constraint>) {
                    assert_eq!(constraint.validate("-32769"), Err(ConstraintError::OutOfRange))
                }

                #[rstest::rstest]
                fn a_float_number(constraint: Box<dyn Constraint>) {
                    assert_eq!(constraint.validate("-3276.9"), Err(ConstraintError::NotAnInt))
                }

                #[rstest::rstest]
                fn a_string(constraint: Box<dyn Constraint>) {
                    assert_eq!(constraint.validate("str"), Err(ConstraintError::NotAnInt))
                }
            }
        }

        #[cfg(test)]
        mod integer {
            use super::*;

            #[cfg(test)]
            mod serialization {
                use super::*;

                #[rstest::fixture]
                fn serializer() -> Box<dyn Serializer> {
                    SqlType::Integer.serializer()
                }

                #[rstest::rstest]
                fn serialize(serializer: Box<dyn Serializer>) {
                    assert_eq!(serializer.ser("1"), vec![0, 0, 0, 1])
                }

                #[rstest::rstest]
                fn deserialize(serializer: Box<dyn Serializer>) {
                    assert_eq!(serializer.des(&[0, 0, 0, 1]), "1".to_owned())
                }
            }

            #[cfg(test)]
            mod validation {
                use super::*;

                #[rstest::fixture]
                fn constraint() -> Box<dyn Constraint> {
                    SqlType::Integer.constraint()
                }

                #[rstest::rstest]
                fn in_range(constraint: Box<dyn Constraint>) {
                    assert_eq!(constraint.validate("1"), Ok(()));
                    assert_eq!(constraint.validate("-2147483648"), Ok(()));
                    assert_eq!(constraint.validate("2147483647"), Ok(()));
                }

                #[rstest::rstest]
                fn greater_than_max(constraint: Box<dyn Constraint>) {
                    assert_eq!(constraint.validate("2147483649"), Err(ConstraintError::OutOfRange))
                }

                #[rstest::rstest]
                fn less_than_min(constraint: Box<dyn Constraint>) {
                    assert_eq!(constraint.validate("-2147483649"), Err(ConstraintError::OutOfRange))
                }

                #[rstest::rstest]
                fn a_float_number(constraint: Box<dyn Constraint>) {
                    assert_eq!(constraint.validate("-214748.3649"), Err(ConstraintError::NotAnInt))
                }

                #[rstest::rstest]
                fn a_string(constraint: Box<dyn Constraint>) {
                    assert_eq!(constraint.validate("str"), Err(ConstraintError::NotAnInt))
                }
            }
        }

        #[cfg(test)]
        mod big_int {
            use super::*;

            #[cfg(test)]
            mod serialization {
                use super::*;

                #[rstest::fixture]
                fn serializer() -> Box<dyn Serializer> {
                    SqlType::BigInt.serializer()
                }

                #[rstest::rstest]
                fn serialize(serializer: Box<dyn Serializer>) {
                    assert_eq!(serializer.ser("1"), vec![0, 0, 0, 0, 0, 0, 0, 1])
                }

                #[rstest::rstest]
                fn deserialize(serializer: Box<dyn Serializer>) {
                    assert_eq!(serializer.des(&[0, 0, 0, 0, 0, 0, 0, 1]), "1".to_owned())
                }
            }

            #[cfg(test)]
            mod validation {
                use super::*;

                #[rstest::fixture]
                fn constraint() -> Box<dyn Constraint> {
                    SqlType::BigInt.constraint()
                }

                #[rstest::rstest]
                fn in_range(constraint: Box<dyn Constraint>) {
                    assert_eq!(constraint.validate("1"), Ok(()));
                    assert_eq!(constraint.validate("-9223372036854775808"), Ok(()));
                    assert_eq!(constraint.validate("9223372036854775807"), Ok(()));
                }

                #[rstest::rstest]
                fn greater_than_max(constraint: Box<dyn Constraint>) {
                    assert_eq!(
                        constraint.validate("9223372036854775809"),
                        Err(ConstraintError::OutOfRange)
                    )
                }

                #[rstest::rstest]
                fn less_than_min(constraint: Box<dyn Constraint>) {
                    assert_eq!(
                        constraint.validate("-9223372036854775809"),
                        Err(ConstraintError::OutOfRange)
                    )
                }

                #[rstest::rstest]
                fn a_float_number(constraint: Box<dyn Constraint>) {
                    assert_eq!(constraint.validate("-3276.9"), Err(ConstraintError::NotAnInt))
                }

                #[rstest::rstest]
                fn a_string(constraint: Box<dyn Constraint>) {
                    assert_eq!(constraint.validate("str"), Err(ConstraintError::NotAnInt))
                }
            }
        }
    }

    #[cfg(test)]
    mod strings {
        use super::*;

        #[cfg(test)]
        mod chars {
            use super::*;

            #[cfg(test)]
            mod serialization {
                use super::*;

                #[rstest::fixture]
                fn serializer() -> Box<dyn Serializer> {
                    SqlType::Char(10).serializer()
                }

                #[rstest::rstest]
                fn serialize(serializer: Box<dyn Serializer>) {
                    assert_eq!(serializer.ser("str"), vec![115, 116, 114])
                }

                #[rstest::rstest]
                fn deserialize(serializer: Box<dyn Serializer>) {
                    assert_eq!(serializer.des(&[115, 116, 114]), "str".to_owned())
                }
            }

            #[cfg(test)]
            mod validation {
                use super::*;

                #[rstest::fixture]
                fn constraint() -> Box<dyn Constraint> {
                    SqlType::Char(10).constraint()
                }

                #[rstest::rstest]
                fn in_length(constraint: Box<dyn Constraint>) {
                    assert_eq!(constraint.validate("1"), Ok(()))
                }

                #[rstest::rstest]
                fn too_long(constraint: Box<dyn Constraint>) {
                    assert_eq!(
                        constraint.validate("1".repeat(20).as_str()),
                        Err(ConstraintError::ValueTooLong)
                    )
                }
            }
        }

        #[cfg(test)]
        mod var_chars {
            use super::*;

            #[cfg(test)]
            mod serialization {
                use super::*;

                #[rstest::fixture]
                fn serializer() -> Box<dyn Serializer> {
                    SqlType::VarChar(10).serializer()
                }

                #[rstest::rstest]
                fn serialize(serializer: Box<dyn Serializer>) {
                    assert_eq!(serializer.ser("str"), vec![115, 116, 114])
                }

                #[rstest::rstest]
                fn deserialize(serializer: Box<dyn Serializer>) {
                    assert_eq!(serializer.des(&[115, 116, 114]), "str".to_owned())
                }
            }

            #[cfg(test)]
            mod validation {
                use super::*;

                #[rstest::fixture]
                fn constraint() -> Box<dyn Constraint> {
                    SqlType::VarChar(10).constraint()
                }

                #[rstest::rstest]
                fn in_length(constraint: Box<dyn Constraint>) {
                    assert_eq!(constraint.validate("1"), Ok(()))
                }

                #[rstest::rstest]
                fn too_long(constraint: Box<dyn Constraint>) {
                    assert_eq!(
                        constraint.validate("1".repeat(20).as_str()),
                        Err(ConstraintError::ValueTooLong)
                    )
                }
            }
        }
    }
}
