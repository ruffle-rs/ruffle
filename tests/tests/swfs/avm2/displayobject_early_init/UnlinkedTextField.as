package {
	import flash.text.TextField;

	public class UnlinkedTextField extends TextField {
		public function UnlinkedTextField() {
			trace("UnlinkedTextField before super(): this.mouseEnabled = " + this.gridFitType);
			super();
			trace("UnlinkedTextField after super(): this.mouseEnabled = " + this.gridFitType);			
		}
	}
}