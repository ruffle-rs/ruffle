class Foo2 extends Foo {
	function Foo2() {
		super();
		trace("Foo2:");
		trace(super);
		trace(this);
		trace("end foo2");
	}
}