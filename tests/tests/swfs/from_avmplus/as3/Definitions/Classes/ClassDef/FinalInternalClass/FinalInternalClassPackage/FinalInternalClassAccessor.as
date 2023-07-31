/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package FinalInternalClassPackage {
    public class FinalInternalClassAccessor {
        private var Obj:FinalInternalClass = new FinalInternalClass();
        
        // Constructor tests
        public function testConstructorOne() : int {
            var foo = new FinalInternalClass();
            return FinalInternalClass.constructorCount;
        }
        public function testConstructorTwo() : int {
            var foo = new FinalInternalClass;
            return FinalInternalClass.constructorCount;
        }
        
        
        // Default method
        public function testGetSetArray(a:Array) : Array {
            Obj.setArray(a);
            return Obj.getArray();
        }
        // Default virtual method
        public function testGetSetVirtualArray(a:Array) : Array {
            Obj.setVirtualArray(a);
            return Obj.getVirtualArray();
        }
        // Default static method
        public function testGetSetStatFunction(f:Function) : Function {
            FinalInternalClass.setStatFunction(f);
            return FinalInternalClass.getStatFunction();
        }
        // Default final method
        public function testGetSetFinNumber(n:Number) : Number {
            Obj.setFinNumber(n);
            return Obj.getFinNumber();
        }
        
        // internal method
        public function testGetSetInternalArray(a:Array) : Array {
            Obj.setInternalArray(a);
            return Obj.getInternalArray();
        }
        // internal virtual method
        public function testGetSetInternalVirtualArray(a:Array) : Array {
            Obj.setInternalVirtualArray(a);
            return Obj.getInternalVirtualArray();
        }
        // internal static method
        public function testGetSetInternalStatFunction(f:Function) : Function {
            FinalInternalClass.setInternalStatFunction(f);
            return FinalInternalClass.getInternalStatFunction();
        }
        // internal final method
        public function testGetSetInternalFinNumber(n:Number) : Number {
            Obj.setInternalFinNumber(n);
            return Obj.getInternalFinNumber();
        }
        
        // private method
        public function testGetSetPrivDate(d:Date) : Date {
            return Obj.testGetSetPrivDate(d);
        }
        // private virtualmethod
        public function testGetSetPrivVirtualDate(d:Date) : Date {
            return Obj.testGetSetPrivVirtualDate(d);
        }
        // Private Static methods
        public function testGetSetPrivStatString(s:String) : String {
            return Obj.testGetSetPrivStatString(s);
        }
        // Private Final methods
        public function testGetSetPrivFinalString(s:String) : String {
            return Obj.testGetSetPrivFinalString(s);
        }
        
        // Public methods
        public function setPubBoolean( b:Boolean ) { Obj.setPubBoolean(b); }
        public function getPubBoolean() : Boolean { return Obj.getPubBoolean(); }
        // Public virtual methods
        public function setPubVirtualBoolean( b:Boolean ) { Obj.setPubVirtualBoolean(b); }
        public function getPubVirtualBoolean() : Boolean { return Obj.getPubVirtualBoolean(); }
        // Public Static methods
        public function setPubStatObject(o:Object) { FinalInternalClass.setPubStatObject(o); }
        public function getPubStatObject() : Object { return FinalInternalClass.getPubStatObject(); }
        // Public Final methods
        public function setPubFinArray(a:Array) { Obj.setPubFinArray(a); }
        public function getPubFinArray() : Array { return Obj.getPubFinArray(); }


    }
    
}
