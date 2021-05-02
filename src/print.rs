use ptree::TreeBuilder;
use vec1::Vec1;

use crate::interner::*;

pub(crate) trait Repr {
	fn repr(&self, i: &Interners) -> String;
}

impl Repr for StringSymbol {
	fn repr(&self, (_, s): &Interners) -> String {
		format!("{:?}", s.string(*self))
	}
}

impl<T: Repr> Repr for Option<T> {
	fn repr(&self, i: &Interners) -> String {
		match self {
			Some(x) => format!("Some({})", x.repr(i)),
			None    => "None".to_string()
		}
	}
}

impl<T: Repr> Repr for &T {
	fn repr(&self, i: &Interners) -> String {
		(*self).repr(i)
	}
}

impl<T: Repr> Repr for Vec<T> {
	fn repr(&self, i: &Interners) -> String {
		let v: Vec<String> = self.iter().map(|x| x.repr(i)).collect();
		v.join(", ")
	}
}

impl<T: Repr> Repr for Vec1<T> {
	fn repr(&self, i: &Interners) -> String {
		let v: Vec<String> = self.iter().map(|x| x.repr(i)).collect();
		v.join(", ")
	}
}

pub(crate) trait Tree {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder);
}

impl<T: Tree> Tree for &T {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		(*self).tree(i, builder);
	}
}

