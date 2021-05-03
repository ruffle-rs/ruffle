class ChildA extends Super {
	function ChildA() {
		super();
		trace("ChildA constructor");
	}
	
	function work() {
		super.work();
		trace("ChildA work");
	}
}