use std::rc::Rc;
use num_rational::Rational32;

pub type Base = Rational32;


#[derive(Clone)]
#[derive(Debug)]
pub struct FieldTower {
    data: Rc<FieldTowerData>
}

#[derive(Debug)]
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
    base: Box<FieldValue>,
    extension: Box<FieldValue>
}

impl FieldTower {
    fn new() -> Self {
        let data: FieldTowerData = FieldTowerData {
            roots: vec![]
        };
        return FieldTower {data: Rc::new(data)};
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

#[cfg(test)]
mod test {
    use crate::field::FieldTower;

    #[test]
    fn field_tower_only_clones_are_equal() {
        let first: FieldTower = FieldTower::new();
        let second: FieldTower = FieldTower::new();
        let first_clone = first.clone();
        assert_eq!(first, first_clone);
        assert_ne!(first, second);
    }
}