use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Symbol<const CASE_SENSITIVE: bool>(usize);

#[derive(Debug, Clone, Default)]
pub struct Interner<const CASE_SENSITIVE: bool> {
	symbol_map: HashMap<Box<str>, Symbol<CASE_SENSITIVE>>,
	string_map: Vec<Box<str>>,
}

impl<const CASE_SENSITIVE: bool> std::fmt::Display for Interner<CASE_SENSITIVE> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{{\n")?;
		for (i, s) in self.string_map.iter().enumerate() {
			write!(f, "    {} => {:?},\n", i, s)?;
		}
		write!(f, "}}")?;
		Ok(())
    }
}

impl<const CASE_SENSITIVE: bool> Interner<CASE_SENSITIVE> {
	pub fn interned(&mut self, string: &str) -> Symbol<CASE_SENSITIVE> {
		if CASE_SENSITIVE {
			if let Some(s) = self.symbol_map.get(&*string) {
				return *s;
			}
		} else {
			if let Some(s) = self.symbol_map.get(&*string.to_lowercase()) {
				return *s;
			}
		}

		let new_sym = Symbol(self.string_map.len());
		if CASE_SENSITIVE {
			self.symbol_map.insert(string.to_string().into_boxed_str(), new_sym);
		} else {
			self.symbol_map.insert(string.to_lowercase().into_boxed_str(), new_sym);
		}
		self.string_map.push(string.to_string().into_boxed_str());

		new_sym
	}

	pub fn string<'s>(&'s self, symbol: Symbol<CASE_SENSITIVE>) -> &'s str {
		&self.string_map[symbol.0]
	}
}

pub type NameInterner = Interner<false>;
pub type NameSymbol   = Symbol<false>;

pub type StringInterner = Interner<true>;
pub type StringSymbol   = Symbol<true>;
