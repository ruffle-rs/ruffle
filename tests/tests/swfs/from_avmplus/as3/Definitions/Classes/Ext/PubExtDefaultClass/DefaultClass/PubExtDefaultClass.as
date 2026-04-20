/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */



package DefaultClass {

  import DefaultClass.*;
  
  public class PubExtDefaultClass extends DefaultClass {

    // ************************************
    // access default method of parent
    // from default method of sub class
    // ************************************

    function subGetArray() : Array { return this.getArray(); }
    function subSetArray(a:Array) { this.setArray(a); }


    // ************************************
    // access default method of parent
    // from public method of sub class
    // ************************************

    public function pubSubGetArray() : Array { return this.getArray(); }
    public function pubSubSetArray(a:Array) { this.setArray(a); }

    // this is needed so that the test cases can access this from
    // outside the class.  This way the test case itself preserved
    public function testSubGetSetArray(a:Array) : Array {
        this.subSetArray(a);
        return this.subGetArray();
    }

    // ************************************
    // access default method of parent
    // from private method of sub class
    // ************************************

    private function privSubGetArray() : Array { return this.getArray(); }
    private function privSubSetArray(a:Array) { this.setArray(a); }

    // function to test above from test scripts
    public function testPrivSubArray(a:Array) : Array {
        this.privSubSetArray(a);
        return this.privSubGetArray();
    }

    // ************************************
    // access default method of parent
    // from final method of sub class
    // ************************************

    final function finSubGetArray() : Array { return this.getArray(); }
    final function finSubSetArray(a:Array) { this.setArray(a); }

    public function testFinSubArray(a:Array):Array{
        this.finSubSetArray(a);
        return this.finSubGetArray();
    }

    // ***************************************
    // access default property from
    // default method of sub class
    // ***************************************

    function subGetDPArray() : Array { return array; }
    function subSetDPArray(a:Array) { array = a; }

    public function testSubGetSetDPArray(a:Array) : Array {
        this.subSetDPArray(a);
        return this.subGetDPArray();
    }
   
    // ***************************************
    // access default property from
    // public method of sub class
    // ***************************************

    public function pubSubGetDPArray() : Array { return this.array; }
    public function pubSubSetDPArray(a:Array) { this.array = a; }

    // ***************************************
    // access default property from
    // private method of sub class
    // ***************************************
 
    private function privSubGetDPArray() : Array { return this.array; }
    private function privSubSetDPArray(a:Array) { this.array = a; }

    public function testPrivSubDPArray(a:Array) : Array {
        this.privSubSetDPArray(a);
        return this.privSubGetDPArray();
    }


    // ***************************************
    // access default property from
    // final method of sub class
    // ***************************************

    final function finSubGetDPArray() : Array { return array; }
    final function finSubSetDPArray(a:Array) { array = a; }

    public function testFinSubDPArray(a:Array):Array {
        this.finSubSetDPArray(a);
        return this.finSubGetDPArray();
    }

   
  }
}
