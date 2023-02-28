package {
	public class Test {
		public static function test() {
			trace("Calling directly");
			var concrete = new Concrete();
			concrete.base_interface();
			concrete.parent_one();
			concrete.parent_two();
			concrete.grandparent_one();
			concrete.grandparent_two();
			
			trace();
			trace("Calling through string lookup");
			concrete["base_interface"]();
			concrete["parent_one"]();
			concrete["parent_two"]();
			concrete["grandparent_one"]();
			concrete["grandparent_two"]();
			
			trace();
			trace("Calling through interface");
			var launder: BaseInterface = concrete;
			launder.base_interface();
			launder.parent_one();
			launder.parent_two();
			launder.grandparent_one();
			launder.grandparent_two();
		}
	}
}

class Concrete implements BaseInterface {
	public function base_interface() { trace("BaseInterface method") }
	public function parent_one() { trace("ParentOne method"); }
	public function parent_two() { trace("ParentTwo method"); }
	public function grandparent_one() { trace("GrandParentOne method"); }
	public function grandparent_two() { trace("GrandParentTwo method"); }
}

interface BaseInterface extends ParentOne, ParentTwo {
	function base_interface();
}
interface ParentOne extends GrandParentOne {
	function parent_one();
}
interface ParentTwo extends GrandParentOne, GrandParentTwo {
	function parent_two();
}
interface GrandParentOne {
	function grandparent_one();
}
interface GrandParentTwo {
	function grandparent_two();
}