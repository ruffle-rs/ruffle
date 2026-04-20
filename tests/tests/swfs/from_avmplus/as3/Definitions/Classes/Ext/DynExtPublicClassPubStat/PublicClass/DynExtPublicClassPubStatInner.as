/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


package PublicClass {

        import PublicClass.*;

    dynamic class DynExtPublicClassPubStatInner extends PublicClass {

        // ************************************
        // access public static method of parent
        // from default method of sub class
        // ************************************

        function subGetArray() : Array { return getPubStatArray(); }
        function subSetArray(a:Array)  { setPubStatArray(a); }
        // function to test above from test scripts
        public function testSubArray(a:Array) : Array {
        this.subSetArray(a);
        return this.subGetArray();
        }

        // ************************************
        // access public static method of parent
        // from public method of sub class
        // ************************************

        public function pubSubGetArray() : Array { return getPubStatArray(); }
        public function pubSubSetArray(a:Array) { setPubStatArray(a); }

        // ************************************
        // access public static method of parent
        // from private method of sub class
        // ************************************

        private function privSubGetArray() : Array { return getPubStatArray(); }
        private function privSubSetArray(a:Array) { setPubStatArray(a); }

        // function to test above from test scripts
        public function testPrivSubArray(a:Array) : Array {
        this.privSubSetArray(a);
        return this.privSubGetArray();
        }

        // ***************************************
        // access public static method of parent
        // from static method of sub class
        // ***************************************

        static function statSubGetArray() : Array { return getPubStatArray(); }
        static function statSubSetArray(a:Array) { setPubStatArray(a); }

        // ***************************************
        // access public static method of parent
        // from public static method of sub class
        // ***************************************

        public static function pubStatSubGetArray() : Array { return getPubStatArray(); }
        public static function pubStatSubSetArray(a:Array) { setPubStatArray(a); }

        // ***************************************
        // access public static method of parent
        // from private static method of sub class
        // ***************************************

        private static function privStatSubGetArray() : Array { return getPubStatArray(); }
        private static function privStatSubSetArray(a:Array) { setPubStatArray(a); }

        // public accessor to test asrt
        public function testPrivStatSubArray(a:Array) : Array {
        privStatSubSetArray(a);
        return privStatSubGetArray();
        }

        // ***************************************
        // access public static method of parent
        // from final method of sub class
        // ***************************************

        final function finSubGetArray() : Array { return getPubStatArray(); }
        final function finSubSetArray(a:Array) { setPubStatArray(a); }
        // public accessor to test asrt
        public function testFinSubArray(a:Array) : Array {
        finSubSetArray(a);
        return finSubGetArray();
        }


        // ***************************************
        // access public static property from
        // default method of sub class
        // ***************************************

        function subGetDPArray() : Array { return pubStatArray; }
        function subSetDPArray(a:Array) { pubStatArray = a; }

        // ***************************************
        // access public static property from
        // public method of sub class
        // ***************************************

        public function pubSubGetDPArray() : Array { return pubStatArray; }
        public function pubSubSetDPArray(a:Array) { pubStatArray = a; }

        // ***************************************
        // access public static property from
        // private method of sub class
        // ***************************************

        private function privSubGetDPArray() : Array { return pubStatArray; }
        private function privSubSetDPArray(a:Array) { pubStatArray = a; }

        // ***************************************
        // access public static property from
        // public static method of sub class
        // ***************************************

        public static function pubStatSubGetSPArray() : Array { return pubStatArray; }
        public static function pubStatSubSetSPArray(a:Array) { pubStatArray = a; }

        // ***************************************
        // access public static property from
        // private static method of sub class
        // ***************************************

        private static function privStatSubGetSPArray() : Array { return pubStatArray; }
        private static function privStatSubSetSPArray(a:Array) { pubStatArray = a; }

        // public accessor for asrt
        public function testPrivStatSubPArray(a:Array) : Array {
        privStatSubSetSPArray( a );
        return privStatSubGetSPArray();
        }

        // ***************************************
        // access public static property from
        // final method of sub class
        // ***************************************

        final function finSubGetDPArray() : Array { return pubStatArray; }
        final function finSubSetDPArray(a:Array) { pubStatArray = a; }


    }
}
