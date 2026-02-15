use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::Add;
use std::ops::Mul;
use num_rational::Rational32;
use num_traits::{CheckedAdd, CheckedMul};
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

#[derive(Clone)]
#[derive(Debug)]
enum RawFieldValue {
    BASIC(Base),
    EXTENDED(RawExtendedValue)
}

#[derive(Clone)]
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
            self.data.borrow_mut().roots.push(value.value.clone());
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

    fn checked_add(&self, other: &Self, context: &FieldTower) -> Option<Self> {
        return bin_op(self, other, context, checked_add_base, checked_add_extended);
    }

    fn checked_mul(&self, other: &Self, context: &FieldTower) -> Option<Self> {
        return bin_op(
            self,
            other,
            context,
            checked_mul_base,
            checked_mul_extended);
    }
}

fn bin_op(
    raw_first: &RawFieldValue,
    raw_second: &RawFieldValue,
    context: &FieldTower,
    base_op: fn(&Base, &Base, &FieldTower) -> Option<Base>,
    extended_op: fn(&RawExtendedValue, &RawExtendedValue, &FieldTower) -> Option<RawExtendedValue>)
    -> Option<RawFieldValue>
{
    let (first, second): (RawFieldValue, RawFieldValue) =
        equalize_num_extensions(raw_first, raw_second);
    match first {
        RawFieldValue::BASIC(base) => {
            match second {
                RawFieldValue::BASIC(other_base) => {
                    let result: Base = base_op(&base, &other_base, context)?;
                    return Option::Some(RawFieldValue::BASIC(result));
                }
                RawFieldValue::EXTENDED(_) => {
                    panic!("RawFieldValue::bin_op() extension depth mismatch: {} vs. {}",
                        first.num_extensions(), second.num_extensions());
                }
            }
        }
        RawFieldValue::EXTENDED(ref extended) => {
            match second {
                RawFieldValue::BASIC(_) => {
                    panic!("RawFieldValue::bin_op() extension depth mismatch: {} vs. {}",
                        first.num_extensions(), second.num_extensions());
                }
                RawFieldValue::EXTENDED(ref other_extended) => {
                    let result: RawExtendedValue = extended_op(extended, other_extended, context)?;
                    return Option::Some(RawFieldValue::EXTENDED(result));
                }
            }
        }
    }
}

fn checked_add_base(first: &Base, second: &Base, _: &FieldTower) -> Option<Base> {
    return first.checked_add(second);
}

fn checked_add_extended(first: &RawExtendedValue, second: &RawExtendedValue, context: &FieldTower) -> Option<RawExtendedValue> {
    let base: RawFieldValue = bin_op(
        &first.base,
        &second.base,
        context,
        checked_add_base,
        checked_add_extended)?;
    let extension: RawFieldValue = bin_op(
        &first.extension,
        &second.extension,
        context,
        checked_add_base,
        checked_add_extended
    )?;
    return Option::Some(RawExtendedValue {
        base: Box::new(base),
        extension: Box::new(extension)
    });
}

fn checked_mul_base(first: &Base, second: &Base, _: &FieldTower) -> Option<Base> {
    return first.checked_mul(second);
}

fn checked_mul_extended(first: &RawExtendedValue, second: &RawExtendedValue, context: &FieldTower) -> Option<RawExtendedValue> {
    let root_index: u32 = first.base.num_extensions();
    let root: &RawFieldValue = &context.data.borrow().roots[root_index as usize];
    let term1: RawFieldValue = bin_op(
        &first.base,
        &second.base,
        context,
        checked_mul_base,
        checked_mul_extended
    )?;
    let term2: RawFieldValue = bin_op(
        &first.base,
        &second.extension,
        context,
        checked_mul_base,
        checked_mul_extended
    )?;
    let term3: RawFieldValue = bin_op(
        &first.extension,
        &second.base,
        context,
        checked_mul_base,
        checked_mul_extended
    )?;
    let term4: RawFieldValue = bin_op(
        &first.extension,
        &second.extension,
        context,
        checked_mul_base,
        checked_mul_extended
    )?;
    let result_base: RawFieldValue = term1.checked_add(&term4.checked_mul(root, context)?, context)?;
    let result_extension: RawFieldValue = term2.checked_add(&term3, context)?;
    return Option::Some(RawExtendedValue {
        base: Box::new(result_base),
        extension: Box::new(result_extension)
    });
}

impl PartialEq for RawFieldValue {
    fn eq(&self, other: &Self) -> bool {
        let (first, second): (RawFieldValue, RawFieldValue) =
            equalize_num_extensions(&self, other);
        return compare_raw_field_values_impl(&first, &second);
    }
}

fn equalize_num_extensions(first: &RawFieldValue, second: &RawFieldValue) -> (RawFieldValue, RawFieldValue) {
    let num_extensions_first = first.num_extensions();
    let num_extensions_second = second.num_extensions();
    if num_extensions_first < num_extensions_second {
        return (promote_raw_field_value(&first, num_extensions_second), second.clone());
    } else if num_extensions_second < num_extensions_first {
        return (first.clone(), promote_raw_field_value(&second, num_extensions_first));
    } else {
        return (first.clone(), second.clone());
    }
}

