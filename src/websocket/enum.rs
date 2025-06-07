#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BroadcastType<'a> {
    PointToPoint(&'a str, &'a str),
    PointToGroup(&'a str),
}
