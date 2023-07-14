package {
	import flash.display.MovieClip;
	import flash.system.ApplicationDomain;
	
	public class Test {
		public function Test(main: MovieClip) {
			var domain = main.stage.loaderInfo.applicationDomain;
			trace("ApplicationDomain.currentDomain.parentDomain: " + ApplicationDomain.currentDomain.parentDomain);
			trace("Stage getQualifiedDefinitionNames(): " + domain.getQualifiedDefinitionNames());
			trace("Stage parent: " + domain.parentDomain);
			trace("ApplicationDomain.currentDomain.getQualifiedDefinitionNames(): " + ApplicationDomain.currentDomain.getQualifiedDefinitionNames());
			trace("ApplicationDomain.currentDomain === ApplicationDomain.currentDomain: " + (ApplicationDomain.currentDomain === ApplicationDomain.currentDomain));
		}
	}
}