use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Debug;
use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Div};
use num_rational::Rational32;
use num_traits::{CheckedAdd, CheckedSub, CheckedMul, CheckedDiv};
use crate::sqrt::sqrt;

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

enum RawSqrtResult {
    NOSOLUTION,
    SOLUTION(RawFieldValue),
    EXTENSION(RawNewRootRequest),
}

struct RawNewRootRequest {
    coefficient: RawFieldValue,
    root: RawFieldValue
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

impl FieldValue {
    pub fn num_extensions(&self) -> u32 {
        return self.value.num_extensions();
    }

    pub fn compare_to_zero(&self) -> Option<Ordering> {
        return self.value.compare_to_zero(&self.context);
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
        return bin_op(self, other, RawFieldValue::checked_add);
    }
}

impl Sub for FieldValue {
    type Output = Self;

    fn sub(self, _: Self) -> Self {
        panic!("Not implemented");
    }
}

impl CheckedSub for FieldValue {
    fn checked_sub(&self, other: &Self) -> Option<Self> {
        return bin_op(self, other, RawFieldValue::checked_sub);
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
        return bin_op(self, other, RawFieldValue::checked_mul);
    }
}

impl Div for FieldValue {
    type Output = Self;

    fn div(self, _: Self) -> Self {
        panic!("Not implemented");
    }
}

impl CheckedDiv for FieldValue {
    fn checked_div(&self, other: &Self) -> Option<Self> {
        return bin_op(self, other, RawFieldValue::checked_div);
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
        return RawFieldValue::to_string(&value, &context);
    }
}

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
        return RawFieldValue::bin_op(self, other, context, RawFieldValue::checked_add_base, RawFieldValue::checked_add_extended);
    }

    fn checked_add_base(first: &Base, second: &Base, _: &FieldTower) -> Option<Base> {
        return first.checked_add(second);
    }

    fn checked_add_extended(first: &RawExtendedValue, second: &RawExtendedValue, context: &FieldTower) -> Option<RawExtendedValue> {
        let base: RawFieldValue = RawFieldValue::bin_op(
            &first.base,
            &second.base,
            context,
            RawFieldValue::checked_add_base,
            RawFieldValue::checked_add_extended)?;
        let extension: RawFieldValue = RawFieldValue::bin_op(
            &first.extension,
            &second.extension,
            context,
            RawFieldValue::checked_add_base,
            RawFieldValue::checked_add_extended
        )?;
        return Option::Some(RawExtendedValue {
            base: Box::new(base),
            extension: Box::new(extension)
        });
    }

    fn checked_sub(&self, other: &Self, context: &FieldTower) -> Option<Self> {
        return RawFieldValue::bin_op(self, other, context, RawFieldValue::checked_sub_base, RawFieldValue::checked_sub_extended);
    }

    fn checked_sub_base(first: &Base, second: &Base, _: &FieldTower) -> Option<Base> {
        return first.checked_sub(second);
    }

    fn checked_sub_extended(first: &RawExtendedValue, second: &RawExtendedValue, context: &FieldTower) -> Option<RawExtendedValue> {
        let base: RawFieldValue = RawFieldValue::bin_op(
            &first.base,
            &second.base,
            context,
            RawFieldValue::checked_sub_base,
            RawFieldValue::checked_sub_extended)?;
        let extension: RawFieldValue = RawFieldValue::bin_op(
            &first.extension,
            &second.extension,
            context,
            RawFieldValue::checked_sub_base,
            RawFieldValue::checked_sub_extended
        )?;
        return Option::Some(RawExtendedValue {
            base: Box::new(base),
            extension: Box::new(extension)
        });
    }

    fn checked_mul(&self, other: &Self, context: &FieldTower) -> Option<Self> {
        return RawFieldValue::bin_op(
            self,
            other,
            context,
            RawFieldValue::checked_mul_base,
            RawFieldValue::checked_mul_extended);
    }

    fn checked_mul_base(first: &Base, second: &Base, _: &FieldTower) -> Option<Base> {
        return first.checked_mul(second);
    }

    fn checked_mul_extended(first: &RawExtendedValue, second: &RawExtendedValue, context: &FieldTower) -> Option<RawExtendedValue> {
        let root_index: u32 = first.base.num_extensions();
        let root: &RawFieldValue = &context.data.borrow().roots[root_index as usize];
        let term1: RawFieldValue = RawFieldValue::bin_op(
            &first.base,
            &second.base,
            context,
            RawFieldValue::checked_mul_base,
            RawFieldValue::checked_mul_extended
        )?;
        let term2: RawFieldValue = RawFieldValue::bin_op(
            &first.base,
            &second.extension,
            context,
            RawFieldValue::checked_mul_base,
            RawFieldValue::checked_mul_extended
        )?;
        let term3: RawFieldValue = RawFieldValue::bin_op(
            &first.extension,
            &second.base,
            context,
            RawFieldValue::checked_mul_base,
            RawFieldValue::checked_mul_extended
        )?;
        let term4: RawFieldValue = RawFieldValue::bin_op(
            &first.extension,
            &second.extension,
            context,
            RawFieldValue::checked_mul_base,
            RawFieldValue::checked_mul_extended
        )?;
        let result_base: RawFieldValue = term1.checked_add(&term4.checked_mul(root, context)?, context)?;
        let result_extension: RawFieldValue = term2.checked_add(&term3, context)?;
        return Option::Some(RawExtendedValue {
            base: Box::new(result_base),
            extension: Box::new(result_extension)
        });
    }

    fn checked_div(&self, other: &Self, context: &FieldTower) -> Option<Self> {
        return RawFieldValue::bin_op(
            self,
            other,
            context,
            RawFieldValue::checked_div_base,
            RawFieldValue::checked_div_extended);
    }

    fn checked_div_base(first: &Base, second: &Base, _: &FieldTower) -> Option<Base> {
        return first.checked_div(second);
    }

    fn checked_div_extended(num: &RawExtendedValue, den: &RawExtendedValue, context: &FieldTower) -> Option<RawExtendedValue> {
        let root_index: usize = num.base.num_extensions() as usize;
        let root: &RawFieldValue = &context.data.borrow().roots[root_index];
        // Multiply num and den with den.base - den.extended * sqrt(...).
        let multiplier: RawFieldValue = RawFieldValue::EXTENDED(RawExtendedValue {
            base: den.base.clone(),
            extension: Box::new(den.extension.checked_neg()?)
        });
        let result_num: RawFieldValue = RawFieldValue::EXTENDED(num.clone()).checked_mul(&multiplier, context)?;
        let lower_level_den: RawFieldValue = RawFieldValue::pseudo_norm(&den, &root, context)?;
        let result: RawExtendedValue = match result_num {
            RawFieldValue::BASIC(_) => panic!("Expected EXTENDED because was constructed that way"),
            RawFieldValue::EXTENDED(num) => RawExtendedValue {
                base: Box::new(num.base.checked_div(&lower_level_den, context)?),
                extension: Box::new(num.extension.checked_div(&lower_level_den, context)?)
            }
        };
        return Option::Some(result);
    }

    fn checked_neg(&self) -> Option<RawFieldValue> {
        let result: RawFieldValue = match self {
            RawFieldValue::BASIC(base) =>
                RawFieldValue::BASIC(Base::new(0, 1).checked_sub(&base)?),
            RawFieldValue::EXTENDED(extended) =>
                RawFieldValue::EXTENDED(RawExtendedValue {
                    base: Box::new(extended.base.checked_neg()?),
                    extension: Box::new(extended.extension.checked_neg()?)
                })
        };
        return Option::Some(result);
    }

    fn pseudo_norm(v: &RawExtendedValue, root: &RawFieldValue, context: &FieldTower) -> Option<RawFieldValue> {
        let term1 = v.base.checked_mul(&v.base, context)?;
        let term2 = v.extension.checked_mul(&
            v.extension.checked_mul(root, context)?, context)?;
        let result: RawFieldValue = term1.checked_sub(&term2, context)?;
        return Option::Some(result);
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
            RawFieldValue::equalize_num_extensions(raw_first, raw_second);
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

    fn equalize_num_extensions(first: &RawFieldValue, second: &RawFieldValue) -> (RawFieldValue, RawFieldValue) {
        let num_extensions_first = first.num_extensions();
        let num_extensions_second = second.num_extensions();
        if num_extensions_first < num_extensions_second {
            return (RawFieldValue::promote(&first, num_extensions_second), second.clone());
        } else if num_extensions_second < num_extensions_first {
            return (first.clone(), RawFieldValue::promote(&second, num_extensions_first));
        } else {
            return (first.clone(), second.clone());
        }
    }

    fn compare_impl(first: &RawFieldValue, second: &RawFieldValue) -> bool {
        match first {
            RawFieldValue::BASIC(base) => {
                match second {
                    RawFieldValue::BASIC(otherBase) => return base == otherBase,
                    RawFieldValue::EXTENDED(_) => {
                        panic!("RawFieldValue::compare_impl(): Cannot happen because RawFieldValue::promote() was applied");
                    }
                }
            }
            RawFieldValue::EXTENDED(extended) => {
                match second {
                    RawFieldValue::BASIC(_) => {
                        panic!("RawFieldValue::compare_impl(): Cannot happen because RawFieldValue::promote() was applied");
                    }
                    RawFieldValue::EXTENDED(other_extended) => {
                        return RawFieldValue::compare_impl(&extended.base, &other_extended.base) &&
                            RawFieldValue::compare_impl(&extended.extension, &other_extended.extension)
                    }
                }
            }
        }
    }

    fn promote(v: &RawFieldValue, num_required_extensions: u32) -> RawFieldValue {
        let num_existing_extensions = v.num_extensions();
        if num_required_extensions < num_existing_extensions {
            panic!("promote(): Cannot promote from {} to {} extensions", num_existing_extensions, num_required_extensions);
        } else if num_required_extensions == num_existing_extensions + 1 {
            return RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(v.clone()),
                extension: Box::new(RawFieldValue::zero(num_existing_extensions))
            });
        } else {
            return RawFieldValue::promote(
                &RawFieldValue::promote(
                    v,
                    num_existing_extensions + 1
                ),
                num_required_extensions
            );
        }
    }

    fn zero(num_extensions: u32) -> RawFieldValue {
        if num_extensions == 0 {
            return RawFieldValue::BASIC(Rational32::new(0, 1));
        } else {
            return RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::zero(num_extensions - 1)),
                extension: Box::new(RawFieldValue::zero(num_extensions - 1))
            });
        }
    }

    fn compare_to_zero(&self, context: &FieldTower) -> Option<Ordering> {
        match self {
            RawFieldValue::BASIC(base) => Option::Some(RawFieldValue::compare_basic_to_zero(&base)),
            RawFieldValue::EXTENDED(extended) => RawFieldValue::compare_extended_to_zero(&extended, context)
        }
    }

    fn compare_basic_to_zero(base: &Base) -> Ordering {
        if base == &Base::new(0, 1) {
            return Ordering::Equal
        } else if base < &Base::new(0, 1) {
            return Ordering::Less
        } else {
            return Ordering::Greater
        }
    }

    fn compare_extended_to_zero(extended: &RawExtendedValue, context: &FieldTower) -> Option<Ordering> {
        match extended.base.compare_to_zero(context)? {
            Ordering::Equal => extended.extension.compare_to_zero(context),
            Ordering::Less =>
                match extended.extension.compare_to_zero(context)? {
                    Ordering::Less => Option::Some(Ordering::Less),
                    Ordering::Equal => Option::Some(Ordering::Less),
                    Ordering::Greater => Option::Some(RawFieldValue::compare_base_to_extension(extended, context)?.reverse()),
                },
            Ordering::Greater => 
                match extended.extension.compare_to_zero(context)? {
                    Ordering::Less => RawFieldValue::compare_base_to_extension(extended, context),
                    Ordering::Equal => Option::Some(Ordering::Greater),
                    Ordering::Greater => Option::Some(Ordering::Greater)
                }
        }
    }

    fn compare_base_to_extension(extended: &RawExtendedValue, context: &FieldTower) -> Option<Ordering> {
        let root_index = extended.base.num_extensions();
        let root: &RawFieldValue = &context.data.borrow().roots[root_index as usize];
        let first = extended.base.checked_mul(&extended.base, context)?;
        let second = extended.extension.checked_mul(
            &extended.extension.checked_mul(root, context)?,
            context
        )?;
        let difference = first.checked_sub(&second, context)?;
        return difference.compare_to_zero(context);
    }

    fn sqrt(&self, context: &FieldTower) -> Option<RawSqrtResult> {
        match self.compare_to_zero(context)? {
            Ordering::Less => Option::Some(RawSqrtResult::NOSOLUTION),
            Ordering::Equal => Option::Some(RawSqrtResult::SOLUTION(RawFieldValue::BASIC(Rational32::new(0, 1)))),
            Ordering::Greater => {
                match self {
                    RawFieldValue::BASIC(base) => RawFieldValue::sqrt_of_base(base),
                    RawFieldValue::EXTENDED(extended) => {
                        if (*extended.extension).eq(&RawFieldValue::zero(extended.extension.num_extensions())) {
                            RawFieldValue::sqrt_of_zero_extension(&extended.base, context)
                        } else {
                            RawFieldValue::sqrt_of_nonzero_extension(extended, context)
                        }
                    }
                }
            }
        }
    }

    fn sqrt_of_base(base: &Base) -> Option<RawSqrtResult> {
        let result: (Base, u32) = sqrt(base.clone())?;
        if result.1 == 1 {
            Option::Some(RawSqrtResult::SOLUTION(RawFieldValue::BASIC(result.0)))
        } else {
            if result.1 > (i32::MAX as u32) {
                Option::None
            } else {
                Option::Some(RawSqrtResult::EXTENSION(RawNewRootRequest {
                    coefficient: RawFieldValue::BASIC(result.0),
                    root: RawFieldValue::BASIC(Base::new(result.1 as i32, 1))
                }))
            }
        }
    }

    fn sqrt_of_zero_extension(base_of_extended: &RawFieldValue, context: &FieldTower) -> Option<RawSqrtResult> {
        match base_of_extended.sqrt(context)? {
            RawSqrtResult::NOSOLUTION => panic!("Cannot happen, we checked that the argument is positive"),
            RawSqrtResult::SOLUTION(solution) => return Some(RawSqrtResult::SOLUTION(solution)),
            RawSqrtResult::EXTENSION(preferred_extension) => {
                let existing_root_index = base_of_extended.num_extensions();
                let existing_root: &RawFieldValue = &context.data.borrow().roots[existing_root_index as usize];
                let base_of_extended_times_existing_root = base_of_extended.checked_mul(existing_root, context)?;
                match base_of_extended_times_existing_root.sqrt(context)? {
                    RawSqrtResult::NOSOLUTION => panic!("Cannot happen, we checked that base_of_extended is positive and roots are positive"),
                    RawSqrtResult::SOLUTION(solution) => {
                        // If sqrt(c) is the existing root and if base_of_extended is p, we want to express sqrt(p).
                        // We have sqrt(p) = sqrt(p)*sqrt(pc)/sqrt(pc) = p*sqrt(c)/sqrt(pc).
                        // The coefficient we need is p/sqrt(pc).
                        let existing_root_coefficient: RawFieldValue = base_of_extended.checked_div(&solution, context)?;
                        return Option::Some(RawSqrtResult::SOLUTION(RawFieldValue::EXTENDED(RawExtendedValue {
                            base: Box::new(RawFieldValue::zero(existing_root_coefficient.num_extensions())),
                            extension: Box::new(existing_root_coefficient),
                        })))},
                    RawSqrtResult::EXTENSION(_) => {
                        return Option::Some(RawSqrtResult::EXTENSION(preferred_extension))
                    },
                };
            },
        }
    }

    fn sqrt_of_nonzero_extension(extended: &RawExtendedValue, context: &FieldTower) -> Option<RawSqrtResult> {
        panic!("Not yet implemented");
    }

    fn to_string(value: &RawFieldValue, context: &FieldTower) -> String {
        match value {
            RawFieldValue::BASIC(base) => base.to_string(),
            RawFieldValue::EXTENDED(extended) => {
                let base_str = RawFieldValue::to_string(&extended.base, &context);
                let extended_coeff_str = RawFieldValue::to_string(&extended.extension, &context);
                let root_index = value.num_extensions() as usize - 1;
                let root_of: &RawFieldValue = &context.data.borrow().roots[root_index];
                let root_of_str = RawFieldValue::to_string(&root_of, &context);
                return format!("{}+{}*sqrt({})", base_str, extended_coeff_str, root_of_str);
            }
        }
    }
}

