package flash.geom {
    public class ColorTransform {
        public var redMultiplier: Number;
        public var greenMultiplier: Number;
        public var blueMultiplier: Number;
        public var alphaMultiplier: Number;
        public var redOffset: Number;
        public var greenOffset: Number;
        public var blueOffset: Number;
        public var alphaOffset: Number;

        public function ColorTransform(redMultiplier: Number = 1, greenMultiplier: Number = 1, blueMultiplier: Number = 1, alphaMultiplier: Number = 1, redOffset: Number = 0, greenOffset: Number = 0, blueOffset: Number = 0, alphaOffset: Number = 0) {
            this.redMultiplier = redMultiplier;
            this.greenMultiplier = greenMultiplier;
            this.blueMultiplier = blueMultiplier;
            this.alphaMultiplier = alphaMultiplier;
            this.redOffset = redOffset;
            this.greenOffset = greenOffset;
            this.blueOffset = blueOffset;
            this.alphaOffset = alphaOffset;
        }

        public function get color(): uint {
            return (this.redOffset << 16) | (this.greenOffset << 8) | this.blueOffset;
        }

        public function set color(newColor: uint): void {
            this.redMultiplier = 0;
            this.greenMultiplier = 0;
            this.blueMultiplier = 0;
            this.redOffset = (newColor >> 16) & 0xFF;
            this.greenOffset = (newColor >> 8) & 0xFF;
            this.blueOffset = newColor & 0xFF;
        }

        public function concat(second: ColorTransform): void {
            this.alphaOffset += this.alphaMultiplier * second.alphaOffset;
            this.alphaMultiplier *= second.alphaMultiplier;
            this.redOffset += this.redMultiplier * second.redOffset;
            this.redMultiplier *= second.redMultiplier;
            this.greenOffset += this.greenMultiplier * second.greenOffset;
            this.greenMultiplier *= second.greenMultiplier;
            this.blueOffset += this.blueMultiplier * second.blueOffset;
            this.blueMultiplier *= second.blueMultiplier;
        }

        public function toString(): String {
            return "(redMultiplier=" + this.redMultiplier + ", greenMultiplier=" + this.greenMultiplier + ", blueMultiplier=" + this.blueMultiplier + ", alphaMultiplier=" + this.alphaMultiplier + ", redOffset=" + this.redOffset + ", greenOffset=" + this.greenOffset + ", blueOffset=" + this.blueOffset + ", alphaOffset=" + this.alphaOffset + ")";
        }
    }
}
