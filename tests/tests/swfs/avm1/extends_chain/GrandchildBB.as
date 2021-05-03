class GrandchildBB extends ChildB implements Pink {
	function GrandchildBB() {
		super();
		trace("GrandchildBB constructor");
	}
	
	function work() {
		super.work();
		trace("GrandchildBB work");
	}
}
