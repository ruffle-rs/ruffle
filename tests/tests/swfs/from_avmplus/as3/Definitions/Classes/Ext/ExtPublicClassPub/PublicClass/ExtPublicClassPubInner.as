/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


package PublicClass {

  import PublicClass.*;

  class ExtPublicClassPubInner extends PublicClass {

    // ************************************
    // access public method of parent
    // from default method of sub class
    // ************************************

    function subGetArray() : Array { return this.getPubArray(); }
    function subSetArray(a:Array) { this.setPubArray(a); }

    public function testSubArray(a:Array):Array{
        this.subSetArray(a);
        return this.subGetArray();
    }

    // ************************************
    // access public method of parent
    // from public method of sub class
    // ************************************

    public function pubSubGetArray() : Array { return this.getPubArray(); }
    public function pubSubSetArray(a:Array) { this.setPubArray(a); }

    // ************************************
    // access public method of parent
    // from private method of sub class
    // ************************************

    private function privSubGetArray() : Array { return this.getPubArray(); }
    private function privSubSetArray(a:Array) { this.setPubArray(a); }

    // function to test above from test scripts
    public function testPrivSubArray(a:Array) : Array {
        this.privSubSetArray(a);
        return this.privSubGetArray();
    }

    // ***************************************
    // access public method of parent
    // from public static method of sub class
    // ***************************************

    public static function pubStatSubGetArray():Array{ return getPubArray(); }

    // ***************************************
    // access public property from
    // default method of sub class
    // ***************************************

    function subGetDPArray() : Array { return this.pubArray; }
    function subSetDPArray(a:Array) { this.pubArray = a; }

    public function testSubDPArray(a:Array):Array{
        this.subSetDPArray(a);
        return this.subGetDPArray();
    }

    // ***************************************
    // access public property from
    // public method of sub class
    // ***************************************

    public function pubSubGetDPArray() : Array { return this.pubArray; }
    public function pubSubSetDPArray(a:Array) { this.pubArray = a; }

    // ***************************************
    // access public property from
    // private method of sub class
    // ***************************************

    private function privSubGetDPArray() : Array { return this.pubArray; }
    private function privSubSetDPArray(a:Array) { this.pubArray = a; }

    // ***************************************
    // access public property from
    // public static method of sub class
    // ***************************************

    public static function pubStatSubGetDPArray():Array{ return pubArray; }

  }

}
