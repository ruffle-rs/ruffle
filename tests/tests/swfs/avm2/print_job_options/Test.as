package {
import flash.display.MovieClip;
	public class Test extends MovieClip {
		import flash.printing.PrintJobOptions;
		public function Test() {
			var printJobOptions1:PrintJobOptions = new PrintJobOptions();
			trace(printJobOptions1.printAsBitmap);
			var printJobOptions2:PrintJobOptions = new PrintJobOptions(false);
			trace(printJobOptions2.printAsBitmap);
			var printJobOptions3:PrintJobOptions = new PrintJobOptions(true);
			trace(printJobOptions3.printAsBitmap);
		}
	}
}
