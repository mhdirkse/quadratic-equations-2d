use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Debug;
use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Div};
use std::u64;
use num_rational::Rational64;
use num_traits::{CheckedAdd, CheckedSub, CheckedMul, CheckedDiv};

pub type Base = Rational64;

const RECURSION_THRESHOLD: u32 = 10;

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

enum SqrtCandidateCheck {
    NO_MATCH,
    MATCH(RawFieldValue)
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

    // self is mutable here because we mutate the FieldTower associated with it.
    pub fn sqrt(&mut self) -> Option<FieldValue> {
        // We should work from the outermost field, otherwise we reintroduce existing roots.
        // This is why we promote before doing RawFieldValue::sqrt.
        let promoted: FieldValue = self.promote();
        let result: RawSqrtResult = promoted.value.sqrt(&self.context, 0)?;
        match result {
            RawSqrtResult::NOSOLUTION => panic!("Tried to take sqrt of negative number"),
            RawSqrtResult::SOLUTION(solution) => return Option::Some(FieldValue {
                value: solution.clone(),
                context: self.context.clone(),
            }),
            RawSqrtResult::EXTENSION(new_root_request) => {
                self.context.add_root(FieldValue { value: new_root_request.root.clone(), context: self.context.clone() });
                return Option::Some(FieldValue {
                    value: RawFieldValue::EXTENDED(RawExtendedValue {
                        base: Box::new(RawFieldValue::zero(promoted.num_extensions())),
                        extension: Box::new(new_root_request.coefficient),
                    }),
                    context: self.context.clone(),
                })
            }
        }
    }

