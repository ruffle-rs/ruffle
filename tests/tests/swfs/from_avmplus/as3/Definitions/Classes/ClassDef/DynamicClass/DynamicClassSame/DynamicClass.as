/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package DynamicClassSame {

    dynamic class DynamicClass {

        var array:Array = new Array(1,2,3);                     // Default property
        internal var intNumber:Number = 100;                    // internal property
        protected var protInt:int = -1;                     // protected property
        public var pubUint:uint = 1;                            // public property
        private var privVar:Boolean = true;                     // private property
        public static var pubStatBoolean:Boolean = true;        // public static property
        ns var nsProp:String = "nsProp";                        // namespace property

        // default method
        function defaultMethod():Boolean{ return true; }
        
        // Internal method
        internal function internalMethod(n:Number):int { return n; }
    
        // protected method
        protected function protectedMethod():uint { return 1; }

        // public method
        public function publicMethod():Boolean { return true; }

        // private method
        private function privateMethod():Boolean { return true; }

        // namespace method
        ns function nsMethod():String { return "nsMethod"; }

        // public final method
        public final function publicFinalMethod():Number { return 1; }

        // public static method
        public static function publicStaticMethod():int { return 42; }

        // access private property from same class same package
        function accPrivProp():Boolean { return this.privVar; }

        // access private method from same class same package
        function accPrivMethod():Boolean { return this.privateMethod(); }
        
    }

    


    
}
