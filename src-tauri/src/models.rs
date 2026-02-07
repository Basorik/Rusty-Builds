use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize, Debug)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct LiteNode {
    pub id: u16,
    pub x: f32,
    pub y: f32,

    pub icon: String,
}
