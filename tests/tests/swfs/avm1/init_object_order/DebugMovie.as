
	class DebugMovie extends MovieClip {
		
		
		public function DebugMovie() {
			trace("Init movie");
		}
		
		public function set setter1(val) {
			trace("setter1: " + val);
			trace("this._x = " + this._x);
			trace("this.nonsetter = " + this["nonsetter"]);
		}
		
		public function set setter2(val) {
			trace("setter2: " + val);
			trace("this._x = " + this._x);
			trace("this.nonsetter = " + this["nonsetter"]);
		}
		
		public function set setter3(val) {
			trace("setter3: " + val);
			trace("this._x = " + this._x);
			trace("this.nonsetter = " + this["nonsetter"]);
		}
	}