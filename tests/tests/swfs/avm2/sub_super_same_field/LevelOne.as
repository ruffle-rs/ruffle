package {
	public class LevelOne {
		public var pubSameName: String = "pubSameName-LevelOne";
		internal var internalSameName: String = "internalSameName-LevelOne";
		
		public function print() {
			trace("In LevelOne: " + this + " this.pubSameName = " + this.pubSameName + " this.internalSameName = " + this.internalSameName);
		}
	}
}