package  {

	import flash.display.MovieClip;

	public class Test extends MovieClip {

		public function Test() {
			super();

			var m = outer.inner.transform.matrix;
			m.a = 0.5;
			m.d = 0.5;
			outer.inner.transform.matrix = m;
		}

	}

}