    // For testing purposes
    fn promote(&self) -> Self {
        FieldValue {
            value: RawFieldValue::promote(&self.value, self.context.data.borrow().roots.len() as u32),
            context: self.context.clone()
        }
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
                    RawFieldValue::BASIC(other_base) => return base == other_base,
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
        } else if num_required_extensions == num_existing_extensions {
            return v.clone()
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
            return RawFieldValue::BASIC(Rational64::new(0, 1));
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

    // We assume here that self has been promoted to the outermost field.
    // We cannot promote inside this method because it is called recursively.
    fn sqrt(&self, context: &FieldTower, recursion: u32) -> Option<RawSqrtResult> {
        if recursion >= RECURSION_THRESHOLD {
            panic!("Max recursion depth reached");
        }
        match self.compare_to_zero(context)? {
            Ordering::Less => Option::Some(RawSqrtResult::NOSOLUTION),
            Ordering::Equal => Option::Some(RawSqrtResult::SOLUTION(RawFieldValue::BASIC(Rational64::new(0, 1)))),
            Ordering::Greater => {
                match self {
                    RawFieldValue::BASIC(base) => RawFieldValue::sqrt_of_base(&base, recursion),
                    RawFieldValue::EXTENDED(extended) => {
                        if (*extended.extension).eq(&RawFieldValue::zero(extended.extension.num_extensions())) {
                            RawFieldValue::sqrt_of_zero_extension(&extended.base, context, recursion)
                        } else if (*extended.base).eq(&RawFieldValue::zero(extended.base.num_extensions())) {
                            RawFieldValue::sqrt_of_zero_base(&extended.extension, context, recursion)
                        } else {
                            RawFieldValue::sqrt_of_nonzero_extension(&extended, context, recursion)
                        }
                    }
                }
            }
        }
    }

    fn sqrt_of_base(base: &Base, _: u32) -> Option<RawSqrtResult> {
        // We already checked that base > 0
        let num = base.numer().clone() as u64;
        let den = base.denom().clone() as u64;
        if let Option::Some(num_root) = RawFieldValue::sqrt_u64(num) {
            if let Option::Some(den_root) = RawFieldValue::sqrt_u64(den) {
                return Option::Some(RawSqrtResult::SOLUTION(RawFieldValue::BASIC(Base::new(
                    num_root as i64, den_root as i64))));
            }
        }
        return Option::Some(RawSqrtResult::EXTENSION(RawNewRootRequest {
            coefficient: RawFieldValue::BASIC(Base::new(1, 1)),
            root: RawFieldValue::BASIC(base.clone())
        }));
    }

    fn sqrt_u64(value: u64) -> Option<u64> {
        if value == 0 || value == 1 {
            return Option::Some(value);
        };
        let mut lower: u64 = 1;
        let mut upper: u64 = 1 << 32;
        while upper - lower >= 2 {
            let candidate: u64 = (lower + upper) / 2;
            let square: u64 = candidate * candidate;
            if square == value {
                return Option::Some(candidate);
            } else if square > value {
                upper = candidate;
            } else {
                lower = candidate;
            }
        }
        return Option::None;
    }

    fn sqrt_of_zero_extension(base_of_extended: &RawFieldValue, context: &FieldTower, recursion: u32) -> Option<RawSqrtResult> {
        match base_of_extended.sqrt(context, recursion + 1)? {
            RawSqrtResult::NOSOLUTION => panic!("Cannot happen, we checked that the argument is positive"),
            RawSqrtResult::SOLUTION(solution) => return Some(RawSqrtResult::SOLUTION(
                RawFieldValue::EXTENDED(RawExtendedValue {
                    base: Box::new(solution),
                    extension: Box::new(RawFieldValue::zero(base_of_extended.num_extensions())),
                }))),
            RawSqrtResult::EXTENSION(preferred_extension) => {
                // Assume this method was called from a value with two extensions.
                // Then we receive a value here with one extension.
                // We need the second root added to the tower which has index one.
                let existing_root_index = base_of_extended.num_extensions();
                let existing_root: &RawFieldValue = &context.data.borrow().roots[existing_root_index as usize];
                let base_of_extended_times_existing_root = base_of_extended.checked_mul(existing_root, context)?;
                match base_of_extended_times_existing_root.sqrt(context, recursion + 1)? {
                    RawSqrtResult::NOSOLUTION => panic!("Cannot happen, we checked that base_of_extended is positive and roots are positive"),
                    RawSqrtResult::SOLUTION(solution) => {
                        // If sqrt(c) is the existing root and if base_of_extended is p, we want to express sqrt(p).
                        // We have sqrt(p) = sqrt(p)*sqrt(pc)/sqrt(pc) = p*sqrt(c)/sqrt(pc).
                        // The coefficient we need is p/sqrt(pc).
                        let existing_root_coefficient: RawFieldValue = base_of_extended.checked_div(&solution, context)?;
                        return Option::Some(RawSqrtResult::SOLUTION(
                            RawFieldValue::EXTENDED(RawExtendedValue {
                                base: Box::new(RawFieldValue::zero(base_of_extended.num_extensions())),
                                extension: Box::new(existing_root_coefficient),
                            })
                        ));
                    },
                    RawSqrtResult::EXTENSION(_) => {
                        // The coefficient of the RawNewRootRequest preferred_extension
                        // comes from a recursive call of sqrt() that takes into account
                        // less extensions. We should promote the coefficient to the
                        // number of extensions the field has at this recursion level.
                        let result = RawNewRootRequest {
                            coefficient: RawFieldValue::promote(&preferred_extension.coefficient, base_of_extended.num_extensions() + 1),
                            root: preferred_extension.root
                        };
                        return Option::Some(RawSqrtResult::EXTENSION(result))
                    },
                };
            },
        }
    }

    fn sqrt_of_zero_base(extension_of_extended: &RawFieldValue, context: &FieldTower, recursion: u32) -> Option<RawSqrtResult> {
        let root_index = extension_of_extended.num_extensions() as usize;
        let root: &RawFieldValue = &context.data.borrow().roots[root_index];
        let sqrt_of_extension_of_extended: RawSqrtResult =
            RawFieldValue::promote(extension_of_extended, root_index as u32 + 1)
                .sqrt(context, recursion + 1)?;
        match sqrt_of_extension_of_extended {
            RawSqrtResult::NOSOLUTION => panic!("Cannot happen, we checked already that extension_of_extended is positive"),
            RawSqrtResult::SOLUTION(solution) => Option::Some(RawSqrtResult::EXTENSION(RawNewRootRequest {
                coefficient: solution,
                root: RawFieldValue::EXTENDED(RawExtendedValue {
                    base: Box::new(RawFieldValue::zero(root_index as u32)),
                    extension: Box::new(RawFieldValue::promote(&RawFieldValue::BASIC(Base::new(1, 1)), root_index as u32))
            })})),
            RawSqrtResult::EXTENSION(_) => {
                let coefficient: RawFieldValue = RawFieldValue::promote(
                    &RawFieldValue::BASIC(Base::new(1, 1)), 
                    root_index as u32 + 1
                );
                let root = RawFieldValue::EXTENDED(RawExtendedValue {
                    base: Box::new(RawFieldValue::zero(root_index as u32)),
                    extension: Box::new(extension_of_extended.clone()), 
                });
                Option::Some(RawSqrtResult::EXTENSION(RawNewRootRequest { coefficient, root }))
            }
        }
    }

    fn sqrt_of_nonzero_extension(extended: &RawExtendedValue, context: &FieldTower, recursion: u32) -> Option<RawSqrtResult> {
        let original: RawFieldValue = RawFieldValue::EXTENDED(RawExtendedValue {
            base: extended.base.clone(),
            extension: extended.extension.clone()
        });
        let root_index: u32 = extended.base.num_extensions();
        let root: RawFieldValue = RawFieldValue::promote(&context.data.borrow().roots[root_index as usize], root_index);
        let two: RawFieldValue = RawFieldValue::promote(&RawFieldValue::BASIC(Base::new(2, 1)), root_index);
        let term1: RawFieldValue = extended.base.checked_div(&root.checked_mul(&two, &context)?, &context)?;
        let discriminant: RawFieldValue = RawFieldValue::pseudo_norm(&extended, &root, &context)?;
        // discriminant has the same number of extensions as extended.base and extended.extension.
        // We ignore the latest root on purpose.
        let optional_sqrt_discriminant: RawSqrtResult = RawFieldValue::sqrt(&discriminant, context, recursion + 1)?;
        if let RawSqrtResult::SOLUTION(root_discriminant) = optional_sqrt_discriminant {
            let term2: RawFieldValue = root_discriminant.checked_div(&root.checked_mul(&two, &context)?, &context)?;
            let candidate_1: RawFieldValue = term1.checked_sub(&term2, &context)?;
            let match_1: SqrtCandidateCheck = RawFieldValue::check_sqrt_candidate(&candidate_1, extended, context, recursion + 1)?;
            if let SqrtCandidateCheck::MATCH(pattern_match_1) = match_1 {
                return Option::Some(RawSqrtResult::SOLUTION(pattern_match_1))
            } else {
                let candidate_2: RawFieldValue = term1.checked_add(&term2, &context)?;
                let match_2: SqrtCandidateCheck = RawFieldValue::check_sqrt_candidate(&candidate_2, extended, context, recursion + 1)?;
                if let SqrtCandidateCheck::MATCH(pattern_match_2) = match_2 {
                    return Option::Some(RawSqrtResult::SOLUTION(pattern_match_2));
                } else {
                    return RawFieldValue::raw_field_value_as_root(&original);
                }
            }
        } else {
            return RawFieldValue::raw_field_value_as_root(&original);
        }
    }

    // sqr_candidate has one extension less than original
    fn check_sqrt_candidate(sqr_candidate: &RawFieldValue, original: &RawExtendedValue, context: &FieldTower, recursion: u32) -> Option<SqrtCandidateCheck> {
        let extension_candidate_option: RawSqrtResult = sqr_candidate.sqrt(context, recursion)?;
        if let RawSqrtResult::SOLUTION(extension_candidate) = extension_candidate_option {
            let two: RawFieldValue = RawFieldValue::promote(&RawFieldValue::BASIC(Base::new(2, 1)), sqr_candidate.num_extensions());
            let base_candidate: RawFieldValue = original.extension.checked_div(&extension_candidate.checked_mul(&two, context)?, context)?;
            let mut candidate = RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(base_candidate),
                extension: Box::new(extension_candidate)
            });
            if let Ordering::Less = candidate.compare_to_zero(context)? {
                candidate = RawFieldValue::BASIC(Base::new(0, 1)).checked_sub(&candidate, context)?;
            }
            let sqr_candidate: RawFieldValue = candidate.checked_mul(&candidate, context)?;
            if sqr_candidate == RawFieldValue::EXTENDED(RawExtendedValue { base: original.base.clone(), extension: original.extension.clone() }) {
                return Option::Some(SqrtCandidateCheck::MATCH(candidate));
            } else {
                return Option::Some(SqrtCandidateCheck::NO_MATCH);
            }
        } else {
            return Option::Some(SqrtCandidateCheck::NO_MATCH);
        }
    }

    fn raw_field_value_as_root(v: &RawFieldValue) -> Option<RawSqrtResult> {
        let num_extensions: u32 = v.num_extensions();
        Option::Some(RawSqrtResult::EXTENSION(RawNewRootRequest {
            coefficient: RawFieldValue::promote(&RawFieldValue::BASIC(Base::new(1, 1)), num_extensions),
            root: v.clone(),
        }))
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
                return format!("({}+{}*sqrt({}))", base_str, extended_coeff_str, root_of_str);
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
    use crate::field::{FieldTower, FieldValue, RawFieldValue, RawExtendedValue};
    use num_rational::Rational64;
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
        let basic_value = tower.value(Rational64::new(5, 2));
        assert_eq!(basic_value.num_extensions(), 0);
    }

    #[test]
    fn value_with_simple_root_has_num_extensions_one_and_can_be_formatted() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational64::new(2, 1)));
        let coefficients: Vec<Rational64> = vec![Rational64::new(3, 1), Rational64::new(5, 1)];
        let raw_test_value = RawFieldValue::from_coefficients(&coefficients[..]);
        let test_value: FieldValue = FieldValue {
            context: tower,
            value: raw_test_value
        };
        assert_eq!("(3+5*sqrt(2))", test_value.to_string());
    }

    #[test]
    fn values_with_simple_root_can_be_added() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational64::new(2, 1)));
        let coefficients_1: Vec<Rational64> = vec![Rational64::new(3, 1), Rational64::new(5, 1)];
        let raw_test_value_1 = RawFieldValue::from_coefficients(&coefficients_1[..]);
        let test_value_1 = FieldValue {
            context: tower.clone(),
            value: raw_test_value_1
        };
        let coefficients_2: Vec<Rational64> = vec![Rational64::new(10, 1), Rational64::new(20, 1)];
        let raw_test_value_2 = RawFieldValue::from_coefficients(&coefficients_2[..]);
        let test_value_2 = FieldValue {
            context: tower.clone(),
            value: raw_test_value_2
        };
        let optional_result: Option<FieldValue> = test_value_1.checked_add(&test_value_2);
        let result = optional_result.expect("Unexpected overflow");
        let coefficients_expected: Vec<Rational64> = vec![Rational64::new(13, 1), Rational64::new(25, 1)];
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
        tower.add_root(tower.value(Rational64::new(2, 1)));
        let coefficients_1: Vec<Rational64> = vec![Rational64::new(3, 1), Rational64::new(5, 1)];
        let raw_test_value_1 = RawFieldValue::from_coefficients(&coefficients_1[..]);
        let test_value_1 = FieldValue {
            context: tower.clone(),
            value: raw_test_value_1
        };
        let coefficients_2: Vec<Rational64> = vec![Rational64::new(10, 1), Rational64::new(20, 1)];
        let raw_test_value_2 = RawFieldValue::from_coefficients(&coefficients_2[..]);
        let test_value_2 = FieldValue {
            context: tower.clone(),
            value: raw_test_value_2
        };
        let optional_result: Option<FieldValue> = test_value_1.checked_sub(&test_value_2);
        let result = optional_result.expect("Unexpected overflow");
        let coefficients_expected: Vec<Rational64> = vec![Rational64::new(-7, 1), Rational64::new(-15, 1)];
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
        tower.add_root(tower.value(Rational64::new(2, 1)));
        let big_base = FieldValue {
            context: tower.clone(),
            value: RawFieldValue::from_coefficients(&vec![Rational64::new(i64::MAX, 1), Rational64::new(1, 1)])
        };
        let big_extended = FieldValue {
            context: tower.clone(),
            value: RawFieldValue::from_coefficients(&vec![Rational64::new(1, 1), Rational64::new(i64::MAX, 1)])
        };
        let with_coefficients_one = FieldValue {
            context: tower.clone(),
            value: RawFieldValue::from_coefficients(&vec![Rational64::new(1, 1), Rational64::new(1, 1)])
        };
        assert_eq!(big_base.checked_add(&with_coefficients_one).is_none(), true);
        assert_eq!(with_coefficients_one.checked_add(&big_base).is_none(), true);
        assert_eq!(big_extended.checked_add(&with_coefficients_one).is_none(), true);
        assert_eq!(with_coefficients_one.checked_add(&big_extended).is_none(), true);
    }

    #[test]
    fn when_values_have_unequal_number_of_extensions_then_can_be_added() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational64::new(2, 1)));
        let extended: FieldValue = FieldValue {
            context: tower.clone(),
            value: RawFieldValue::from_coefficients(&vec![Rational64::new(3, 1), Rational64::new(5, 1)])
        };
        let simple: FieldValue = tower.value(Rational64::new(6, 1));
        let result_1: FieldValue = extended.checked_add(&simple).expect("No overflow expected");
        let result_2: FieldValue = simple.checked_add(&extended).expect("No overflow expected");
        let expected_str = "(9+5*sqrt(2))";
        assert_eq!(result_1.to_string(), expected_str);
        assert_eq!(result_2.to_string(), expected_str);
    }

    #[test]
    fn values_with_simple_root_can_be_multiplied() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational64::new(2, 1)));
        let coefficients_1: Vec<Rational64> = vec![Rational64::new(3, 1), Rational64::new(5, 1)];
        let raw_test_value_1 = RawFieldValue::from_coefficients(&coefficients_1[..]);
        let test_value_1 = FieldValue {
            context: tower.clone(),
            value: raw_test_value_1
        };
        let coefficients_2: Vec<Rational64> = vec![Rational64::new(10, 1), Rational64::new(20, 1)];
        let raw_test_value_2 = RawFieldValue::from_coefficients(&coefficients_2[..]);
        let test_value_2 = FieldValue {
            context: tower.clone(),
            value: raw_test_value_2
        };
        let optional_result: Option<FieldValue> = test_value_1.checked_mul(&test_value_2);
        let result = optional_result.expect("Unexpected overflow");
        let coefficients_expected: Vec<Rational64> = vec![Rational64::new(230, 1), Rational64::new(110, 1)];
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
        tower.add_root(tower.value(Rational64::new(2, 1)));
        let big_base = FieldValue {
            context: tower.clone(),
            value: RawFieldValue::from_coefficients(&vec![Rational64::new(i64::MAX, 1), Rational64::new(1, 1)])
        };
        let big_extended = FieldValue {
            context: tower.clone(),
            value: RawFieldValue::from_coefficients(&vec![Rational64::new(1, 1), Rational64::new(i64::MAX, 1)])
        };
        let multiplier = FieldValue {
            context: tower.clone(),
            value: RawFieldValue::from_coefficients(&vec![Rational64::new(2, 1), Rational64::new(2, 1)])
        };
        assert_eq!(big_base.checked_add(&multiplier).is_none(), true);
        assert_eq!(multiplier.checked_add(&big_base).is_none(), true);
        assert_eq!(big_extended.checked_add(&multiplier).is_none(), true);
        assert_eq!(multiplier.checked_add(&big_extended).is_none(), true);
    }

    #[test]
    fn when_values_have_unequal_number_of_extensions_then_can_be_multiplied() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational64::new(2, 1)));
        let extended: FieldValue = FieldValue {
            context: tower.clone(),
            value: RawFieldValue::from_coefficients(&vec![Rational64::new(3, 1), Rational64::new(5, 1)])
        };
        let simple: FieldValue = tower.value(Rational64::new(6, 1));
        let result_1: FieldValue = extended.checked_mul(&simple).expect("No overflow expected");
        let result_2: FieldValue = simple.checked_mul(&extended).expect("No overflow expected");
        let expected_str = "(18+30*sqrt(2))";
        assert_eq!(result_1.to_string(), expected_str);
        assert_eq!(result_2.to_string(), expected_str);
    }

    // Test was created based on multiplication test. Expected value of multiplication
    // is divided by one of the arguments and the other multiplication factor is
    // expected here as result.
    #[test]
    fn values_with_simple_root_can_be_divided() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational64::new(2, 1)));

        let coefficients_num: Vec<Rational64> = vec![Rational64::new(230, 1), Rational64::new(110, 1)];
        let raw_num = RawFieldValue::from_coefficients(&coefficients_num[..]);
        let num = FieldValue {
            context: tower.clone(),
            value: raw_num
        };
        let coefficients_den: Vec<Rational64> = vec![Rational64::new(3, 1), Rational64::new(5, 1)];
        let raw_den = RawFieldValue::from_coefficients(&coefficients_den[..]);
        let den = FieldValue {
            context: tower.clone(),
            value: raw_den
        };
        let coefficients_expected: Vec<Rational64> = vec![Rational64::new(10, 1), Rational64::new(20, 1)];
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
        let big: FieldValue = tower.value(Rational64::new(i64::MAX, 1));
        let small: FieldValue = tower.value(Rational64::new(i64::MIN, 1));
        let zero: FieldValue = tower.value(Rational64::new(0, 1));
        assert_eq!(small.compare_to_zero().expect("Expected Some"), Ordering::Less);
        assert_eq!(big.compare_to_zero().expect("Expected Some"), Ordering::Greater);
        assert_eq!(zero.compare_to_zero().expect("Expected Some"), Ordering::Equal);
    }

    #[test]
    fn sqrt_u64() {
        assert_eq!(RawFieldValue::sqrt_u64(0).expect("sqrt(0) should exist"), 0);
        assert_eq!(RawFieldValue::sqrt_u64(1).expect("sqrt(1) should exist"), 1);
        assert_eq!(RawFieldValue::sqrt_u64(4).expect("sqrt(4) should exist"), 2);
        let root: u64 = u32::MAX as u64 - 1;
        assert_eq!(RawFieldValue::sqrt_u64(root * root).expect("Maximum root should exist"), root);
        assert_eq!(RawFieldValue::sqrt_u64(3), Option::None);
        assert_eq!(RawFieldValue::sqrt_u64(root * root + 1), Option::None);
    }

    #[test]
    fn sqrt_rational() {
        let tower: FieldTower = FieldTower::new();
        let mut square: FieldValue = tower.value(Rational64::new(18, 8));
        let root: FieldValue = square.sqrt().expect("sqrt(18/8) should not be out of bounds");
        assert_eq!(tower.data.borrow().roots.len(), 0);
        assert_eq!(root.value, RawFieldValue::BASIC(Rational64::new(3, 2)));
    }

    #[test]
    fn when_coefficient_root_zero_then_sign_of_nonroot_coefficient_returned() {
        let mut tower: FieldTower = FieldTower::new();
        tower.add_root(tower.value(Rational64::new(2, 1)));
        let positive = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational64::new(1, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational64::new(0,1))),
            }),
            context: tower.clone(),
        };
        let negative = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational64::new(-1, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational64::new(0, 1))),
            }),
            context: tower.clone(),
        };
        let zero = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational64::new(0, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational64::new(0, 1))),
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
        tower.add_root(tower.value(Rational64::new(2, 1)));
        let positive = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational64::new(0, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational64::new(1,1))),
            }),
            context: tower.clone(),
        };
        let negative = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational64::new(0, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational64::new(-1, 1))),
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
        tower.add_root(tower.value(Rational64::new(2, 1)));
        let positive = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational64::new(1, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational64::new(1,1))),
            }),
            context: tower.clone(),
        };
        let negative = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational64::new(-1, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational64::new(-1, 1))),
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
        tower.add_root(tower.value(Rational64::new(2, 1)));
        let positive = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational64::new(2, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational64::new(-1,1))),
            }),
            context: tower.clone(),
        };
        let negative = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational64::new(2, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational64::new(-2, 1))),
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
        tower.add_root(tower.value(Rational64::new(2, 1)));
        let positive = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational64::new(-2, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational64::new(2,1))),
            }),
            context: tower.clone(),
        };
        let negative = FieldValue {
            value: RawFieldValue::EXTENDED(RawExtendedValue {
                base: Box::new(RawFieldValue::BASIC(Rational64::new(-2, 1))),
                extension: Box::new(RawFieldValue::BASIC(Rational64::new(1, 1))),
            }),
            context: tower.clone(),
        };
        assert_eq!(positive.num_extensions(), 1);
        assert_eq!(negative.num_extensions(), 1);
        assert_eq!(positive.compare_to_zero().expect("Expected Some"), Ordering::Greater);
        assert_eq!(negative.compare_to_zero().expect("Expected Some"), Ordering::Less);
    }

    #[test]
    fn when_we_take_sqrt_of_rational_square_we_get_plain_rational() {
        let tower: FieldTower = FieldTower::new();
        let mut value: FieldValue = tower.value(Rational64::new(4, 1));
        let sqrt_value: FieldValue = value.sqrt().expect("square root of 4 exists");
        assert_eq!(tower.data.borrow().roots.len(), 0);
        assert_eq!(sqrt_value.num_extensions(), 0);
        assert_eq!(sqrt_value.to_string(), "2");
    }

    #[test]
    fn when_we_have_one_extension_we_do_not_unnecessarily_extend() {
        let tower: FieldTower = FieldTower::new();
        let mut two: FieldValue = tower.value(Rational64::new(2, 1));
        let sqrt_of_two: FieldValue = two.sqrt().expect("sqrt(2) should exist");
        assert_eq!(tower.data.borrow().roots.len(), 1);
        assert_eq!(sqrt_of_two.num_extensions(), 1);
        assert_eq!(sqrt_of_two.to_string(), "(0+1*sqrt(2))");
        let mut promoted_nine: FieldValue = tower.value(Rational64::new(9, 1)).promote();
        let sqrt_of_nine: FieldValue = promoted_nine.sqrt().expect("sqrt(9) should exist");        
        assert_eq!(sqrt_of_nine.num_extensions(), 1);
        assert_eq!(sqrt_of_nine.to_string(), "(3+0*sqrt(2))");
        let mut eighteen = tower.value(Rational64::new(18, 1));
        let sqrt_of_eighteen = eighteen.sqrt().expect("sqrt(18) should exist");
        assert_eq!(sqrt_of_eighteen.num_extensions(), 1);
        assert_eq!(sqrt_of_eighteen.to_string(), "(0+3*sqrt(2))");
    }

    #[test]
    fn when_we_have_two_extensions_we_do_not_unnecessarily_extend_roots_of_plain_rationals() {
        let tower: FieldTower = FieldTower::new();
        let mut two: FieldValue = tower.value(Rational64::new(2, 1));
        two.sqrt().expect("sqrt(2) should exist");
        let mut three: FieldValue = tower.value(Rational64::new(3, 1));
        three.sqrt().expect("sqrt(3) should exist");
        let mut promoted_four = tower.value(Rational64::new(4, 1)).promote();
        assert_eq!(promoted_four.num_extensions(), 2);
        let sqrt_of_four = promoted_four.sqrt().expect("sqrt(4) should exist");
        assert_eq!(sqrt_of_four.num_extensions(), 2);
        if let RawFieldValue::EXTENDED(extended) = &sqrt_of_four.value {
            assert_eq!(extended.base.num_extensions(), 1);
            assert_eq!(extended.extension.num_extensions(), 1);
        } else {
            panic!("sqrt_of_four is expected to enum RawFieldValue::EXTENDED");
        }
        assert_eq!(sqrt_of_four.to_string(), "((2+0*sqrt(2))+(0+0*sqrt(2))*sqrt(3))");
        let sqrt_eight: FieldValue = tower.value(Rational64::new(8, 1)).sqrt().expect("sqrt(8) should exist");
        // Check that sqrt(2) is not added again to the FieldTower.
        assert_eq!(tower.data.borrow().roots.len(), 2);
        assert_eq!(sqrt_eight.to_string(), "((0+2*sqrt(2))+(0+0*sqrt(2))*sqrt(3))");
        let sqrt_twelve = tower.value(Rational64::new(12, 1)).sqrt().expect("sqrt(12) should exist");
        assert_eq!(sqrt_twelve.to_string(), "((0+0*sqrt(2))+(2+0*sqrt(2))*sqrt(3))");
        let sqrt_six = tower.value(Rational64::new(6, 1)).sqrt().expect("sqrt(6) should exist");
        assert_eq!(sqrt_six.to_string(), "((0+0*sqrt(2))+(0+1*sqrt(2))*sqrt(3))");
    }

    #[test]
    fn when_root_taken_with_square_extension_coefficient_then_extension_not_within_new_root() {
        let tower: FieldTower = FieldTower::new();
        let mut two: FieldValue = tower.value(Rational64::new(2, 1));
        let sqrt_of_two: FieldValue = two.sqrt().expect("sqrt(2) should exist");
        let mut four_times_sqrt_two: FieldValue = tower
            .value(Rational64::new(4, 1))
            .checked_mul(&sqrt_of_two)
            .expect("four times sqrt(2) should not produce overflow");
        let sqrt_of_it = four_times_sqrt_two.sqrt().expect("Should be able to calculate sqrt(4 * sqrt(2))");
        assert_eq!(sqrt_of_it.to_string(), "((0+0*sqrt(2))+(2+0*sqrt(2))*sqrt((0+1*sqrt(2))))");
        // In the above the parenthesis are difficult to check. Test it another way.
        check_integrity(&sqrt_of_it.value, 2);
        assert_eq!(sqrt_of_it.context.data.borrow().roots.len(), 2);
        let the_root: RawFieldValue = sqrt_of_it.context.data.borrow().roots.last().expect("There should be roots").clone();
        check_integrity(&the_root, 1);
    }

    #[test]
    fn when_root_of_rational_added_to_already_extended_field_then_is_extension_of_extension() {
        let tower: FieldTower = FieldTower::new();
        let mut five: FieldValue = tower.value(Rational64::new(5, 1));
        let sqrt_five: FieldValue = five.sqrt().expect("sqrt(5) should exist");
        let mut six: FieldValue = tower.value(Rational64::new(6, 1));
        let sqrt_six: FieldValue = six.sqrt().expect("sqrt(6) should exist");
        assert_eq!("((0+0*sqrt(5))+(1+0*sqrt(5))*sqrt(6))", sqrt_six.to_string());
        check_integrity(&sqrt_six.value, 2);
    }

    #[test]
    fn when_root_added_from_outer_field_then_coefficient_has_right_number_of_extensions() {
        let tower: FieldTower = FieldTower::new();
        let mut five: FieldValue = tower.value(Rational64::new(5, 1));
        let sqrt_five: FieldValue = five.sqrt().expect("sqrt(5) should exist");
        let one: FieldValue = tower.value(Rational64::new(1, 1));
        let mut one_plus_sqrt_five: FieldValue = one.checked_add(&sqrt_five).expect("1+sqrt(5) should exist");
        let new_sqrt: FieldValue = one_plus_sqrt_five.sqrt().expect("sqrt(1+sqrt(5)) should exist");
        check_integrity(&new_sqrt.value, 2);
        assert_eq!("((0+0*sqrt(5))+(1+0*sqrt(5))*sqrt((1+1*sqrt(5))))", new_sqrt.to_string());
    }

    #[test]
    fn when_root_taken_with_coefficient_matching_existing_root_then_extension_not_within_new_root() {
        let tower: FieldTower = FieldTower::new();
        let mut two: FieldValue = tower.value(Rational64::new(2, 1));
        let sqrt_of_two: FieldValue = two.sqrt().expect("sqrt(2) should exist");
        let mut two_times_sqrt_two: FieldValue = tower
            .value(Rational64::new(2, 1))
            .checked_mul(&sqrt_of_two)
            .expect("two times sqrt(2) should not produce overflow");
        let sqrt_of_it: FieldValue = two_times_sqrt_two.sqrt().expect("sqrt(2*sqrt(2)) should exist");
        assert_eq!(sqrt_of_it.to_string(), "((0+0*sqrt(2))+(0+1*sqrt(2))*sqrt((0+1*sqrt(2))))");
        check_integrity(&sqrt_of_it.value, 2);
        assert_eq!(sqrt_of_it.context.data.borrow().roots.len(), 2);
        let the_root: RawFieldValue = sqrt_of_it.context.data.borrow().roots.last().expect("There should be roots").clone();
        check_integrity(&the_root, 1);
    }

    #[test]
    fn when_root_taken_with_coefficient_not_matching_existing_root_then_extension_within_new_root() {
        let tower: FieldTower = FieldTower::new();
        let mut two: FieldValue = tower.value(Rational64::new(2, 1));
        let sqrt_of_two: FieldValue = two.sqrt().expect("sqrt(2) should exist");
        let mut three_times_sqrt_two: FieldValue = tower
            .value(Rational64::new(3, 1))
            .checked_mul(&sqrt_of_two)
            .expect("three times sqrt(2) should not produce overflow");
        let sqrt_of_it = three_times_sqrt_two.sqrt().expect("sqrt(3*sqrt(2)) should exist");
        assert_eq!(sqrt_of_it.to_string(), "((0+0*sqrt(2))+(1+0*sqrt(2))*sqrt((0+3*sqrt(2))))");
        check_integrity(&sqrt_of_it.value, 2);
        assert_eq!(sqrt_of_it.context.data.borrow().roots.len(), 2);
        let the_root: RawFieldValue = sqrt_of_it.context.data.borrow().roots.last().expect("second root should exist").clone();
        check_integrity(&the_root, 1);
    }

    #[test]
    fn when_complex_root_exists_then_found() {
        // (sqrt(2) - 1)^2 = 3 - 2*sqrt(2);
        let tower: FieldTower = FieldTower::new();
        let mut two: FieldValue = tower.value(Rational64::new(2, 1));
        let sqrt_of_two: FieldValue = two.sqrt().expect("sqrt(2) should exist");
        let mut original: FieldValue = tower.value(Rational64::new(3, 1)).checked_sub(
            &two.checked_mul(&sqrt_of_two).expect("Term should exist"),
        ).expect("Value should exist");
        let result: FieldValue = original.sqrt().expect("sqrt(3 - 2*sqrt(2)) should exist");
        assert_eq!(tower.data.borrow().roots.len(), 1);
        assert_eq!(result.to_string(), "(-1+1*sqrt(2))");
    }

    #[test]
    fn when_complex_root_with_negative_root_coefficient_exists_then_positive_root_returned() {
        // (3 - 2*sqrt(2))^2 = 17 - 12*sqrt(2)
        let tower: FieldTower = FieldTower::new();
        let mut two: FieldValue = tower.value(Rational64::new(2, 1));
        let sqrt_of_two: FieldValue = two.sqrt().expect("sqrt(2) should exist");
        let seventeen = tower.value(Rational64::new(17, 1));
        let twelve = tower.value(Rational64::new(12, 1));
        let mut original: FieldValue = seventeen.checked_sub(
            &twelve.checked_mul(&sqrt_of_two).expect("12*sqrt(2) should exist")
        ).expect("17 - 12*sqrt(2) should exist");
        let result = original.sqrt().expect("sqrt(17 - 12*sqrt(2)) should exist");
        assert_eq!(tower.data.borrow().roots.len(), 1);
        assert_eq!(result.to_string(), "(3+-2*sqrt(2))");
    }

    #[test]
    fn when_rationals_are_extended_two_times_then_can_solve_sqrt() {
        // (4 + 3*sqrt(5) + 7*sqrt(6))^2 =
        //     16 + 24*sqrt(5) + 56*sqrt(6) + 45 + 42*sqrt(5)*sqrt(6) + 294 =
        //     355 + 24*sqrt(5) + (56 + 42*sqrt(5))*sqrt(6)
        let tower: FieldTower = FieldTower::new();
        let mut five: FieldValue = tower.value(Rational64::new(5, 1));
        let sqrt_five: FieldValue = five.sqrt().expect("sqrt(5) should exist");
        let mut six: FieldValue = tower.value(Rational64::new(6, 1));
        let sqrt_six: FieldValue = six.sqrt().expect("sqrt(6) should exist");
        println!("sqrt(6) shows as [{}]", sqrt_six.to_string());
        check_integrity(&sqrt_six.value, 2);
        assert_eq!(tower.data.borrow().roots.len(), 2);
        let term1: FieldValue = tower.value(Rational64::new(355, 1));
        let term2: FieldValue = tower.value(Rational64::new(24, 1))
            .checked_mul(&sqrt_five).expect("24*sqrt(5) should exist");
        assert_eq!("(0+24*sqrt(5))", term2.to_string());
        let term31: FieldValue = tower.value(Rational64::new(56, 1));
        let term32: FieldValue = tower.value(Rational64::new(42, 1))
            .checked_mul(&sqrt_five).expect("42*sqrt(5) should exist");
        let factor3 = term31.checked_add(&term32).expect("56 + 42*sqrt(5) should exist");
        check_integrity(&factor3.value, 1);
        let term3: FieldValue = factor3.checked_mul(&sqrt_six).expect("(56 + 42*sqrt(5))*sqrt(6) should exist");
        check_integrity(&term3.value, 2);
        let summed_1: FieldValue = term1.checked_add(&term2).expect("Sum of first two terms should exist");
        let mut square: FieldValue = summed_1.checked_add(&term3).expect("Value to try sqrt with should exist");
        check_integrity(&square.value, 2);
        let actual: FieldValue = square.sqrt().expect("Should be able to take the sqrt");
        check_integrity(&actual.value, 2);
        assert_eq!("((4+3*sqrt(5))+(7+0*sqrt(5))*sqrt(6))", actual.to_string());
    }

    #[test]
    fn when_multiple_field_extensions_then_root_of_squared_value_can_be_found() {
        // (7 + 3*sqrt(2) + 5*sqrt(3) + sqrt(1+sqrt(2)))^2
        //   = 49 + 42*sqrt(2) + 70*sqrt(3) + 14*sqrt(1+sqrt(2)) + 18 + 30*sqrt(6) + 6*sqrt(2+sqrt(2)) + 75 + 10*sqrt(3+sqrt(6)) + 1 + sqrt(2)
        //   = (49 + 18 + 75 + 1) + (42 + 1)*sqrt(2) + (70 + 30*sqrt(2))*sqrt(3) + (14 + 6*sqrt(2) + 10*sqrt(3))*sqrt(1+sqrt(2))
        //   = 143 + 43*sqrt(2) + (70 + 30*sqrt(2))*sqrt(3) + (14 + 6*sqrt(2) + 10*sqrt(3))*sqrt(1+sqrt(2))
        let tower = FieldTower::new();
        let mut two: FieldValue = tower.value(Rational64::new(2, 1));
        let sqrt_two: FieldValue = two.sqrt().expect("sqrt(2) should exist");
        let mut three: FieldValue = tower.value(Rational64::new(3, 1));
        let sqrt_three: FieldValue = three.sqrt().expect("sqrt(3) should exist");
        let one: FieldValue = tower.value(Rational64::new(1, 1));
        let mut one_plus_sqrt_two: FieldValue = one.checked_add(&sqrt_two).expect("1 + sqrt(2) should exist");
        let sqrt_one_plus_sqrt_two: FieldValue = one_plus_sqrt_two.sqrt().expect("sqrt(1+sqrt(2)) should exist");
        let term1: FieldValue = tower.value(Rational64::new(143, 1));
        let term2: FieldValue = tower.value(Rational64::new(43, 1)).checked_mul(&sqrt_two).expect("43*sqrt(2) should exist");
        let term32: FieldValue = tower.value(Rational64::new(30, 1)).checked_mul(&sqrt_two).expect("30*sqrt(2) should exist");
        let factor3: FieldValue = tower.value(Rational64::new(70, 1)).checked_add(&term32).expect("70 + 30*sqrt(2) should exist");
        let term3: FieldValue = factor3.checked_mul(&sqrt_three).expect("(70 + 30*sqrt(2))*sqrt(3) should exist");
        let term42: FieldValue = tower.value(Rational64::new(6, 1)).checked_mul(&sqrt_two).expect("6*sqrt(2) should exist");
        let term43: FieldValue = tower.value(Rational64::new(10, 1)).checked_mul(&sqrt_three).expect("10*sqrt(3) should exist");
        let factor42: FieldValue = term42.checked_add(&term3).expect("6*sqrt(2) + 10*sqrt(3) should exist");
        let factor4: FieldValue = tower.value(Rational64::new(14, 1)).checked_add(&factor42).expect("14 + 6*sqrt(2) + 10*sqrt(3) should exist");
        let term4: FieldValue = factor4.checked_mul(&sqrt_one_plus_sqrt_two).expect("(14 + 6*sqrt(2) + 10*sqrt(3))*sqrt(1+sqrt(2)) should exist");
        let result1: FieldValue = term1.checked_add(&term2).expect("term1 + term2 should exist");
        let result2: FieldValue = result1.checked_add(&term3).expect("term1 + term2 + term3 should exist");
        let mut square = result2.checked_add(&term4).expect("Value to take root of should exist");
        assert_eq!(tower.data.borrow().roots.len(), 3);
        let actual: FieldValue = square.sqrt().expect("Should be able to take the sqrt");
        assert_eq!(tower.data.borrow().roots.len(), 3);
        check_integrity(&actual.value, 3);
        assert_eq!(actual.to_string(), "(((7+3*sqrt(2))+(5+0*sqrt(2))*sqrt(3))+((1+0*sqrt(2))+(0+0*sqrt(2))*sqrt(3))*sqrt(1+sqrt(2)))");
    }

    fn check_integrity(v: &RawFieldValue, depth: u32) {
        if depth == 0 {
            if let RawFieldValue::EXTENDED(_) = *v {
                panic!("check_integrity(): Expected RawFieldValue::BASIC");
            }
        } else {
            match v {
                RawFieldValue::BASIC(_) => panic!("check_integrity(): Expected RawFieldValue::EXTENDED"),
                RawFieldValue::EXTENDED(extended) => {
                    check_integrity(&extended.base, depth - 1);
                    check_integrity(&extended.extension, depth - 1);
                }
            }
        }
    }
}