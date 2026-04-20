/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */



package PublicClass {

  import PublicClass.*;

  class ExtPublicClassInner extends PublicClass {

    // ************************************
    // access private method of parent
    // from default method of sub class
    // ************************************

    function subGetArray() : Array { return this.getPrivArray(); }
    function subSetArray(a:Array) { this.setPrivArray(a); }

    public function testSubGetSetArray(a:Array) : Array {
        this.subSetArray(a);
        return this.subGetArray();
    }


    // ************************************
    // access private method of parent
    // from public method of sub class
    // ************************************

    public function pubSubGetArray() : Array { return this.getPrivArray(); }
    public function pubSubSetArray(a:Array) { this.setPrivArray(a); }

    // ************************************
    // access private method of parent
    // from private method of sub class
    // ************************************

    private function privSubGetArray() : Array { return this.getPrivArray(); }
    private function privSubSetArray(a:Array) { this.setPrivArray(a); }

    // function to test above from test scripts
    public function testPrivSubArray(a:Array) : Array {
        this.privSubSetArray(a);
        return this.privSubGetArray();
    }

    // ***************************************
    // access private property from
    // default method of sub class
    // ***************************************

    function subGetDPArray() : Array { return this.privArray; }
    function subSetDPArray(a:Array) { this.privArray = a; }

    public function testSubGetSetDPArray(a:Array) : Array {
        //this.subSetDPArray(a);
        return this.subGetDPArray();
    }


    // ***************************************
    // access private property from
    // public method of sub class
    // ***************************************

    public function pubSubGetDPArray() : Array { return this.privArray; }
    public function pubSubSetDPArray(a:Array) { this.privArray = a; }

    // ***************************************
    // access private property from
    // private method of sub class
    // ***************************************

    private function privSubGetDPArray() : Array { return this.privArray; }
    private function privSubSetDPArray(a:Array) { this.privArray = a; }

    public function testPrivSubDPArray(a:Array) : Array {
        this.privSubSetDPArray(a);
        return this.privSubGetDPArray();
    }

    // ***************************************
    // access private property from public static sub method
    // ***************************************
    public static function pubStatSubGetDPArray() { return privArray; }

  }

}
