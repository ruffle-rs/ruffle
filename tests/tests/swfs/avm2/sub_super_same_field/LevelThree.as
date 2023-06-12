package {
	public class LevelThree extends LevelTwo {
		// Note - the final 'test.swf' has been manually edited
		// to remove '_SUFFIX', so that these fields have the same
		// names as the fields in LevelOne
		public var pubSameName_SUFFIX3 = "pubSameName-LevelThree";
		internal var internalSameName_SUFFIX3 = "pubSameName-LevelThree";
		
		override public function print() {
			super.print();
			trace("In LevelThree: " + this + " this.pubSameName = " + this.pubSameName + " this.internalSameName = " + this.internalSameName);
		}
	}
}