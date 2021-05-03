class ChildB extends Super implements Blue {
	function ChildB() {
		super();
		trace("ChildA constructor");
	}
	
	function work() {
		super.work();
		trace("ChildA work");
	}
}
