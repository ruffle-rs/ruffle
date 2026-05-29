package  {

	import flash.display.MovieClip;
	import flash.geom.ColorTransform;

	public class Test extends MovieClip {

		public function Test() {
			super();

			var ct:ColorTransform = new ColorTransform();
			ct.color = 0xFF0000;

			outer.inner.transform.colorTransform = ct;
		}

	}

}
