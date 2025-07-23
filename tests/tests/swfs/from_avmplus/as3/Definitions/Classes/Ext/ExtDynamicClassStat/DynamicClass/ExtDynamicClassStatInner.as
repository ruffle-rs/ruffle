/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


package DynamicClass {

 import DynamicClass.*;

 class ExtDynamicClassStatInner extends DynamicClass {

    // ************************************
    // access static method of parent
    // from default method of sub class
    // ************************************

    function subGetArray() : Array { return getStatArray(); }
    function subSetArray(a:Array) { setStatArray(a); }

    public function testSubSetArray(a:Array) : Array {
        this.subSetArray(a);
        return this.subGetArray();
    }


    // ************************************
    // access static method of parent
    // from public method of sub class
    // ************************************

    public function pubSubGetArray() : Array { return getStatArray(); }
    public function pubSubSetArray(a:Array) { setStatArray(a); }

    // ************************************
    // access static method of parent
    // from private method of sub class
    // ************************************

    private function privSubGetArray() : Array { return getStatArray(); }
    private function privSubSetArray(a:Array) { setStatArray(a); }

    // function to test above from test scripts
    public function testPrivSubArray(a:Array) : Array {
        this.privSubSetArray(a);
        return this.privSubGetArray();
    }

    // ***************************************
    // access static method of parent
    // from static method of sub class
    // ***************************************

    static function statSubGetArray() : Array { return getStatArray(); }
    static function statSubSetArray(a:Array) { setStatArray(a); }

    //public static function testStatSubSetArray(a:Array) : Array {
    public function testStatSubSetArray(a:Array) : Array {
        statSubSetArray(a);
        return statSubGetArray();
    }


    // ***************************************
    // access static method of parent
    // from public static method of sub class
    // ***************************************

    public static function pubStatSubGetArray() : Array { return getStatArray(); }
    public static function pubStatSubSetArray(a:Array) { setStatArray(a); }

    public function testPubStatSubGetSetArray(a:Array):Array {
        pubStatSubSetArray(a);
        return pubStatSubGetArray();
    }

    // ***************************************
    // access static method of parent
    // from private static method of sub class
    // ***************************************

    private static function privStatSubGetArray() : Array { return getStatArray(); }
    private static function privStatSubSetArray(a:Array) { setStatArray(a); }

    // public accessor to test asrt
    public function testPrivStatSubArray(a:Array) : Array {
        privStatSubSetArray(a);
        return privStatSubGetArray();
    }

    // ***************************************
    // access static property from
    // default method of sub class
    // ***************************************

    function subGetDPArray() : Array { return statArray; }
    function subSetDPArray(a:Array) { statArray = a; }


    public function testSubSetDPArray(a:Array) : Array {
        this.subSetDPArray(a);
        return this.subGetDPArray();
    }


    // ***************************************
    // access static property from
    // public method of sub class
    // ***************************************

    public function pubSubGetDPArray() : Array { return statArray; }
    public function pubSubSetDPArray(a:Array) { statArray = a; }

    // ***************************************
    // access static property from
    // private method of sub class
    // ***************************************

    private function privSubGetDPArray() : Array { return statArray; }
    private function privSubSetDPArray(a:Array) { statArray = a; }

    public function testPrivSubSetDPArray(a:Array) : Array {
        this.privSubSetDPArray(a);
        return this.privSubGetDPArray();
    }


    // ***************************************
    // access static property from
    // static method of sub class
    // ***************************************

    static function statSubGetSPArray() : Array { return statArray; }
    static function statSubSetSPArray(a:Array) { statArray = a; }

    public function testStatSubSetDPArray(a:Array) : Array {
    //public static function testStatSubSetDPArray(a:Array) : Array {
        statSubSetSPArray(a);
        return statSubGetSPArray();
    }


    // ***************************************
    // access static property from
    // public static method of sub class
    // ***************************************

    public static function pubStatSubGetSPArray() : Array { return statArray; }
    public static function pubStatSubSetSPArray(a:Array) { statArray = a; }

    public function testPubStatSubGetSetSPArray(a:Array):Array{
        pubStatSubSetSPArray(a);
        return pubStatSubGetSPArray();
    }

    // ***************************************
    // access static property from
    // private static method of sub class
    // ***************************************

    private static function privStatSubGetSPArray() : Array { return statArray; }
    private static function privStatSubSetSPArray(a:Array) { statArray = a; }

    // public accessor for asrt
    public function testPrivStatSubPArray(a:Array) : Array {
        privStatSubSetSPArray( a );
        return privStatSubGetSPArray();
    }
 }

}
