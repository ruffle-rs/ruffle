/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */




package DefaultClass {

    import DefaultClass.*

    dynamic class DynExtDefaultClassPubInner extends DefaultClass {

    // ************************************
    // access public method of parent
    // from default method of sub class
    // ************************************

    function subGetArray() : Array { return this.getPubArray(); }
    function subSetArray(a:Array) { this.setPubArray(a); }
    // function to test above from test scripts
    public function testSubArray(a:Array) : Array {
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

    // ************************************
    // access public method of parent
    // from final method of sub class
    // ************************************

    final function finSubGetArray() : Array { return this.getPubArray(); }
    final function finSubSetArray(a:Array) { this.setPubArray(a); }
    // function to test above from test scripts
    public function testFinSubArray(a:Array) : Array {
        this.finSubSetArray(a);
        return this.finSubGetArray();
    }

    // ************************************
    // access public method of parent
    // from public final method of sub class
    // ************************************

    public final function pubFinSubGetArray() : Array { return this.getPubArray(); }
    public final function pubFinSubSetArray(a:Array) { this.setPubArray(a); }
    // function to test above from test scripts
    public function testPubFinSubArray(a:Array) : Array {
        this.pubFinSubSetArray(a);
        return this.pubFinSubGetArray();
    }

    // ************************************
    // access public method of parent
    // from private final method of sub class
    // ************************************

    private final function privFinSubGetArray() : Array { return this.getPubArray(); }
    private final function privFinSubSetArray(a:Array) { this.setPubArray(a); }
    // function to test above from test scripts
    public function testPrivFinSubArray(a:Array) : Array {
        this.privFinSubSetArray(a);
        return this.privFinSubGetArray();
    }

    // ************************************
    // access public method of parent
    // from virtual method of sub class
    // ************************************

    virtual function virSubGetArray() : Array { return this.getPubArray(); }
    virtual function virSubSetArray(a:Array) { this.setPubArray(a); }
    // function to test above from test scripts
    public function testVirSubArray(a:Array) : Array {
        this.virSubSetArray(a);
        return this.virSubGetArray();
    }

     // ************************************
    // access public method of parent
    // from public virtual method of sub class
    // ************************************

    public virtual function pubVirSubGetArray() : Array { return this.getPubArray(); }
    public virtual function pubVirSubSetArray(a:Array) { this.setPubArray(a); }
    // function to test above from test scripts
    public function testPubVirSubArray(a:Array) : Array {
        this.pubVirSubSetArray(a);
        return this.pubVirSubGetArray();
    }

     // ************************************
    // access public method of parent
    // from private virtual method of sub class
    // ************************************

    private virtual function privVirSubGetArray() : Array { return this.getPubArray(); }
    private virtual function privVirSubSetArray(a:Array) { this.setPubArray(a); }
    // function to test above from test scripts
    public function testPrivVirSubArray(a:Array) : Array {
        this.pubVirSubSetArray(a);
        return this.pubVirSubGetArray();
    }

    // ***************************************
    // access public property from
    // default method of sub class
    // ***************************************

    function subGetDPArray() : Array { return this.pubArray; }
    function subSetDPArray(a:Array) { this.pubArray = a; }
    // function to test above from test scripts
    public function testSubDPArray(a:Array) : Array {
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
    // function to test above from test scripts
    public function testPrivDPArray(a:Array) : Array {
        this.privSubSetDPArray(a);
        return this.privSubGetDPArray();
    }

    // ***************************************
    // access public property from
    // final method of sub class
    // ***************************************

    final function finSubGetDPArray() : Array { return this.pubArray; }
    final function finSubSetDPArray(a:Array) { this.pubArray = a; }
    // function to test above from test scripts
    public function testFinDPArray(a:Array) : Array {
        this.finSubSetDPArray(a);
        return this.finSubGetDPArray();
    }

    // ***************************************
    // access public property from
    // public final method of sub class
    // ***************************************

    public final function pubFinSubGetDPArray() : Array { return this.pubArray; }
    public final function pubFinSubSetDPArray(a:Array) { this.pubArray = a; }
    // function to test above from test scripts
    public function testPubFinDPArray(a:Array) : Array {
        this.pubFinSubSetDPArray(a);
        return this.pubFinSubGetDPArray();
    }

    // ***************************************
    // access public property from
    // private final method of sub class
    // ***************************************

    private final function privFinSubGetDPArray() : Array { return this.pubArray; }
    private final function privFinSubSetDPArray(a:Array) { this.pubArray = a; }
    // function to test above from test scripts
    public function testPrivFinDPArray(a:Array) : Array {
        this.privFinSubSetDPArray(a);
        return this.pubFinSubGetDPArray();
    }

        // ***************************************
        // access public property from
        // virtual method of sub class
        // ***************************************

        virtual function virSubGetDPArray() : Array { return this.pubArray; }
        virtual function virSubSetDPArray(a:Array) { this.pubArray = a; }
        // function to test above from test scripts
        public function testVirDPArray(a:Array) : Array {
            this.virSubSetDPArray(a);
            return this.virSubGetDPArray();
        }

        // ***************************************
        // access public property from
        // public final method of sub class
        // ***************************************

        public virtual function pubVirSubGetDPArray() : Array { return this.pubArray; }
        public virtual function pubVirSubSetDPArray(a:Array) { this.pubArray = a; }
        // function to test above from test scripts
        public function testPubVirDPArray(a:Array) : Array {
            this.pubFinSubSetDPArray(a);
            return this.pubFinSubGetDPArray();
        }

        // ***************************************
        // access public property from
        // private final method of sub class
        // ***************************************

        private virtual function privVirSubGetDPArray() : Array { return this.pubArray; }
        private virtual function privVirSubSetDPArray(a:Array) { this.pubArray = a; }
        // function to test above from test scripts
        public function testPrivVirDPArray(a:Array) : Array {
            this.privFinSubSetDPArray(a);
            return this.pubFinSubGetDPArray();
    }

    }

}
