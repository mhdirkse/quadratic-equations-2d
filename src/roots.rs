// Manages field extensions. We need to extend the field of
// rational numbers three times.
//
// First, we have a 2x2 matrix
// of rationals for which we calculate the eigenvalues. This
// produces the field of numbers a + b * sqrt(c) in which
// c is not a square. We normalize such that c is an integer
// that has each prime factor only once.
//
// Second, we need the eigenvector of the first eigenvalue and
// it has to have length one. For this, we need the square root
// of a number a + b * sqrt(c).
//
// Third, we need the eignevector of the second eigenvalue and
// it has to have length one. For this, we need the square root
// of another number d + e * sqrt(c).
//
// For each of this three extensions, we should check whether
// the field is really extended or whether the new root fits
// within the existing field.
//
// A field extension is a vector space over the rationals. When
// the field of rationals is extended by sqrt(c), the basis vectors
// are one and sqrt(c). When an additional root sqrt(a + b * sqrt(c)) is
// added, the basis becomes: one, sqrt(c), sqrt(a + b * sqrt(c)) and
// sqrt(c) * sqrt(a + b * sqrt(c)). The last one can be simplified
// as sqrt(a * c + b * c + sqrt(c)). If another root sqrt(d + e * sqrt(c))
// is added, there are eight basis vectors. Each of the four previous
// basis vectors is multiplied by the new root to get the additional
// four.

use std::rc::Rc;
use std::option::Option;
use ::num_rational::Rational32;
use num_traits::CheckedAdd;

trait Field: Sized {
    fn checked_add(self, other: &Self) -> Option<Self>;
    fn checked_sub(self, other: &Self) -> Option<Self>;
    fn checked_mul(self, other: &Self) -> Option<Self>;
    fn checked_div(self, other: &Self) -> Option<Self>;
}

struct FirstExtension {
    root: u32
}

struct FirstExtensionValue {
    // First coeff is times one, second coeff is times the root.
    coeffs: [Rational32; 2],
    field: Rc<FirstExtension>
}

impl Field for FirstExtensionValue {
    fn checked_add(self, other: &Self) -> Option<Self> {
        let v0 = self.coeffs[0].checked_add(&other.coeffs[0]);
        let v1 = self.coeffs[1].checked_add(&other.coeffs[1]);
        let sum: Option<[Rational32; 2]> = match (v0, v1) {
            (None, None) => None,
            (None, Some(_)) => None,
            (Some(_), None) => None,
            (Some(v0), Some(v1)) => Some([v0, v1])
        };
        return match sum {
            Some(sum_coeffs) => Some(Self {coeffs: sum_coeffs, field: self.field.clone()}),
            None => None
        };
    }

    fn checked_sub(self, other: &Self) -> Option<Self> {
        // TODO: Implement
        return None;
    }

    fn checked_mul(self, other: &Self) -> Option<Self> {
        // Todo: Implement
        return Option::None;
    }

    fn checked_div(self, other: &Self) -> Option<Self> {
        // Todo: Implement
        return None;
    }
}

impl ToString for FirstExtensionValue {
    fn to_string(&self) -> String {
        let str_one_coeff = self.coeffs[0].to_string();
        let str_root_coeff = self.coeffs[1].to_string();
        let str_root = self.field.root.to_string();
        let mut result = String::from("");
        result.push_str(&str_one_coeff[..]);
        result.push_str(" + ");
        result.push_str(&str_root_coeff[..]);
        result.push_str(" * sqrt(");
        result.push_str(&str_root[..]);
        result.push_str(")");
        return result;
    }
}