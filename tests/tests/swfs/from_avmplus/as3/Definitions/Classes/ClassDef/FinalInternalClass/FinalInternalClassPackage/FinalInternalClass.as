/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package FinalInternalClassPackage {
    final internal class FinalInternalClass {

        var array:Array;                            // Default property
        static var statFunction:Function;           // Default Static property
        var finNumber:Number;                   // Default Final property
        static var finStatNumber:Number;        // Default Final Static property
        
        internal var internalArray:Array;                           // Internal property
        internal static var internalStatFunction:Function;          // Internal Static property
        internal var internalFinNumber:Number;              // Internal Final property
        internal static var internalFinStatNumber:Number;       // Internal Final Static property

        private var privDate:Date;                              // Private property
        private static var privStatString:String;               // Private Static property
        private var privFinalString:String;             // Private Final property
        private static var privFinalStaticString:String // Private Final Static property

        public var pubBoolean:Boolean;                      // Public property
        public static var pubStatObject:Object;             // Public Static property
        public var pubFinArray:Array;                   // Public Final property
        public static var pubFinalStaticNumber:Number   // Public Final Static property

        // Testing property for constructor testing
        public static var constructorCount : int = 0;
        
        // *****************
        // Constructors
        // *****************
        function FinalInternalClass () {
            FinalInternalClass.constructorCount ++;
        }

        // *****************
        // Default methods
        // *****************
        function getArray() : Array { return array; }
        function setArray( a:Array ) { array = a; }
        
        
        // ************************
        // Default virtual methods
        // ************************
        virtual function getVirtualArray() : Array { return array; }
        virtual function setVirtualArray( a:Array ) { array = a; }
        
        
        // ***********************
        // Default Static methods
        // ***********************
        static function setStatFunction(f:Function) { statFunction = f; }
        static function getStatFunction() : Function { return statFunction; }

        
        // **********************
        // Default Final methods
        // **********************
        final function setFinNumber(n:Number) { finNumber = n; }
        final function getFinNumber() : Number { return finNumber; }

        
        // *****************
        // Internal methods
        // *****************
        internal function getInternalArray() : Array { return internalArray; }
        internal function setInternalArray( a:Array ) { internalArray = a; }
        
        
        // *************************
        // Internal virtual methods
        // *************************
        internal virtual function getInternalVirtualArray() : Array { return internalArray; }
        internal virtual function setInternalVirtualArray( a:Array ) { internalArray = a; }
        
        
        // ***********************
        // Internal Static methods
        // ***********************
        internal static function setInternalStatFunction(f:Function) { FinalInternalClass.internalStatFunction = f; }
        internal static function getInternalStatFunction() : Function { return FinalInternalClass.internalStatFunction; }
        
        
        // **********************
        // Internal Final methods
        // **********************
        internal final function setInternalFinNumber(n:Number) { internalFinNumber = n; }
        internal final function getInternalFinNumber() : Number { return internalFinNumber; }
        
        
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
        
        
        // *******************
        // Private virutal methods
        // *******************
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
        private static function setPrivStatString(s:String) { privStatString = s; }
        private static function getPrivStatString() : String { return privStatString; }
        // wrapper function
        public function testGetSetPrivStatString(s:String) : String {
            setPrivStatString(s);
            return getPrivStatString();
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
        
        
        // *******************
        // Public virtual methods
        // *******************
        public virtual function setPubVirtualBoolean( b:Boolean ) { pubBoolean = b; }
        public virtual function getPubVirtualBoolean() : Boolean { return pubBoolean; }


        // **************************
        // Public Static methods
        // **************************
        public static function setPubStatObject(o:Object) { FinalInternalClass.pubStatObject = o; }
        public static function getPubStatObject() : Object { return FinalInternalClass.pubStatObject; }


        // *******************
        // Public Final methods
        // *******************
        public final function setPubFinArray(a:Array) { pubFinArray = a; }
        public final function getPubFinArray() : Array { return pubFinArray; }


    }
}