fn compare_raw_field_values_impl(first: &RawFieldValue, second: &RawFieldValue) -> bool {
    match first {
        RawFieldValue::BASIC(base) => {
            match second {
                RawFieldValue::BASIC(otherBase) => return base == otherBase,
                RawFieldValue::EXTENDED(_) => {
                    panic!("compare_raw_field_values_impl(): Cannot happen because promote_raw_field_value() was applied");
                }
            }
        }
        RawFieldValue::EXTENDED(extended) => {
            match second {
                RawFieldValue::BASIC(_) => {
                    panic!("compare_raw_field_values_impl(): Cannot happen because promote_raw_field_value() was applied");
                }
                RawFieldValue::EXTENDED(other_extended) => {
                    return compare_raw_field_values_impl(&extended.base, &other_extended.base) &&
                        compare_raw_field_values_impl(&extended.extension, &other_extended.extension)
                }
            }
        }
    }
}

fn promote_raw_field_value(v: &RawFieldValue, num_required_extensions: u32) -> RawFieldValue {
    let num_existing_extensions = v.num_extensions();
    if num_required_extensions < num_existing_extensions {
        panic!("promoteRawFieldValue(): Cannot promote from {} to {} extensions", num_existing_extensions, num_required_extensions);
    } else if num_required_extensions == num_existing_extensions + 1 {
        return RawFieldValue::EXTENDED(RawExtendedValue {
            base: Box::new(v.clone()),
            extension: Box::new(raw_field_value_zero(num_existing_extensions))
        });
    } else {
        return promote_raw_field_value(&promote_raw_field_value(v, num_existing_extensions + 1), num_required_extensions);
    }
}

fn raw_field_value_zero(num_extensions: u32) -> RawFieldValue {
    if num_extensions == 0 {
        return RawFieldValue::BASIC(Rational32::new(0, 1));
    } else {
        return RawFieldValue::EXTENDED(RawExtendedValue {
            base: Box::new(raw_field_value_zero(num_extensions - 1)),
            extension: Box::new(raw_field_value_zero(num_extensions - 1))
        });
    }
}

impl Eq for RawFieldValue {}

impl FieldValue {
    pub fn num_extensions(&self) -> u32 {
        return self.value.num_extensions();
    }
}

impl PartialEq for FieldValue {
    fn eq(&self, other: &Self) -> bool {
        if self.context != other.context {
            panic!("FieldValue::eq() not allowed on value from different FieldTower instances");
        } else {
            return self.value == other.value;
        }
    }
}

impl Eq for FieldValue {}

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

impl Add for FieldValue {
    type Output = Self;

    fn add(self, _: Self) -> Self {
        panic!("Not implemented");
    }
}

impl CheckedAdd for FieldValue {
    fn checked_add(&self, other: &Self) -> Option<Self> {
        if self.context != other.context {
            panic!("FieldValue::checked_add not allowed on value from different FieldTower instances")
        }
        return Option::Some(FieldValue {
            context: self.context.clone(),
            value: self.value.checked_add(&other.value, &self.context)?,
        });
    }
}

impl Mul for FieldValue {
    type Output = Self;

    fn mul(self, _: Self) -> Self {
        panic!("Not implemented");
    }
}

impl CheckedMul for FieldValue {
    fn checked_mul(&self, other: &Self) -> Option<Self> {
        if self.context != other.context {
            panic!("FieldValue::checked_mul() not allowed on value from different FieldTower instances")
        }
        return Option::Some(FieldValue {
            context: self.context.clone(),
            value: self.value.checked_mul(&other.value, &self.context)?,
        });
    }
}

#[cfg(test)]
mod test {
    use crate::field::{FieldTower, FieldValue, RawFieldValue};
    use num_rational::Rational32;
    use num_traits::{CheckedAdd, CheckedMul};

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

