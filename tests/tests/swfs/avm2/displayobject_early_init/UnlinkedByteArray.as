package {
	import flash.utils.ByteArray;

	public class UnlinkedByteArray extends ByteArray {
		public function UnlinkedByteArray() {
			trace("UnlinkedByteArray before super(): this.objectEncoding = " + this.objectEncoding);
			super();
			trace("UnlinkedByteArray after super(): this.objectEncoding = " + this.objectEncoding);			
		}
	}
}