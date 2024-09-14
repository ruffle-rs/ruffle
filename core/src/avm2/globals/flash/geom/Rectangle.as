package flash.geom {
    public class Rectangle {
        public var x: Number;
        public var y: Number;
        public var width: Number;
        public var height: Number;

        public function Rectangle(x: Number = 0, y: Number = 0, width: Number = 0, height: Number = 0) {
            this.x = x;
            this.y = y;
            this.width = width;
            this.height = height;
        }

        public function get left(): Number {
            return this.x;
        }

        public function set left(value: Number): void {
            this.width += this.x - value;
            this.x = value;
        }

        public function get right(): Number {
            return this.x + this.width;
        }

        public function set right(value: Number): void {
            this.width = value - this.x;
        }

        public function get top(): Number {
            return this.y;
        }

        public function set top(value: Number): void {
            this.height += this.y - value;
            this.y = value;
        }

        public function get bottom(): Number {
            return this.y + this.height;
        }

        public function set bottom(value: Number): void {
            this.height = value - this.y;
        }

        public function get topLeft(): Point {
            return new Point(this.x, this.y);
        }

        public function set topLeft(value: Point): void {
            this.width += this.x - value.x;
            this.height += this.y - value.y;
            this.x = value.x;
            this.y = value.y;
        }

        public function get bottomRight(): Point {
            return new Point(this.right, this.bottom);
        }

        public function set bottomRight(value: Point): void {
            this.width = value.x - this.x;
            this.height = value.y - this.y;
        }

        public function get size(): Point {
            return new Point(this.width, this.height);
        }

        public function set size(value: Point): void {
            this.width = value.x;
            this.height = value.y;
        }

        public function clone(): Rectangle {
            return new Rectangle(this.x, this.y, this.width, this.height);
        }

        public function isEmpty(): Boolean {
            return this.width <= 0 || this.height <= 0;
        }

        public function setEmpty(): void {
            this.x = 0;
            this.y = 0;
            this.width = 0;
            this.height = 0;
        }

        public function inflate(dx: Number, dy: Number): void {
            this.x -= dx;
            this.width += 2 * dx;
            this.y -= dy;
            this.height += 2 * dy;
        }

        public function inflatePoint(point: Point): void {
            this.x -= point.x;
            this.width += 2 * point.x;
            this.y -= point.y;
            this.height += 2 * point.y;
        }

        public function offset(dx: Number, dy: Number): void {
            this.x += dx;
            this.y += dy;
        }

        public function offsetPoint(point: Point): void {
            this.x += point.x;
            this.y += point.y;
        }

        public function contains(x: Number, y: Number): Boolean {
            return x >= this.x && x < this.x + this.width && y >= this.y && y < this.y + this.height;
        }

        public function containsPoint(point: Point): Boolean {
            return point.x >= this.x && point.x < this.x + this.width && point.y >= this.y && point.y < this.y + this.height;
        }

        public function containsRect(rect: Rectangle): Boolean {
            var r1 = rect.x + rect.width;
            var b1 = rect.y + rect.height;
            var r2 = this.x + this.width;
            var b2 = this.y + this.height;
            return rect.x >= this.x && rect.x < r2 && rect.y >= this.y && rect.y < b2 && r1 > this.x && r1 <= r2 && b1 > this.y && b1 <= b2;
        }

        public function intersection(toIntersect: Rectangle): Rectangle {
            var result = new Rectangle();
            if (this.isEmpty() || toIntersect.isEmpty()) {
                result.setEmpty();
                return result;
            }
            result.x = Math.max(this.x, toIntersect.x);
            result.y = Math.max(this.y, toIntersect.y);
            result.width = Math.min(this.x + this.width, toIntersect.x + toIntersect.width) - result.x;
            result.height = Math.min(this.y + this.height, toIntersect.y + toIntersect.height) - result.y;
            if (result.width <= 0 || result.height <= 0) {
                result.setEmpty();
            }
            return result;
        }

        public function intersects(toIntersect: Rectangle): Boolean {
            if (this.isEmpty() || toIntersect.isEmpty()) {
                return false;
            }
            var resultx = Math.max(this.x, toIntersect.x);
            var resulty = Math.max(this.y, toIntersect.y);
            var resultwidth = Math.min(this.x + this.width, toIntersect.x + toIntersect.width) - resultx;
            var resultheight = Math.min(this.y + this.height, toIntersect.y + toIntersect.height) - resulty;
            if (resultwidth <= 0 || resultheight <= 0) {
                return false;
            }
            return true;
        }

        public function union(toUnion: Rectangle): Rectangle {
            if (this.isEmpty()) {
                return toUnion.clone();
            }
            if (toUnion.isEmpty()) {
                return this.clone();
            }
            var r = new Rectangle();
            r.x = Math.min(this.x, toUnion.x);
            r.y = Math.min(this.y, toUnion.y);
            r.width = Math.max(this.x + this.width, toUnion.x + toUnion.width) - r.x;
            r.height = Math.max(this.y + this.height, toUnion.y + toUnion.height) - r.y;
            return r;
        }

        public function equals(toCompare: Rectangle): Boolean {
            return toCompare.x == this.x && toCompare.y == this.y && toCompare.width == this.width && toCompare.height == this.height;
        }

        public function toString(): String {
            return "(x=" + this.x + ", y=" + this.y + ", w=" + this.width + ", h=" + this.height + ")";
        }

        [API("674")]
        public function copyFrom(sourceRect: Rectangle): void {
            this.x = sourceRect.x;
            this.y = sourceRect.y;
            this.width = sourceRect.width;
            this.height = sourceRect.height;
        }

        [API("674")]
        public function setTo(xa: Number, ya: Number, widtha: Number, heighta: Number): void {
            this.x = xa;
            this.y = ya;
            this.width = widtha;
            this.height = heighta;
        }
    }
}
