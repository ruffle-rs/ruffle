package {
	import flash.display.MovieClip;

	public class FirstChild extends MovieClip {
		public var fourthChild;
		
		public function FirstChild() {
			var self = this;
			this.addFrameScript(1, function() {
				trace("FirstChild framescript frame 2: this.fourthChild = " + self.fourthChild);
			});
			trace("Calling FirstChild super(): Container.firstChild = " + Container.INSTANCE.firstChild + " Container.secondChild = " + Container.INSTANCE.secondChild + " Container.thirdChildDummy = " + Container.INSTANCE.thirdChildDummy);
			super();
			trace("Called FirstChild super(): Container.firstChild = " + Container.INSTANCE.firstChild + " Container.secondChild = " + Container.INSTANCE.secondChild + " Container.thirdChildDummy = " + Container.INSTANCE.thirdChildDummy);
			this.gotoAndStop(2);
			trace("FirstChild: ran this.gotoAndStop(2): Container.firstChild = " + Container.INSTANCE.firstChild + " Container.secondChild = " + Container.INSTANCE.secondChild + " Container.thirdChildDummy = " + Container.INSTANCE.thirdChildDummy + " this.stage.getChildAt(0).getChildAt(1) = " + MovieClip(this.stage.getChildAt(0)).getChildAt(1)); 
		}
	}
}