    #[test]
    fn values_with_simple_root_can_be_added() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational32::new(2, 1)));
        let coefficients_1: Vec<Rational32> = vec![Rational32::new(3, 1), Rational32::new(5, 1)];
        let raw_test_value_1 = RawFieldValue::from_coefficients(&coefficients_1[..]);
        let test_value_1 = FieldValue {
            context: tower.clone(),
            value: raw_test_value_1
        };
        let coefficients_2: Vec<Rational32> = vec![Rational32::new(10, 1), Rational32::new(20, 1)];
        let raw_test_value_2 = RawFieldValue::from_coefficients(&coefficients_2[..]);
        let test_value_2 = FieldValue {
            context: tower.clone(),
            value: raw_test_value_2
        };
        let optional_result: Option<FieldValue> = test_value_1.checked_add(&test_value_2);
        let result = optional_result.expect("Unexpected overflow");
        let coefficients_expected: Vec<Rational32> = vec![Rational32::new(13, 1), Rational32::new(25, 1)];
        let raw_expected_value = RawFieldValue::from_coefficients(&coefficients_expected[..]);
        let expected_value = FieldValue {
            context: tower,
            value: raw_expected_value
        };
        assert_eq!(result.to_string(), expected_value.to_string());
        assert_eq!(result == expected_value, true);
    }

    #[test]
    fn when_adding_produces_overflow_than_none_returned() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational32::new(2, 1)));
        let big_base = FieldValue {
            context: tower.clone(),
            value: RawFieldValue::from_coefficients(&vec![Rational32::new(i32::MAX, 1), Rational32::new(1, 1)])
        };
        let big_extended = FieldValue {
            context: tower.clone(),
            value: RawFieldValue::from_coefficients(&vec![Rational32::new(1, 1), Rational32::new(i32::MAX, 1)])
        };
        let with_coefficients_one = FieldValue {
            context: tower.clone(),
            value: RawFieldValue::from_coefficients(&vec![Rational32::new(1, 1), Rational32::new(1, 1)])
        };
        assert_eq!(big_base.checked_add(&with_coefficients_one).is_none(), true);
        assert_eq!(with_coefficients_one.checked_add(&big_base).is_none(), true);
        assert_eq!(big_extended.checked_add(&with_coefficients_one).is_none(), true);
        assert_eq!(with_coefficients_one.checked_add(&big_extended).is_none(), true);
    }

    #[test]
    fn when_values_have_unequal_number_of_extensions_then_can_be_added() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational32::new(2, 1)));
        let extended: FieldValue = FieldValue {
            context: tower.clone(),
            value: RawFieldValue::from_coefficients(&vec![Rational32::new(3, 1), Rational32::new(5, 1)])
        };
        let simple: FieldValue = tower.value(Rational32::new(6, 1));
        let result_1: FieldValue = extended.checked_add(&simple).expect("No overflow expected");
        let result_2: FieldValue = simple.checked_add(&extended).expect("No overflow expected");
        let expected_str = "9+5*sqrt(2)";
        assert_eq!(result_1.to_string(), expected_str);
        assert_eq!(result_2.to_string(), expected_str);
    }

    #[test]
    fn values_with_simple_root_can_be_multiplied() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational32::new(2, 1)));
        let coefficients_1: Vec<Rational32> = vec![Rational32::new(3, 1), Rational32::new(5, 1)];
        let raw_test_value_1 = RawFieldValue::from_coefficients(&coefficients_1[..]);
        let test_value_1 = FieldValue {
            context: tower.clone(),
            value: raw_test_value_1
        };
        let coefficients_2: Vec<Rational32> = vec![Rational32::new(10, 1), Rational32::new(20, 1)];
        let raw_test_value_2 = RawFieldValue::from_coefficients(&coefficients_2[..]);
        let test_value_2 = FieldValue {
            context: tower.clone(),
            value: raw_test_value_2
        };
        let optional_result: Option<FieldValue> = test_value_1.checked_mul(&test_value_2);
        let result = optional_result.expect("Unexpected overflow");
        let coefficients_expected: Vec<Rational32> = vec![Rational32::new(230, 1), Rational32::new(110, 1)];
        let raw_expected_value = RawFieldValue::from_coefficients(&coefficients_expected[..]);
        let expected_value = FieldValue {
            context: tower,
            value: raw_expected_value
        };
        assert_eq!(result.to_string(), expected_value.to_string());
        assert_eq!(result == expected_value, true);
    }

    #[test]
    fn when_multiplying_produces_overflow_than_none_returned() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational32::new(2, 1)));
        let big_base = FieldValue {
            context: tower.clone(),
            value: RawFieldValue::from_coefficients(&vec![Rational32::new(i32::MAX, 1), Rational32::new(1, 1)])
        };
        let big_extended = FieldValue {
            context: tower.clone(),
            value: RawFieldValue::from_coefficients(&vec![Rational32::new(1, 1), Rational32::new(i32::MAX, 1)])
        };
        let multiplier = FieldValue {
            context: tower.clone(),
            value: RawFieldValue::from_coefficients(&vec![Rational32::new(2, 1), Rational32::new(2, 1)])
        };
        assert_eq!(big_base.checked_add(&multiplier).is_none(), true);
        assert_eq!(multiplier.checked_add(&big_base).is_none(), true);
        assert_eq!(big_extended.checked_add(&multiplier).is_none(), true);
        assert_eq!(multiplier.checked_add(&big_extended).is_none(), true);
    }

    #[test]
    fn when_values_have_unequal_number_of_extensions_then_can_be_multiplied() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational32::new(2, 1)));
        let extended: FieldValue = FieldValue {
            context: tower.clone(),
            value: RawFieldValue::from_coefficients(&vec![Rational32::new(3, 1), Rational32::new(5, 1)])
        };
        let simple: FieldValue = tower.value(Rational32::new(6, 1));
        let result_1: FieldValue = extended.checked_mul(&simple).expect("No overflow expected");
        let result_2: FieldValue = simple.checked_mul(&extended).expect("No overflow expected");
        let expected_str = "18+30*sqrt(2)";
        assert_eq!(result_1.to_string(), expected_str);
        assert_eq!(result_2.to_string(), expected_str);
    }
}