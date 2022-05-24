package
{
	public class Test {
		public function Test() {
			new Subclass();
		}
	}
}
	class BaseClass
	{
		public static var t = "static prop";

		public function instance_method()
		{
			trace("// Getting static property in instance method");
			trace(t);
		}

	}

	class Subclass extends BaseClass
	{
		public function Subclass()
		{
			trace("// Getting static property in subclass constructor");
			trace(t);
			instance_method();
		}

		public override function instance_method() {
			trace("Calling super!");
			super.instance_method();
		}
	}