impl PartialEq for RawFieldValue {
    fn eq(&self, other: &Self) -> bool {
        let (first, second): (RawFieldValue, RawFieldValue) =
            RawFieldValue::equalize_num_extensions(&self, other);
        return RawFieldValue::compare_impl(&first, &second);
    }
}

impl Eq for RawFieldValue {}

const NUM_BITS: u32 = (usize::MAX).count_ones();

fn bin_op(
    first: &FieldValue,
    second: &FieldValue,
    handler: fn(&RawFieldValue, &RawFieldValue, &FieldTower) -> Option<RawFieldValue>)
    -> Option<FieldValue>
{
    if first.context != second.context {
        panic!("Binary operation not allowed on values from different FieldTower instances");
    }
    let result: RawFieldValue = handler(&first.value, &second.value, &first.context)?;
    return Option::Some(FieldValue {
        context: first.context.clone(),
        value: result
    });
}

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
    use std::i32;

    use crate::field::{FieldTower, FieldValue, RawFieldValue, RawExtendedValue};
    use num_rational::Rational32;
    use num_traits::{CheckedAdd, CheckedSub, CheckedMul, CheckedDiv};
    use std::cmp::Ordering;

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
    fn values_with_simple_root_can_be_subtracted() {
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
        let optional_result: Option<FieldValue> = test_value_1.checked_sub(&test_value_2);
        let result = optional_result.expect("Unexpected overflow");
        let coefficients_expected: Vec<Rational32> = vec![Rational32::new(-7, 1), Rational32::new(-15, 1)];
        let raw_expected_value = RawFieldValue::from_coefficients(&coefficients_expected[..]);
        let expected_value = FieldValue {
            context: tower,
            value: raw_expected_value
        };
        assert_eq!(result.to_string(), expected_value.to_string());
        assert_eq!(result == expected_value, true);
    }

    #[test]
    fn when_adding_produces_overflow_then_none_returned() {
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
    fn when_multiplying_produces_overflow_then_none_returned() {
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

    // Test was created based on multiplication test. Expected value of multiplication
    // is divided by one of the arguments and the other multiplication factor is
    // expected here as result.
    #[test]
    fn values_with_simple_root_can_be_divided() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational32::new(2, 1)));

        let coefficients_num: Vec<Rational32> = vec![Rational32::new(230, 1), Rational32::new(110, 1)];
        let raw_num = RawFieldValue::from_coefficients(&coefficients_num[..]);
        let num = FieldValue {
            context: tower.clone(),
            value: raw_num
        };
        let coefficients_den: Vec<Rational32> = vec![Rational32::new(3, 1), Rational32::new(5, 1)];
        let raw_den = RawFieldValue::from_coefficients(&coefficients_den[..]);
        let den = FieldValue {
            context: tower.clone(),
            value: raw_den
        };
        let coefficients_expected: Vec<Rational32> = vec![Rational32::new(10, 1), Rational32::new(20, 1)];
        let raw_expected = RawFieldValue::from_coefficients(&coefficients_expected[..]);
        let expected = FieldValue {
            context: tower.clone(),
            value: raw_expected
        };
        let optional_result: Option<FieldValue> = num.checked_div(&den);
        let result = optional_result.expect("Unexpected overflow");
        assert_eq!(result.to_string(), expected.to_string());
        assert_eq!(result == expected, true);
    }

    #[test]
    fn comparing_rationals() {
        let tower: FieldTower = FieldTower::new();
        let big: FieldValue = tower.value(Rational32::new(i32::MAX, 1));
        let small: FieldValue = tower.value(Rational32::new(i32::MIN, 1));
        let zero: FieldValue = tower.value(Rational32::new(0, 1));
        assert_eq!(small.compare_to_zero().expect("Expected Some"), Ordering::Less);
        assert_eq!(big.compare_to_zero().expect("Expected Some"), Ordering::Greater);
        assert_eq!(zero.compare_to_zero().expect("Expected Some"), Ordering::Equal);
    }

    #[test]
    fn when_coefficient_root_zero_then_sign_of_nonroot_coefficient_returned() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational32::new(2, 1)));
        let positive = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational32::new(1, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational32::new(0,1))),
            }),
            context: tower.clone(),
        };
        let negative = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational32::new(-1, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational32::new(0, 1))),
            }),
            context: tower.clone(),
        };
        let zero = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational32::new(0, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational32::new(0, 1))),
            }),
            context: tower.clone(),
        };
        assert_eq!(positive.num_extensions(), 1);
        assert_eq!(negative.num_extensions(), 1);
        assert_eq!(zero.num_extensions(), 1);
        assert_eq!(positive.compare_to_zero().expect("Expected Some"), Ordering::Greater);
        assert_eq!(negative.compare_to_zero().expect("Expected Some"), Ordering::Less);
        assert_eq!(zero.compare_to_zero().expect("Expected Some"), Ordering::Equal);
    }

    #[test]
    fn when_coefficient_nonroot_zero_then_sign_of_root_coefficient_returned() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational32::new(2, 1)));
        let positive = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational32::new(0, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational32::new(1,1))),
            }),
            context: tower.clone(),
        };
        let negative = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational32::new(0, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational32::new(-1, 1))),
            }),
            context: tower.clone(),
        };
        assert_eq!(positive.num_extensions(), 1);
        assert_eq!(negative.num_extensions(), 1);
        assert_eq!(positive.compare_to_zero().expect("Expected Some"), Ordering::Greater);
        assert_eq!(negative.compare_to_zero().expect("Expected Some"), Ordering::Less);
    }

    #[test]
    fn when_coefficients_have_same_sign_then_that_sign_returned() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational32::new(2, 1)));
        let positive = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational32::new(1, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational32::new(1,1))),
            }),
            context: tower.clone(),
        };
        let negative = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational32::new(-1, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational32::new(-1, 1))),
            }),
            context: tower.clone(),
        };
        assert_eq!(positive.num_extensions(), 1);
        assert_eq!(negative.num_extensions(), 1);
        assert_eq!(positive.compare_to_zero().expect("Expected Some"), Ordering::Greater);
        assert_eq!(negative.compare_to_zero().expect("Expected Some"), Ordering::Less);
    }

    #[test]
    fn when_nonroot_coeff_positive_then_squares_compared() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational32::new(2, 1)));
        let positive = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational32::new(2, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational32::new(-1,1))),
            }),
            context: tower.clone(),
        };
        let negative = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational32::new(2, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational32::new(-2, 1))),
            }),
            context: tower.clone(),
        };
        assert_eq!(positive.num_extensions(), 1);
        assert_eq!(negative.num_extensions(), 1);
        assert_eq!(positive.compare_to_zero().expect("Expected Some"), Ordering::Greater);
        assert_eq!(negative.compare_to_zero().expect("Expected Some"), Ordering::Less);
    }

    #[test]
    fn when_nonroot_coeff_negative_then_squares_compared() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational32::new(2, 1)));
        let positive = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational32::new(-2, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational32::new(2,1))),
            }),
            context: tower.clone(),
        };
        let negative = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational32::new(-2, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational32::new(1, 1))),
            }),
            context: tower.clone(),
        };
        assert_eq!(positive.num_extensions(), 1);
        assert_eq!(negative.num_extensions(), 1);
        assert_eq!(positive.compare_to_zero().expect("Expected Some"), Ordering::Greater);
        assert_eq!(negative.compare_to_zero().expect("Expected Some"), Ordering::Less);
    }
}