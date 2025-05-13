#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SharedType {
	Var1 {
		a: u32,
		b: bool,
	},
	Var2(String),
}
