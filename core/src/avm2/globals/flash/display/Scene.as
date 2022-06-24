package flash.display {
    public final class Scene {
        private var _name: String;
        private var _labels: Array;
        private var _numFrames: int;

        public function Scene(name: String, labels: Array, numFrames: int) {
            this._name = name;
            this._labels = labels;
            this._numFrames = numFrames;
        }

        public function get name(): String {
            return this._name;
        }

        public function get labels(): Array {
            return this._labels;
        }

        public function get numFrames(): int {
            return this._numFrames;
        }
    }
}
