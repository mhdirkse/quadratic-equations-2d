use std::rc::Rc;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::DebugList;
use num_rational::Rational32;
use std::fmt;

pub type Base = Rational32;

#[derive(Clone)]
#[derive(Debug)]
pub struct FieldTower {
    data: Rc<FieldTowerData>
}

pub struct FieldValue {
    context: FieldTower,
    value: RawFieldValue
}

#[derive(Debug)]
struct FieldTowerData {
    roots: Vec<RawFieldValue>
}

#[derive(Debug)]
enum RawFieldValue {
    BASIC(Base),
    EXTENDED(RawExtendedValue)
}

#[derive(Debug)]
struct RawExtendedValue {
    base: Box<RawFieldValue>,
    extension: Box<RawFieldValue>
}

impl FieldTower {
    pub fn new() -> Self {
        let data: FieldTowerData = FieldTowerData {
            roots: vec![]
        };
        return FieldTower {data: Rc::new(data)};
    }

    pub fn value(&self, base: Base) -> FieldValue {
        return FieldValue {
            context: FieldTower {data: self.data.clone()},
            value: RawFieldValue::BASIC(base)
        };
    }

    fn add_root(&mut self, value: FieldValue) {
        if value.context != *self {
            panic!("Cannot add root because of field tower mismatch");
        } else {
            self.data.roots.push(clone(&value.value));
        }
    }
}

impl PartialEq for FieldTower {
    fn eq(&self, other: &Self) -> bool {
        let my_data: &Rc<FieldTowerData> = &self.data;
        let other_data: &Rc<FieldTowerData> = &other.data;
        return Rc::<FieldTowerData>::ptr_eq(my_data, other_data);
    }
}

impl Eq for FieldTower {}

impl RawFieldValue {
    fn from_coefficients(coeffs: &[Base]) -> RawFieldValue {
        if coeffs.len() == 0 || !is_power_of_two(coeffs.len()) {
            panic!("Invalid number of coefficients: {}", coeffs.len());
        }
        if coeffs.len() == 1 {
            return RawFieldValue::BASIC(coeffs[0]);
        } else {
            let split_index: usize = coeffs.len() / 2;
            let base_coeffs = &coeffs[0..split_index];
            let extension_coeffs = &coeffs[split_index..];
            return RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::from_coefficients(base_coeffs)),
                extension: Box::new(RawFieldValue::from_coefficients(extension_coeffs))
            })
        }
    }

    fn num_extensions(&self) -> u32 {
        return match &self {
            RawFieldValue::BASIC(_) => 0,
            RawFieldValue::EXTENDED(value) => value.base.num_extensions() + 1
        }
    }
}

fn clone(value: &RawFieldValue) -> RawFieldValue {
    match value {
        RawFieldValue::BASIC(base) => RawFieldValue::BASIC(Base::clone(base)),
        RawFieldValue::EXTENDED(values) => {
            let clone_base: RawFieldValue = clone(&values.base);
            let clone_extension: RawFieldValue = clone(&values.extension);
            RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(clone_base),
                extension: Box::new(clone_extension)
            })
        }
    }
}

impl FieldValue {
    pub fn num_extensions(&self) -> u32 {
        return self.value.num_extensions();
    }
}

impl Debug for FieldValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value = &self.value;
        add_fmt_entries(value, self.context, "", &mut f);
        return Ok(());
    }
}

fn add_fmt_entries(value: &RawFieldValue, field_tower: FieldTower, suffix: &str, f: &mut Formatter<'_>) {
    match value {        
        RawFieldValue::BASIC(base) => {
            f.write_fmt(format_args!("{}", base));
        }
        RawFieldValue::EXTENDED(extended) => {
            let raw_root: &RawFieldValue = field_tower.data.roots[value.num_extensions()-1];
            let root_field_value: FieldValue = FieldValue {
                context: field_tower,
                value: clone(raw_root)
            };
            let root_suffix: String = format!("sqrt({})", root_field_value);
            add_fmt_entries(&extended.base, field_tower, "", &mut f);
            f.write_fmt(format_args!("+"));
            add_fmt_entries(&extended.extension, field_tower, &root_suffix, &mut f);
        }
    }
}

const NUM_BITS: u32 = (usize::MAX).count_ones();

fn is_power_of_two(n: usize) -> bool {
    if n == 0 {
        return true;
    } else {
        for p in 0..NUM_BITS {
            if n == 1 << p {
                return true;
            }
        }
        return false;
    }
}

#[cfg(test)]
mod test {
    use crate::field::{FieldTower, FieldValue, RawFieldValue};
    use num_rational::Rational32;

    #[test]
    fn field_tower_only_clones_are_equal() {
        let first: FieldTower = FieldTower::new();
        let second: FieldTower = FieldTower::new();
        let first_clone = first.clone();
        assert_eq!(first, first_clone);
        assert_ne!(first, second);
    }

    #[test]
    fn basic_value_has_no_extensions() {
        let tower: FieldTower = FieldTower::new();
        let basic_value = tower.value(Rational32::new(5, 2));
        assert_eq!(basic_value.num_extensions(), 0);
    }

    #[test]
    fn value_with_simple_root_has_num_extensions_one_and_can_be_formatted() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational32::new(2, 1)));
        let coefficients: Vec<Rational32> = vec![Rational32::new(3, 1), Rational32::new(5, 1)];
        let raw_test_value = RawFieldValue::from_coefficients(&coefficients[..]);
        let test_value: FieldValue = FieldValue {
            context: tower,
            value: raw_test_value
        };
        assert_eq!("[3,5sqrt(2)]", format!("{}", test_value));
    }
}