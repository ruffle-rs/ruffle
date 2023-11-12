package flash.accessibility {
    import flash.geom.Rectangle;

    public class AccessibilityImplementation {
        public var errno: uint;
        public var stub: Boolean;

        public function AccessibilityImplementation() {
            this.errno = 0;
            this.stub = false;
        }

        public function accDoDefaultAction(childID:uint):void { }

        public function accLocation(childID:uint):* {
            return null;
        }

        public function accSelect(operation:uint, childID:uint):void { }

        public function get_accDefaultAction(childID:uint):String {
            return null;
        }

        public function get_accFocus():uint {
            return 0;
        }

        public function get_accName(childID:uint):String {
            return null;
        }

        public function get_accRole(childID:uint):uint {
            throw new Error("Error #2143: AccessibilityImplementation.get_accRole() must be overridden from its default.", 2143);
        }

        public function get_accSelection():Array {
            return null;
        }

        public function get_accState(childID:uint):uint {
            throw new Error("Error #2144: AccessibilityImplementation.get_accState() must be overridden from its default.", 2144);
        }

        public function get_accValue(childID:uint):String {
            return null;
        }

        public function get_selectionActiveIndex():* {
            return this["selectionActiveIndex"];
        }

        public function get_selectionAnchorIndex():* {
            return this["selectionAnchorIndex"];
        }

        public function getChildIDArray():Array {
            return null;
        }

        public function isLabeledBy(labelBounds:Rectangle):Boolean {
            return false;
        }
    }
}
