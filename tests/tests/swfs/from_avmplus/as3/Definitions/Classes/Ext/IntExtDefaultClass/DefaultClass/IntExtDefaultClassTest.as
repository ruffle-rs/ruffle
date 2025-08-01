/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


package DefaultClass {

  import DefaultClass.*

  // this is the test case so we can't alter it too much
  // change the name of the class to ExtDefaultClassTest
  // we'll create a public class ExtDefaultClass that
  // extends it for testing runtime
  internal class IntExtDefaultClassTest extends DefaultClass {

    // ************************************
    // access default method of parent
    // from default method of sub class
    // ************************************

    internal function subGetArray() : Array { return this.getArray(); }
    function subSetArray(a:Array) { this.setArray(a); }

    // this is needed so that the test cases can access this from
    // outside the class.  This way the test case itself preserved
    public function testSubGetSetArray(a:Array) : Array {
        this.subSetArray(a);
        return this.subGetArray();
    }

    // ************************************
    // access default method of parent
    // from public method of sub class
    // ************************************

    public function pubSubGetArray() : Array { return this.getArray(); }
    public function pubSubSetArray(a:Array) { this.setArray(a); }

    // ************************************
    // access default method of parent
    // from final method of sub class
    //
    // final will behave the same as default
    // ************************************

    internal final function finSubGetArray() : Array { return this.getArray(); }
    final function finSubSetArray(a:Array) { this.setArray(a); }

    public function testFinSubArray(a:Array):Array{
        this.finSubSetArray(a);
        return this.finSubGetArray();
    }

    // ************************************
    // access default method of parent
    // from public final method of sub class
    // ************************************

    public final function pubFinSubGetArray() : Array { return this.getArray(); }
    final public function pubFinSubSetArray(a:Array) { this.setArray(a); }

    // ************************************
    // access default method of parent
    // from private final method of sub class
    // ************************************

    private final function privFinSubGetArray() : Array { return this.getArray(); }
    final private function privFinSubSetArray(a:Array) { this.setArray(a); }

    public function testPrivFinSubArray(a:Array):Array {
        this.privFinSubSetArray(a);
        return this.privFinSubGetArray();
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
    // from virtual method of sub class
    // ************************************

    internal virtual function virtSubGetArray() : Array { return this.getArray(); }
    virtual function virtSubSetArray(a:Array) { this.setArray(a); }

    public function testVirtSubArray(a:Array) : Array {
        this.virtSubSetArray(a);
        return this.virtSubGetArray();
    }

    // ************************************
    // access default method of parent
    // from public virtual method of sub class
    // ************************************

    virtual public function pubVirtSubGetArray() : Array { return this.getArray(); }
    public virtual function pubVirtSubSetArray(a:Array) { this.setArray(a); }

    // ************************************
    // access default method of parent
    // from private virtual method of sub class
    // ************************************

    virtual private function privVirtSubGetArray() : Array { return this.getArray(); }
    private virtual function privVirtSubSetArray(a:Array) { this.setArray(a); }

    public function testPrivVirtSubArray(a:Array) : Array {
        this.privVirtSubSetArray(a);
        return this.privVirtSubGetArray();
    }

    // ***************************************
    // access default property from
    // static method of sub class
    // ***************************************

    internal static function statSubGetArray():Array{ return getArray(); }
    static public function pubStatSubGetArray():Array { return statSubGetArray(); }

    // ***************************************
    // access default property from
    // default method of sub class
    // ***************************************

    internal function subGetDPArray() : Array { return array; }
    function subSetDPArray(a:Array) { array = a; }

    // this is needed so that the test cases can access this from
    // outside the class.  This way the test case itself preserved
    public function testSubGetSetDPArray(a:Array) : Array {
        this.subSetDPArray(a);
        return this.subGetDPArray();
    }

    // ***************************************
    // access default property from
    // final method of sub class
    // ***************************************

    internal final function finSubGetDPArray() : Array { return array; }
    final function finSubSetDPArray(a:Array) { array = a; }

    public function testFinSubDPArray(a:Array) : Array {
        this.finSubSetDPArray(a);
        return this.finSubGetDPArray();
    }

    // ***************************************
    // access default property from
    // virtual method of sub class
    // ***************************************

    internal virtual function virtSubGetDPArray() : Array { return array; }
    virtual function virtSubSetDPArray(a:Array) { array = a; }

    public function testVirtSubDPArray(a:Array) : Array {
        this.virtSubSetDPArray(a);
        return this.virtSubGetDPArray();
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
    // public final method of sub class
    // ***************************************

    public final function pubFinSubGetDPArray() : Array { return this.array; }
    final public function pubFinSubSetDPArray(a:Array) { this.array = a; }

    // ***************************************
    // access default property from
    // public virtual method of sub class
    // ***************************************

    public virtual function pubVirtSubGetDPArray() : Array { return this.array; }
    virtual public function pubVirtSubSetDPArray(a:Array) { this.array = a; }

    // ***************************************
    // access default property from
    // private final method of sub class
    // ***************************************

    private final function privFinSubGetDPArray() : Array { return this.array; }
    final private function privFinSubSetDPArray(a:Array) { this.array = a; }

    public function testPrivFinSubDPArray(a:Array):Array {
        this.privFinSubSetDPArray(a);
        return this.privFinSubGetDPArray();
    }

    // ***************************************
    // access default property from
    // private virtual method of sub class
    // ***************************************

    private virtual function privVirtSubGetDPArray() : Array { return this.array; }
    virtual private function privVirtSubSetDPArray(a:Array) { this.array = a; }

    public function testPrivVirtSubDPArray(a:Array):Array {
        this.privVirtSubSetDPArray(a);
        return this.privVirtSubGetDPArray();
    }

    // ***************************************
    // access default property from
    // public static method of sub class
    // ***************************************

    public static function pubStatSubGetDPArray(){
        return array;
    }

  }

}
