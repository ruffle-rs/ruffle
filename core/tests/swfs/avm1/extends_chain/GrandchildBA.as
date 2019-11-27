class GrandchildBA extends ChildB {
	function GrandchildBA() {
		super();
		trace("GrandchildBA constructor");
	}
	
	function work() {
		super.work();
		trace("GrandchildBA work");
	}
}
