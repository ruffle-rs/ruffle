/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package FinalPublicDynamicClassPackage {
    final public dynamic class FinalPublicDynamicClass {

        var array:Array;                            // Default property
        var finNumber:Number;                   // Default final property
        static var statFunction:Function;           // Default Static property
        static var finStatNumber:Number;        // Default final static property
        // TODO: virtual vars are not implemented yet
        // virtual var virtArray:Array;
        
        internal var internalArray:Array;                           // Internal property
        internal var internalFinNumber:Number;              // Internal final property
        internal static var internalStatFunction:Function;          // Internal static property
        internal static var internalFinStatNumber:Number;       // Internal Final Static property
        // TODO: virtual vars are not implemented yet
        // internal virtual var internalVirtNumber:Number;          // internal virtual property

        private var privDate:Date;                              // Private property
        private static var privStatString:String;               // Private Static property
        private var privFinalString:String;             // Private Final property
        private static var privFinalStaticString:String // Private Final Static property
        // TODO: virtual vars are not implemented yet
        // private virtual privVirtDate:Date;

        public var pubBoolean:Boolean;                      // Public property
        public static var pubStatObject:Object;             // Public Static property
        public var pubFinArray:Array;                   // Public Final property
        public static var pubFinalStaticNumber:Number   // Public Final Static property
        // TODO: virtual vars are not implemented yet
        // public virtual var pubVirtBoolean:Boolean;



        // *****************
        // Default methods
        // *****************
        function getArray() : Array { return array; }
        function setArray( a:Array ) { array = a; }
        // wrapper function
        public function testGetSetArray(a:Array) : Array {
            setArray(a);
            return getArray();
        }
        
        
        // ************************
        // Default virtual methods
        // ************************
        // TODO: virtual vars are not implemented yet so this is currently using a normal var
        virtual function getVirtualArray() : Array { return array; }
        virtual function setVirtualArray( a:Array ) { array = a; }
        // wrapper function
        public function testGetSetVirtualArray(a:Array) : Array {
            setVirtualArray(a);
            return getVirtualArray();
        }
        
        
        // ***********************
        // Default Static methods
        // ***********************
        static function setStatFunction(f:Function) { FinalPublicDynamicClass.statFunction = f; }
        static function getStatFunction() : Function { return FinalPublicDynamicClass.statFunction; }
        // wrapper function
        public function testGetSetStatFunction(f:Function) : Function {
            FinalPublicDynamicClass.setStatFunction(f);
            return FinalPublicDynamicClass.getStatFunction();
        }
        
        
        // **********************
        // Default Final methods
        // **********************
        final function setFinNumber(n:Number) { finNumber = n; }
        final function getFinNumber() : Number { return finNumber; }
        // wrapper function
        public function testGetSetFinNumber(n:Number) : Number {
            setFinNumber(n);
            return getFinNumber();
        }
        
        
        // *****************
        // Internal methods
        // *****************
        internal function getInternalArray() : Array { return internalArray; }
        internal function setInternalArray( a:Array ) { internalArray = a; }
        // wrapper function
        public function testGetSetInternalArray(a:Array) : Array {
            setInternalArray(a);
            return getInternalArray();
        }
        
        
        // *************************
        // Internal virtual methods
        // ************************
        // TODO: virtual vars are not implemented yet so this is currently using a normal var
        internal virtual function getInternalVirtualArray() : Array { return internalArray; }
        internal virtual function setInternalVirtualArray( a:Array ) { internalArray = a; }
        // wrapper function
        public function testGetSetInternalVirtualArray(a:Array) : Array {
            setInternalVirtualArray(a);
            return getInternalVirtualArray();
        }
        
        
        // ***********************
        // Internal Static methods
        // ***********************
        internal static function setInternalStatFunction(f:Function) { internalStatFunction = f; }
        internal static function getInternalStatFunction() : Function { return internalStatFunction; }
        // wrapper function
        public function testGetSetInternalStatFunction(f:Function) : Function {
            setInternalStatFunction(f);
            return getInternalStatFunction();
        }
        
        
        
        // **********************
        // Internal Final methods
        // **********************
        internal final function setInternalFinNumber(n:Number) { internalFinNumber = n; }
        internal final function getInternalFinNumber() : Number { return internalFinNumber; }
        // wrapper function
        public function testGetSetInternalFinNumber(n:Number) : Number {
            setInternalFinNumber(n);
            return getInternalFinNumber();
        }
        
        
        // *******************
        // Private methods
        // *******************
        private function getPrivDate() : Date { return privDate; }
        private function setPrivDate( d:Date ) { privDate = d; }
        // wrapper function
        public function testGetSetPrivDate(d:Date) : Date {
            setPrivDate(d);
            return getPrivDate();
        }

        
        // ************************
        // Private virtual methods
        // ***********************
        // TODO: virtual vars are not implemented yet so this is currently using a normal var
        private virtual function getPrivVirtualDate() : Date { return privDate; }
        private virtual function setPrivVirtualDate( d:Date ) { privDate = d; }
        // wrapper function
        public function testGetSetPrivVirtualDate(d:Date) : Date {
            setPrivVirtualDate(d);
            return getPrivVirtualDate();
        }
        

        // **************************
        // Private Static methods
        // **************************
        private static function setPrivStatString(s:String) { FinalPublicDynamicClass.privStatString = s; }
        private static function getPrivStatString() : String { return FinalPublicDynamicClass.privStatString; }
        // wrapper function
        public function testGetSetPrivStatString(s:String) : String {
            FinalPublicDynamicClass.setPrivStatString(s);
            return FinalPublicDynamicClass.getPrivStatString();
        }
        
        
        // **************************
        // Private Final methods
        // **************************
        private final function setPrivFinalString(s:String) { privFinalString = s; }
        private final function getPrivFinalString() : String { return privFinalString; }
        // wrapper function
        public function testGetSetPrivFinalString(s:String) : String {
            setPrivFinalString(s);
            return getPrivFinalString();
        }
        
        
        // *******************
        // Public methods
        // *******************
        public function setPubBoolean( b:Boolean ) { pubBoolean = b; }
        public function getPubBoolean() : Boolean { return pubBoolean; }
        
        
        // ************************
        // Public virutal methods
        // ***********************
        // TODO: virtual vars are not implemented yet so this is currently using a normal var
        public virtual function setPubVirtualBoolean( b:Boolean ) { pubBoolean = b; }
        public virtual function getPubVirtualBoolean() : Boolean { return pubBoolean; }


        // **************************
        // Public Static methods
        // **************************
        public static function setPubStatObject(o:Object) { FinalPublicDynamicClass.pubStatObject = o; }
        public static function getPubStatObject() : Object { return FinalPublicDynamicClass.pubStatObject; }


        // *******************
        // Public Final methods
        // *******************
        public final function setPubFinArray(a:Array) { pubFinArray = a; }
        public final function getPubFinArray() : Array { return pubFinArray; }

    }
}
