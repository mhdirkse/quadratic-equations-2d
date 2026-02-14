use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Debug;
use num_rational::Rational32;

pub type Base = Rational32;

#[derive(Clone)]
#[derive(Debug)]
pub struct FieldTower {
    data: Rc<RefCell<FieldTowerData>>
}

#[derive(Debug)]
struct FieldTowerData {
    roots: Vec<RawFieldValue>
}

pub struct FieldValue {
    context: FieldTower,
    value: RawFieldValue
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
        let data: FieldTowerData = FieldTowerData {roots: vec![]};
        return FieldTower {data: Rc::new(RefCell::new(data))};
    }

    pub fn value(&self, base: Base) -> FieldValue {
        return FieldValue {
            context: self.clone(),
            value: RawFieldValue::BASIC(base)
        };
    }

    fn add_root(&mut self, value: FieldValue) {
        if value.context != *self {
            panic!("Cannot add root because of field tower mismatch");
        } else {
            (*self.data).borrow_mut().roots.push(clone(&value.value));
        }
    }
}

impl PartialEq for FieldTower {
    fn eq(&self, other: &Self) -> bool {
        let my_data: &Rc<RefCell<FieldTowerData>> = &self.data;
        let other_data: &Rc<RefCell<FieldTowerData>> = &other.data;
        return Rc::<RefCell<FieldTowerData>>::ptr_eq(my_data, other_data);
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

impl ToString for FieldValue {
    fn to_string(&self) -> String {
        let value = &self.value;
        let context = &self.context;
        return to_string(&value, &context);
    }
}

fn to_string(value: &RawFieldValue, context: &FieldTower) -> String {
    match value {
        RawFieldValue::BASIC(base) => base.to_string(),
        RawFieldValue::EXTENDED(extended) => {
            let base_str = to_string(&extended.base, &context);
            let extended_coeff_str = to_string(&extended.extension, &context);
            let root_index = value.num_extensions() as usize - 1;
            let root_of: &RawFieldValue = &context.data.borrow().roots[root_index];
            let root_of_str = to_string(&root_of, &context);
            return format!("{}+{}*sqrt({})", base_str, extended_coeff_str, root_of_str);
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
        assert_eq!("3+5*sqrt(2)", test_value.to_string());
    }
}