use crate::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BroadcastType<T: BroadcastTypeTrait> {
    PointToPoint(T, T),
    PointToGroup(T),
}
